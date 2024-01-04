use anyhow::{format_err, Result};
use io::Error as IoError;
use thiserror::Error;
use tracing_subscriber::EnvFilter;
// use std::collections::HashMap;
use std::env;
// use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
// use std::str::FromStr;
use getopts::{Matches, Options};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("MOVEFMT_LOG"))
        .init();
    let opts = make_opts();

    let exit_code = match execute(&opts) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e:#}");
            1
        }
    };
    // Make sure standard output is flushed before we exit.
    std::io::stdout().flush().unwrap();

    // Exit with given exit code.
    //
    // NOTE: this immediately terminates the process without doing any cleanup,
    // so make sure to finish all necessary cleanup before this is called.
    std::process::exit(exit_code);
}

/// movefmt operations.
enum Operation {
    /// Format files and their child modules.
    Format {
        files: Vec<PathBuf>,
        minimal_config_path: Option<String>,
    },
    /// Print the help message.
    Help(HelpOp),
    /// Print version information
    Version,
    /// Output default config to a file, or stdout if None
    ConfigOutputDefault { path: Option<String> },
    /// Output current config (as if formatting to a file) to stdout
    ConfigOutputCurrent { path: Option<String> },
    /// No file specified, read from stdin
    Stdin { input: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum EmitMode {
    /// Emits to files.
    Files,
    /// Writes the output to stdout.
    Stdout,
    /// Displays how much of the input file was processed
    Coverage,
    /// Unfancy stdout
    Checkstyle,
    /// Writes the resulting diffs in a JSON format. Returns an empty array
    /// `[]` if there were no diffs.
    Json,
    /// Output the changed lines (for internal value only)
    ModifiedLines,
    /// Checks if a diff can be generated. If so, movefmt outputs a diff and
    /// quits with exit code 1.
    /// This option is designed to be run in CI where a non-zero exit signifies
    /// non-standard code formatting. Used for `--check`.
    Diff,
}

/// movefmt operations errors.
#[derive(Error, Debug)]
pub enum OperationError {
    /// An unknown help topic was requested.
    #[error("Unknown help topic: `{0}`.")]
    UnknownHelpTopic(String),
    /// An unknown print-config option was requested.
    #[error("Unknown print-config option: `{0}`.")]
    UnknownPrintConfigTopic(String),
    /// Attempt to generate a minimal config from standard input.
    #[error("The `--print-config=minimal` option doesn't work with standard input.")]
    MinimalPathWithStdin,
    /// An io error during reading or writing.
    #[error("{0}")]
    IoError(IoError),
}

impl From<IoError> for OperationError {
    fn from(e: IoError) -> OperationError {
        OperationError::IoError(e)
    }
}

/// Arguments to `--help`
enum HelpOp {
    None,
    Config,
}

fn make_opts() -> Options {
    let mut opts = Options::new();

    opts.optflag(
        "",
        "check",
        "Run in 'check' mode. Exits with 0 if input is formatted correctly. Exits \
         with 1 and prints a diff if formatting is required.",
    );
    let emit_opts = "[files|stdout]";
    opts.optopt("", "emit", "What data to emit and how", emit_opts);
    opts.optopt(
        "",
        "config-path",
        "Recursively searches the given path for the movefmt.toml config file. If not \
         found reverts to the input file path",
        "[Path for the configuration file]",
    );
    opts.optopt(
        "",
        "print-config",
        "Dumps a default or minimal config to PATH. A minimal config is the \
         subset of the current config file used for formatting the current program. \
         `current` writes to stdout current config as if formatting the file at PATH.",
        "[default|minimal|current] PATH",
    );
    opts.optflag(
        "l",
        "files-with-diff",
        "Prints the names of mismatched files that were formatted. Prints the names of \
         files that would be formatted when used with `--check` mode. ",
    );
    opts.optmulti(
        "",
        "config",
        "Set options from command line. These settings take priority over .movefmt.toml",
        "[key1=val1,key2=val2...]",
    );

    opts.optflag("v", "verbose", "Print verbose output");
    opts.optflag("q", "quiet", "Print less output");
    opts.optflag("V", "version", "Show version information");
    let help_topics = "`config`";
    let mut help_topic_msg = "Show this message or help about a specific topic: ".to_owned();
    help_topic_msg.push_str(help_topics);

    opts.optflagopt("h", "help", &help_topic_msg, "=TOPIC");

    opts
}

// Returned i32 is an exit code
fn execute(opts: &Options) -> Result<i32> {
    let matches = opts.parse(env::args().skip(1))?;
    let options = GetOptsOptions::from_matches(&matches)?;

    match determine_operation(&matches)? {
        Operation::Help(HelpOp::None) => {
            print_usage_to_stdout(opts, "");
            Ok(0)
        }
        Operation::Help(HelpOp::Config) => {
            print_usage_to_stdout(opts, "");
            Ok(0)
        }
        Operation::Version => {
            print_version();
            Ok(0)
        }
        Operation::Stdin { input } => format_string(input, options),
        Operation::Format {
            files,
            minimal_config_path,
        } => format(files, minimal_config_path, &options),
        _ => Ok(0)
    }
}

fn format_string(input: String, options: GetOptsOptions) -> Result<i32> {
    Ok(0)
}

fn format(
    files: Vec<PathBuf>,
    minimal_config_path: Option<String>,
    options: &GetOptsOptions,
) -> Result<i32> {
    Ok(0)
}

fn print_usage_to_stdout(opts: &Options, reason: &str) {
    let sep = if reason.is_empty() {
        String::new()
    } else {
        format!("{reason}\n\n")
    };
    let msg = format!("{sep}Format Move code\n\nusage: movefmt [options] <file>...");
    println!("{}", opts.usage(&msg));
}

fn print_version() {
    println!("movefmt v0.0.1");
}

fn determine_operation(matches: &Matches) -> Result<Operation, OperationError> {
    if matches.opt_present("h") {
        let topic = matches.opt_str("h");
        if topic == None {
            return Ok(Operation::Help(HelpOp::None));
        } else if topic == Some("config".to_owned()) {
            return Ok(Operation::Help(HelpOp::Config));
        } else {
            return Err(OperationError::UnknownHelpTopic(topic.unwrap()));
        }
    }
    let mut free_matches = matches.free.iter();

    let mut minimal_config_path = None;
    if let Some(kind) = matches.opt_str("print-config") {
        let path = free_matches.next().cloned();
        match kind.as_str() {
            "default" => return Ok(Operation::ConfigOutputDefault { path }),
            "current" => return Ok(Operation::ConfigOutputCurrent { path }),
            "minimal" => {
                minimal_config_path = path;
                if minimal_config_path.is_none() {
                    eprintln!("WARNING: PATH required for `--print-config minimal`.");
                }
            }
            _ => {
                return Err(OperationError::UnknownPrintConfigTopic(kind));
            }
        }
    }

    if matches.opt_present("version") {
        return Ok(Operation::Version);
    }

    let files: Vec<_> = free_matches
        .map(|s| {
            let p = PathBuf::from(s);
            // we will do comparison later, so here tries to canonicalize first
            // to get the expected behavior.
            p.canonicalize().unwrap_or(p)
        })
        .collect();

    // if no file argument is supplied, read from stdin
    if files.is_empty() {
        if minimal_config_path.is_some() {
            return Err(OperationError::MinimalPathWithStdin);
        }
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        return Ok(Operation::Stdin { input: buffer });
    }

    Ok(Operation::Format {
        files,
        minimal_config_path,
    })
}

// const STABLE_EMIT_MODES: [EmitMode; 3] = [EmitMode::Files, EmitMode::Stdout, EmitMode::Diff];

/// Parsed command line options.
#[derive(Clone, Debug, Default)]
struct GetOptsOptions {
    quiet: bool,
    verbose: bool,    
    emit_mode: Option<EmitMode>,
    check: bool,
    print_misformatted_file_names: bool,
}

impl GetOptsOptions {
    pub fn from_matches(matches: &Matches) -> Result<GetOptsOptions> {
        let mut options = GetOptsOptions::default();
        options.verbose = matches.opt_present("verbose");
        options.quiet = matches.opt_present("quiet");
        if options.verbose && options.quiet {
            return Err(format_err!("Can't use both `--verbose` and `--quiet`"));
        }

        options.check = matches.opt_present("check");
        if let Some(ref emit_str) = matches.opt_str("emit") {
            if options.check {
                return Err(format_err!("Invalid to use `--emit` and `--check`"));
            }

            options.emit_mode = Some(emit_mode_from_emit_str(emit_str)?);
        }

        if matches.opt_present("files-with-diff") {
            options.print_misformatted_file_names = true;
        }

        Ok(options)
    }

}

fn emit_mode_from_emit_str(emit_str: &str) -> Result<EmitMode> {
    match emit_str {
        "files" => Ok(EmitMode::Files),
        "stdout" => Ok(EmitMode::Stdout),
        _ => Err(format_err!("Invalid value for `--emit`")),
    }
}
