use crate::config::Config;
use config_proc_macro::config_type;
use std::path::Path;

/// What movefmt should emit. Mostly corresponds to the `--emit` command line
/// option.
#[config_type]
pub enum EmitMode {
    /// Overwrite the source file
    Overwrite,
    /// Emits to new files, eg: xxx.fmt.move.
    NewFile,
    /// Writes the output to stdout.
    Stdout,
    /// Checks if a diff can be generated. If so, movefmt outputs a diff and
    /// quits with exit code 1.
    /// This option is designed to be run in CI where a non-zero exit signifies
    /// non-standard code formatting. Used for `--check`.
    Diff,
}

/// How chatty should movefmt be?
#[config_type]
pub enum Verbosity {
    /// Emit more.
    Verbose,
    /// Default.
    Normal,
    /// Emit as little as possible.
    Quiet,
}

impl Default for EmitMode {
    fn default() -> EmitMode {
        EmitMode::Overwrite
    }
}

/// Maps client-supplied options to movefmt's internals, mostly overriding
/// values in a config with values from the command line.
pub trait CliOptions {
    fn apply_to(self, config: &mut Config);
    fn config_path(&self) -> Option<&Path>;
}
