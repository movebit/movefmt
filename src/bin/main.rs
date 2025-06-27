// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{format_err, Result};
use commentfmt::{load_config, CliOptions, Config, EmitMode, Verbosity};
use getopts::{Matches, Options};
use io::Error as IoError;
use movefmt::{
    core::fmt::format_entry,
    tools::movefmt_diff::{make_diff, print_mismatches_default_message, DIFF_CONTEXT_SIZE},
    tools::utils::*,
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::result::Result::Ok;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

extern crate colored;
use colored::Colorize;

const ERR_EMPTY_INPUT_FROM_STDIN: i32 = 1;
const ERR_INVALID_MOVE_CODE_FROM_STDIN: i32 = 2;
const ERR_FMT: i32 = 3;
const ERR_WRITE: i32 = 4;
const ENABLE_THREAD: bool = false;

#[derive(Error, Debug)]
enum MoveFmtError {
    #[error("Format failed by Stdin with exit code: {0}")]
    ErrStdin(i32),

    #[error("Format failed with exit code: {0}")]
    ErrFmt(i32),

    #[error("Failed to write file")]
    ErrWrite(i32),
}

struct WriteResult {
    path: PathBuf,
    result: std::io::Result<()>,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("MOVEFMT_LOG").unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
    let opts = make_opts();

    let exit_code = match execute(&opts) {
        Ok(code) => code,
        Err(e) => {
            tracing::error!("{e:#}");
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
    /// Format files and their child modules. The bool value indicates whether
    /// the file is aspecified Move file from command line, which should not be escaped.
    Format { files: Vec<(PathBuf, bool)> },
    /// Print the help message.
    Help(HelpOp),
    /// Print version information
    Version,
    /// Output default config to a file, or stdout if None
    ConfigOutputDefault { path: Option<String> },
    /// Output current config (as if formatting to a file) to stdout
    ConfigOutputCurrent { path: Option<String> },
    /// No file specified, read from stdin
    Stdin { exit_code: i32 },
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
    /// An io error during reading or writing.
    #[error("{0}")]
    IoError(IoError),
    /// An error during escape check.
    #[error("{0}")]
    EscapeError(String),
}

/// formatting errors.
#[derive(Error, Debug)]
pub enum FormattingError {
    #[error("{0}")]
    ParseContentError(String),

    #[error("{0}")]
    FmtError(String),
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
    let emit_opts = "[overwrite|new_file|stdout|diff]";

    opts.optopt("", "emit", "What data to emit and how", emit_opts);
    opts.optopt(
        "",
        "config-path",
        "Recursively searches the given path for the movefmt.toml config file",
        "[Path for the configuration file]",
    );
    opts.optopt(
        "",
        "print-config",
        "Dumps a default or current config to PATH(eg: movefmt.toml)",
        "[default|current] PATH",
    );
    opts.optmulti(
        "",
        "config",
        "Set options from command line. These settings take priority over .movefmt.toml",
        "[key1=val1,key2=val2...]",
    );
    opts.optopt(
        "",
        "file-path",
        "Format the full path of the specified Move file",
        "[Absolute path of the specified Move file]",
    );
    opts.optopt(
        "",
        "dir-path",
        "Format all Move files in the specified directory",
        "[Absolute path of specified directory]",
    );
    opts.optflag("v", "verbose", "Print verbose output");
    opts.optflag("q", "quiet", "Print less output");
    opts.optflag("V", "version", "Show version information");
    let help_topic_msg = "Show help".to_owned();
    opts.optflagopt("h", "help", &help_topic_msg, "=TOPIC");
    opts.optflag("i", "stdin", "Receive code text from stdin");

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
        Operation::ConfigOutputDefault { path } => {
            let toml = Config::default().all_options().to_toml()?;
            if let Some(path) = path {
                let mut file = File::create(path)?;
                file.write_all(toml.as_bytes())?;
            } else {
                io::stdout().write_all(toml.as_bytes())?;
            }
            Ok(0)
        }
        Operation::ConfigOutputCurrent { path } => {
            let path = match path {
                Some(path) => path,
                None => return Err(format_err!("PATH required for `--print-config current`")),
            };

            let file = PathBuf::from(path);
            let file = file.canonicalize().unwrap_or(file);

            let (config, _) = load_config(Some(file.parent().unwrap()), Some(options))?;
            let toml = config.all_options().to_toml()?;
            io::stdout().write_all(toml.as_bytes())?;

            Ok(0)
        }
        Operation::Stdin { exit_code } => {
            if exit_code > 0 {
                Err(MoveFmtError::ErrStdin(exit_code).into())
            } else {
                Ok(0)
            }
        }
        Operation::Format { files } => format(files, &options),
    }
}

fn format_string(content_origin: String, options: GetOptsOptions) -> Result<i32> {
    let (config, config_path) = load_config(None, Some(options.clone()))?;
    let use_config = config.clone();
    if config.verbose() == Verbosity::Verbose {
        if let Some(path) = config_path.as_ref() {
            println!("Using movefmt config file {}", path.display());
        }
    }
    match format_entry(content_origin.clone(), use_config.clone()) {
        Ok(formatted_text) => {
            let emit_mode = if let Some(op_emit) = options.emit_mode {
                op_emit
            } else {
                use_config.emit_mode()
            };
            match emit_mode {
                EmitMode::Diff => {
                    let compare = make_diff(&content_origin, &formatted_text, DIFF_CONTEXT_SIZE);
                    if !compare.is_empty() {
                        let mut failures = HashMap::new();
                        failures.insert(PathBuf::new(), compare);
                        print_mismatches_default_message(failures);
                    }
                }
                _ => {
                    if options.quiet.is_none() || !options.quiet.unwrap() {
                        tracing::warn!(
                            "\n{}\n--------------------------------------------------------------------",
                            "The formatted result of the Move code read from stdin is as follows:".green()
                        );
                    }
                    println!("{}", formatted_text);
                }
            }
            Ok(0)
        }
        Err(_) => Err(FormattingError::ParseContentError("parse failed".to_string()).into()),
    }
}

fn format(files: Vec<(PathBuf, bool)>, options: &GetOptsOptions) -> Result<i32> {
    if options.quiet.is_none() || !options.quiet.unwrap() {
        println!("options = {:?}", options);
    }

    let (config, config_path) = load_config(None, Some(options.clone()))?;
    let mut use_config = config.clone();
    let mut use_config_path = config_path.clone();
    let mut success_cnt = 0;
    let mut skips_cnt_expected = 0;
    let mut skips_cnt_not_belong_to_any_package = 0;
    tracing::info!(
        "config.[verbose, indent] = [{:?}, {:?}], {:?}",
        config.verbose(),
        config.indent_size(),
        options
    );

    if config.verbose() == Verbosity::Verbose {
        if let Some(path) = config_path.as_ref() {
            println!("Using movefmt config file {}", path.display());
        }
    }

    let mut files = files;
    let files_len = files.len();
    let mut no_files_argument = files.is_empty();
    if no_files_argument {
        if let Ok(current_dir) = std::env::current_dir() {
            for x in walkdir::WalkDir::new(current_dir) {
                let x = match x {
                    Ok(x) => x,
                    Err(_) => {
                        break;
                    }
                };
                if x.file_type().is_file()
                    && x.file_name().to_str().unwrap().ends_with(".move")
                    && !x.file_name().to_str().unwrap().contains(".fmt")
                    && !x.file_name().to_str().unwrap().contains(".out")
                {
                    files.push((x.clone().into_path(), false));
                }
            }
        } else {
            return Err(format_err!(
                "Failed to get the current directory when file argument is not specified."
            ));
        }
    }

    let (pool, tx, rx) = get_item_for_mutil_thread(ENABLE_THREAD);

    for (file, is_specified_file) in files {
        if !file.exists() {
            eprintln!("Error: file `{}` does not exist", file.to_str().unwrap());
            continue;
        } else if file.is_dir() {
            eprintln!("Error: `{}` is a directory", file.to_str().unwrap());
            continue;
        } else {
            // Check the file directory if the config-path could not be read or not provided
            if config_path.is_none() {
                let (local_config, config_path) =
                    load_config(Some(file.parent().unwrap()), Some(options.clone()))?;
                tracing::debug!("local config_path = {:?}", config_path);

                if let Some(path) = config_path {
                    if local_config.verbose() == Verbosity::Verbose {
                        println!(
                            "Using movefmt local config file {} for {}",
                            path.display(),
                            file.display()
                        );
                    }
                    use_config = local_config.clone();
                    use_config_path = Some(path);
                }
            }
        }

        if !use_config.auto_apply_package()
            && no_files_argument
            && use_config.verbose() == Verbosity::Verbose
        {
            tracing::warn!("\n{}",
            "No file argument is supplied, movefmt runs on current directory by default, \nformatting all .move files within it......".yellow());
            println!(
                "----------------------------------------------------------------------------\n"
            );
            no_files_argument = false;
        }

        if !is_specified_file && should_escape_not_in_package(&file, &use_config) {
            skips_cnt_not_belong_to_any_package += 1;
            if use_config.verbose() == Verbosity::Verbose
                && (options.quiet.is_none() || !options.quiet.unwrap())
            {
                tracing::warn!(
                    "\n{}: {} {}\n",
                    "Escape file".yellow(),
                    file.display(),
                    "because it's not belong to any Move-Package".yellow()
                );
            }
            continue;
        }

        if !is_specified_file
            && should_escape(&file, &use_config, use_config_path.clone()).is_some()
        {
            skips_cnt_expected += 1;
            if use_config.verbose() == Verbosity::Verbose
                && (options.quiet.is_none() || !options.quiet.unwrap())
            {
                tracing::warn!(
                    "\n{}: {} {}: {}\n",
                    "Escape file".yellow(),
                    file.display(),
                    "by config".yellow(),
                    use_config_path.clone().unwrap_or_default().display()
                );
            }
            continue;
        }

        let content_origin = std::fs::read_to_string(file.as_path()).unwrap();
        if use_config.verbose() == Verbosity::Verbose {
            println!("Formatting {}", file.display());
        }
        match format_entry(content_origin.clone(), use_config.clone()) {
            Ok(formatted_text) => {
                success_cnt += 1;
                let emit_mode = if let Some(op_emit) = options.emit_mode {
                    op_emit
                } else {
                    use_config.emit_mode()
                };
                match emit_mode {
                    EmitMode::NewFile => {
                        let file_path = mk_result_filepath(&file.to_path_buf());
                        write_file(file_path, formatted_text, ENABLE_THREAD, &pool, &tx)?;
                    }
                    EmitMode::Overwrite => {
                        write_file(file, formatted_text, ENABLE_THREAD, &pool, &tx)?;
                    }
                    EmitMode::Stdout => {
                        println!("{}", formatted_text);
                    }
                    EmitMode::Diff => {
                        let compare =
                            make_diff(&content_origin, &formatted_text, DIFF_CONTEXT_SIZE);
                        if !compare.is_empty() {
                            let mut failures = HashMap::new();
                            failures.insert(file.to_owned(), compare);
                            print_mismatches_default_message(failures);

                            // Only for github CI
                            if std::env::var("CI").is_ok() {
                                return Err(format_err!(
                                    "CHECK FAILED, your source code has not been formatted with movefmt."
                                ));
                            }
                        }
                    }
                }
            }
            Err(diags) => {
                let mut files_source_text: move_compiler::diagnostics::FilesSourceText =
                    HashMap::new();
                files_source_text.insert(
                    move_command_line_common::files::FileHash::empty(),
                    (file.display().to_string().into(), content_origin.clone()),
                );
                let diags_buf = move_compiler::diagnostics::report_diagnostics_to_color_buffer(
                    &files_source_text,
                    diags,
                );
                if std::io::stdout().write_all(&diags_buf).is_err() {
                    // Cannot output compiler diagnostics;
                    // https://github.com/movebit/movefmt/issues/2
                    eprintln!("file '{:?}' skipped because of parse not ok", file);
                }
                return Err(MoveFmtError::ErrFmt(ERR_FMT).into());
            }
        }
    }

    if ENABLE_THREAD {
        let mut counter = 0;
        while counter != files_len {
            let WriteResult { path, result } = rx.as_ref().unwrap().recv().unwrap();
            if result.is_err() {
                eprintln!("Failed to write file({}): {:?}", path.display(), result);
                return Err(MoveFmtError::ErrWrite(ERR_WRITE).into());
            }
            counter += 1;
        }
    }

    if options.quiet.is_none() || !options.quiet.unwrap() {
        println!(
            "\n----------------------------------------------------------------------------\n"
        );
        if skips_cnt_expected > 0 {
            println!(
                "{:?} files skipped because escaped by movefmt.toml",
                skips_cnt_expected
            );
        }
        if skips_cnt_not_belong_to_any_package > 0 {
            println!(
                "{:?} files skipped because it's not belong to any Move-Package",
                skips_cnt_not_belong_to_any_package
            );
        }
        if success_cnt > 0 {
            println!("{:?} files successfully formatted\n", success_cnt);
        }
    }

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
    println!("movefmt v1.2.3");
}

fn determine_operation(matches: &Matches) -> Result<Operation, OperationError> {
    if matches.opt_present("h") {
        let topic = matches.opt_str("h");
        if topic.is_none() {
            return Ok(Operation::Help(HelpOp::None));
        } else if topic == Some("config".to_owned()) {
            return Ok(Operation::Help(HelpOp::Config));
        }
    }
    let mut free_matches = matches.free.iter();
    if let Some(kind) = matches.opt_str("print-config") {
        let path = free_matches.next().cloned();
        match kind.as_str() {
            "default" => return Ok(Operation::ConfigOutputDefault { path }),
            "current" => return Ok(Operation::ConfigOutputCurrent { path }),
            _ => {
                return Err(OperationError::UnknownPrintConfigTopic(kind));
            }
        }
    }

    if matches.opt_present("version") {
        return Ok(Operation::Version);
    }

    let mut files: Vec<_> = free_matches
        .map(|s| {
            let p = PathBuf::from(s);
            // we will do comparison later, so here tries to canonicalize first
            // to get the expected behavior.
            (p.canonicalize().unwrap_or(p), true)
        })
        .collect();

    if matches.opt_present("file-path") {
        if let Some(move_file_path) = matches.opt_str("file-path") {
            files.push((PathBuf::from(move_file_path), true));
        }
    }

    if matches.opt_present("dir-path") {
        if let Some(move_dir_path) = matches.opt_str("dir-path") {
            for x in walkdir::WalkDir::new(PathBuf::from(move_dir_path)) {
                let x = match x {
                    Ok(x) => x,
                    Err(_) => {
                        break;
                    }
                };
                if x.file_type().is_file()
                    && x.file_name().to_str().unwrap().ends_with(".move")
                    && !x.file_name().to_str().unwrap().contains(".fmt")
                    && !x.file_name().to_str().unwrap().contains(".out")
                {
                    files.push((x.clone().into_path(), false));
                }
            }
        }
    }

    if matches.opt_present("stdin") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let options = GetOptsOptions::from_matches(&matches).unwrap_or_default();
        if buffer.is_empty() {
            tracing::warn!(
                "\n{}",
                "You haven't entered any Move code. Please run movefmt again.".yellow()
            );
            return Ok(Operation::Stdin {
                exit_code: ERR_EMPTY_INPUT_FROM_STDIN,
            });
        } else if let Ok(_) = format_string(buffer, options) {
            return Ok(Operation::Stdin { exit_code: 0 });
        } else {
            tracing::error!(
                "{}, please re-enter a valid move code",
                "Format Failed on stdin's buffer".red()
            );
            return Ok(Operation::Stdin {
                exit_code: ERR_INVALID_MOVE_CODE_FROM_STDIN,
            });
        }
    }

    Ok(Operation::Format { files })
}

/// Parsed command line options.
#[derive(Clone, Debug, Default)]
struct GetOptsOptions {
    quiet: Option<bool>,
    verbose: Option<bool>,
    config_path: Option<PathBuf>,
    emit_mode: Option<EmitMode>,
    inline_config: HashMap<String, String>,
}

impl GetOptsOptions {
    pub fn from_matches(matches: &Matches) -> Result<GetOptsOptions> {
        let mut options = GetOptsOptions {
            quiet: if matches.opt_present("quiet") {
                Some(true)
            } else {
                None
            },
            verbose: if matches.opt_present("verbose") {
                Some(true)
            } else {
                None
            },
            config_path: matches.opt_str("config-path").map(PathBuf::from),
            ..Default::default()
        };
        if options.verbose.is_some() && options.quiet.is_some() {
            return Err(format_err!("Can't use both `--verbose` and `--quiet`"));
        }

        if let Some(ref emit_str) = matches.opt_str("emit") {
            options.emit_mode = Some(emit_mode_from_emit_str(emit_str)?);
        }
        options.inline_config = matches
            .opt_strs("config")
            .iter()
            .flat_map(|config| config.split(','))
            .map(
                |key_val| match key_val.char_indices().find(|(_, ch)| *ch == '=') {
                    Some((middle, _)) => {
                        let (key, val) = (&key_val[..middle], &key_val[middle + 1..]);
                        if !Config::is_valid_key_val(key, val) {
                            Err(format_err!("invalid key=val pair: `{}`", key_val))
                        } else {
                            Ok((key.to_string(), val.to_string()))
                        }
                    }

                    None => Err(format_err!(
                        "--config expects comma-separated list of key=val pairs, found `{}`",
                        key_val
                    )),
                },
            )
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(options)
    }
}

impl CliOptions for GetOptsOptions {
    fn apply_to(self, config: &mut Config) {
        if self.verbose.is_some() && self.verbose.unwrap() {
            config.set().verbose(Verbosity::Verbose);
        } else if self.quiet.is_some() && self.quiet.unwrap() {
            config.set().verbose(Verbosity::Quiet);
        }

        if let Some(emit_mode) = self.emit_mode {
            config.set().emit_mode(emit_mode);
        }
        for (key, val) in self.inline_config {
            config.override_value(&key, &val);
        }
    }

    fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }
}

fn emit_mode_from_emit_str(emit_str: &str) -> Result<EmitMode> {
    match emit_str {
        "overwrite" => Ok(EmitMode::Overwrite),
        "new_file" => Ok(EmitMode::NewFile),
        "stdout" => Ok(EmitMode::Stdout),
        "diff" => Ok(EmitMode::Diff),
        _ => Err(format_err!("Invalid value for `--emit`")),
    }
}

fn should_escape(
    file: &Path,
    use_config: &Config,
    config_path: Option<PathBuf>,
) -> Option<PathBuf> {
    if config_path.is_none() {
        return None;
    }

    let escape = use_config
        .skip_formatting_dirs()
        .split(";")
        .filter(|s| !s.is_empty())
        .find_map(|x| {
            let mut p = PathBuf::from(x);
            if !p.is_absolute() && std::env::current_dir().is_ok() {
                p = std::env::current_dir().ok().unwrap_or_default().join(p);
            }

            if file.starts_with(&p) {
                Some(p)
            } else {
                None
            }
        });
    escape
}

fn should_escape_not_in_package(file: &Path, use_config: &Config) -> bool {
    if !use_config.auto_apply_package() {
        return false;
    }
    for ancestor in file.ancestors() {
        if let Some(dir_name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if matches!(dir_name, "sources" | "scripts" | "tests" | "examples") {
                if let Some(parent) = ancestor.parent() {
                    let toml_path = parent.join("Move.toml");
                    if toml_path.exists() {
                        if let Ok(toml_content) = std::fs::read_to_string(&toml_path) {
                            if toml_content.contains("https://github.com/MystenLabs/sui.git") {
                                return true;
                            }
                        }
                        return false;
                    } else {
                        return true;
                    }
                }
            }
        }
    }
    true
}

fn get_item_for_mutil_thread(
    is_enable_thread: bool,
) -> (
    Option<rayon::ThreadPool>,
    Option<std::sync::mpsc::Sender<WriteResult>>,
    Option<std::sync::mpsc::Receiver<WriteResult>>,
) {
    if is_enable_thread {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        // let pool = ThreadPool::new(4);
        let (tx, rx) = std::sync::mpsc::channel();
        (Some(pool), Some(tx), Some(rx))
    } else {
        (None, None, None)
    }
}

fn write_file(
    path: PathBuf,
    content: String,
    is_enable_thread: bool,
    pool: &Option<rayon::ThreadPool>,
    tx: &Option<std::sync::mpsc::Sender<WriteResult>>,
) -> Result<()> {
    if is_enable_thread {
        let tx = tx.clone();
        pool.as_ref().unwrap().spawn(move || {
            let write_result = std::fs::write(path.clone(), content);
            if write_result.is_err() {
                let _ = tx.as_ref().unwrap().send(WriteResult {
                    path: path,
                    result: write_result,
                });
            } else {
                let _ = tx.as_ref().unwrap().send(WriteResult {
                    path: path,
                    result: Ok(()),
                });
            }
        });
    } else {
        std::fs::write(&path, content)?;
    }
    Ok(())
}
