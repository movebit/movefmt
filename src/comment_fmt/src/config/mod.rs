use std::cell::Cell;
use std::default::Default;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::{env, fs};

use thiserror::Error;

use crate::config::config_type::ConfigType;
#[allow(unreachable_pub)]
pub use crate::config::options::*;

#[macro_use]
pub mod config_type;
#[macro_use]
#[allow(unreachable_pub)]
pub mod options;

// This macro defines configuration options used in movefmt. Each option
// is defined as follows:
//
// `name: value type, default value, is stable, description;`
create_config! {
    // Fundamental stuff
    max_width: usize, 100, true, "Maximum width of each line";
    hard_tabs: bool, false, true, "Use tab characters for indentation, spaces for alignment";
    tab_spaces: usize, 4, true, "Number of spaces per tab";
    newline_style: NewlineStyle, NewlineStyle::Auto, true, "Unix or Windows line endings";

    // Comments. macros, and strings
    wrap_comments: bool, false, false, "Break comments to fit on the line";
    format_code_in_doc_comments: bool, false, false, "Format the code snippet in doc comments.";
    doc_comment_code_block_width: usize, 100, false, "Maximum width for code snippets in doc \
        comments. No effect unless format_code_in_doc_comments = true";
    comment_width: usize, 80, false,
        "Maximum length of comments. No effect unless wrap_comments = true";
    normalize_comments: bool, false, false, "Convert /* */ comments to // comments where possible";
    normalize_doc_attributes: bool, false, false, "Normalize doc attributes as doc comments";
    format_strings: bool, false, false, "Format string literals where necessary";
    format_macro_matchers: bool, false, false,
        "Format the metavariable matching patterns in macros";
    format_macro_bodies: bool, true, false, "Format the bodies of macros";
    hex_literal_case: HexLiteralCase, HexLiteralCase::Preserve, false,
        "Format hexadecimal integer literals";

    // Single line expressions and items
    empty_item_single_line: bool, true, false,
        "Put empty-body functions and impls on a single line";
    struct_lit_single_line: bool, true, false,
        "Put small struct literals on a single line";
    fn_single_line: bool, false, false, "Put single-expression functions on a single line";
    where_single_line: bool, false, false, "Force where-clauses to be on a single line";

    // Imports
    imports_indent: IndentStyle, IndentStyle::Block, false, "Indent of imports";
    imports_granularity: ImportGranularity, ImportGranularity::Preserve, false,
        "Merge or split imports to the provided granularity";
    group_imports: GroupImportsTactic, GroupImportsTactic::Preserve, false,
        "Controls the strategy for how imports are grouped together";
    merge_imports: bool, false, false, "(deprecated: use imports_granularity instead)";

    // Ordering
    reorder_imports: bool, true, true, "Reorder import and extern crate statements alphabetically";
    reorder_modules: bool, true, true, "Reorder module statements alphabetically in group";
    reorder_impl_items: bool, false, false, "Reorder impl items";

    // Spaces around punctuation
    type_punctuation_density: TypeDensity, TypeDensity::Wide, false,
        "Determines if '+' or '=' are wrapped in spaces in the punctuation of types";
    space_before_colon: bool, false, false, "Leave a space before the colon";
    space_after_colon: bool, true, false, "Leave a space after the colon";
    spaces_around_ranges: bool, false, false, "Put spaces around the  .. and ..= range operators";

    // Misc.
    remove_nested_parens: bool, true, true, "Remove nested parens";
    combine_control_expr: bool, true, false, "Combine control expressions with function calls";
    short_array_element_width_threshold: usize, 10, true,
        "Width threshold for an array element to be considered short";
    overflow_delimited_expr: bool, false, false,
        "Allow trailing bracket/brace delimited expressions to overflow";
    struct_field_align_threshold: usize, 0, false,
        "Align struct fields if their diffs fits within threshold";
    enum_discrim_align_threshold: usize, 0, false,
        "Align enum variants discrims, if their diffs fit within threshold";
    match_arm_blocks: bool, true, false, "Wrap the body of arms in blocks when it does not fit on \
        the same line with the pattern of arms";
    match_arm_leading_pipes: MatchArmLeadingPipe, MatchArmLeadingPipe::Never, true,
        "Determines whether leading pipes are emitted on match arms";
    force_multiline_blocks: bool, false, false,
        "Force multiline closure bodies and match arms to be wrapped in a block";
    fn_args_layout: Density, Density::Tall, true,
        "(deprecated: use fn_params_layout instead)";
    fn_params_layout: Density, Density::Tall, true,
        "Control the layout of parameters in function signatures.";
    brace_style: BraceStyle, BraceStyle::SameLineWhere, false, "Brace style for items";
    control_brace_style: ControlBraceStyle, ControlBraceStyle::AlwaysSameLine, false,
        "Brace style for control flow constructs";
    trailing_semicolon: bool, true, false,
        "Add trailing semicolon after break, continue and return";
    match_block_trailing_comma: bool, false, true,
        "Put a trailing comma after a block based match arm (non-block arms are not affected)";
    blank_lines_upper_bound: usize, 1, false,
        "Maximum number of blank lines which can be put between items";
    blank_lines_lower_bound: usize, 0, false,
        "Minimum number of blank lines which must be put between items";
    edition: Edition, Edition::Edition2015, true, "The edition of the parser (RFC 2052)";
    version: Version, Version::One, false, "Version of formatting rules";
    inline_attribute_width: usize, 0, false,
        "Write an item and its attribute on the same line \
        if their combined width is below a threshold";
    format_generated_files: bool, true, false, "Format generated files";

    // Options that can change the source code beyond whitespace/blocks (somewhat linty things)
    merge_derives: bool, true, true, "Merge multiple `#[derive(...)]` into a single one";
    use_try_shorthand: bool, false, true, "Replace uses of the try! macro by the ? shorthand";
    use_field_init_shorthand: bool, false, true, "Use field initialization shorthand if possible";
    force_explicit_abi: bool, true, true, "Always print the abi for extern items";
    condense_wildcard_suffixes: bool, false, false, "Replace strings of _ wildcards by a single .. \
                                                     in tuple patterns";

    // Control options (changes the operation of movefmt, rather than the formatting)
    color: Color, Color::Auto, false,
        "What Color option to use when none is supplied: Always, Never, Auto";
    required_version: String, env!("CARGO_PKG_VERSION").to_owned(), false,
        "Require a specific version of movefmt";
    unstable_features: bool, false, false,
            "Enables unstable features. Only available on nightly channel";
    disable_all_formatting: bool, false, true, "Don't reformat anything";
    skip_children: bool, false, false, "Don't reformat out of line modules";
    hide_parse_errors: bool, false, false, "Hide errors from the parser";
    error_on_line_overflow: bool, false, false, "Error if unable to get all lines within max_width";
    error_on_unformatted: bool, false, false,
        "Error if unable to get comments or string literals within max_width, \
         or they are left with trailing whitespaces";
    ignore: IgnoreList, IgnoreList::default(), false,
        "Skip formatting the specified files and directories";

    // Not user-facing
    verbose: Verbosity, Verbosity::Normal, false, "How much to information to emit to the user";
    emit_mode: EmitMode, EmitMode::Files, false,
        "What emit Mode to use when none is supplied";
    make_backup: bool, false, false, "Backup changed files";
    print_misformatted_file_names: bool, false, true,
        "Prints the names of mismatched files that were formatted. Prints the names of \
         files that would be formatted when used with `--check` mode. ";
}

#[derive(Error, Debug)]
#[error("Could not output config: {0}")]
pub struct ToTomlError(toml::ser::Error);

impl PartialConfig {
    pub fn to_toml(&self) -> Result<String, ToTomlError> {
        // Non-user-facing options can't be specified in TOML
        let mut cloned = self.clone();
        cloned.verbose = None;
        cloned.print_misformatted_file_names = None;
        cloned.merge_imports = None;
        cloned.fn_args_layout = None;

        ::toml::to_string(&cloned).map_err(ToTomlError)
    }
}

impl Config {
    /// Constructs a `Config` from the toml file specified at `file_path`.
    ///
    /// This method only looks at the provided path, for a method that
    /// searches parents for a `movefmt.toml` see `from_resolved_toml_path`.
    ///
    /// Returns a `Config` if the config could be read and parsed from
    /// the file, otherwise errors.
    pub(super) fn from_toml_path(file_path: &Path) -> Result<Config, Error> {
        let mut file = File::open(&file_path)?;
        let mut toml = String::new();
        file.read_to_string(&mut toml)?;
        Config::from_toml(&toml, file_path.parent().unwrap())
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }

    /// Resolves the config for input in `dir`.
    ///
    /// Searches for `movefmt.toml` beginning with `dir`, and
    /// recursively checking parents of `dir` if no config file is found.
    /// If no config file exists in `dir` or in any parent, a
    /// default `Config` will be returned (and the returned path will be empty).
    ///
    /// Returns the `Config` to use, and the path of the project file if there was
    /// one.
    pub(super) fn from_resolved_toml_path(dir: &Path) -> Result<(Config, Option<PathBuf>), Error> {
        /// Try to find a project file in the given directory and its parents.
        /// Returns the path of a the nearest project file if one exists,
        /// or `None` if no project file was found.
        fn resolve_project_file(dir: &Path) -> Result<Option<PathBuf>, Error> {
            let mut current = if dir.is_relative() {
                env::current_dir()?.join(dir)
            } else {
                dir.to_path_buf()
            };

            current = fs::canonicalize(current)?;

            loop {
                match get_toml_path(&current) {
                    Ok(Some(path)) => return Ok(Some(path)),
                    Err(e) => return Err(e),
                    _ => (),
                }

                // If the current directory has no parent, we're done searching.
                if !current.pop() {
                    break;
                }
            }

            // If nothing was found, check in the home directory.
            if let Some(home_dir) = dirs::home_dir() {
                if let Some(path) = get_toml_path(&home_dir)? {
                    return Ok(Some(path));
                }
            }

            // If none was found ther either, check in the user's configuration directory.
            if let Some(mut config_dir) = dirs::config_dir() {
                config_dir.push("movefmt");
                if let Some(path) = get_toml_path(&config_dir)? {
                    return Ok(Some(path));
                }
            }

            Ok(None)
        }

        match resolve_project_file(dir)? {
            None => Ok((Config::default(), None)),
            Some(path) => Config::from_toml_path(&path).map(|config| (config, Some(path))),
        }
    }

    pub fn from_toml(toml: &str, dir: &Path) -> Result<Config, String> {
        let parsed: ::toml::Value = toml
            .parse()
            .map_err(|e| format!("Could not parse TOML: {}", e))?;
        let mut err = String::new();
        let table = parsed
            .as_table()
            .ok_or_else(|| String::from("Parsed config was not table"))?;
        for key in table.keys() {
            if !Config::is_valid_name(key) {
                let msg = &format!("Warning: Unknown configuration option `{key}`\n");
                err.push_str(msg)
            }
        }
        match parsed.try_into() {
            Ok(parsed_config) => {
                if !err.is_empty() {
                    eprint!("{err}");
                }
                Ok(Config::default().fill_from_parsed_config(parsed_config, dir))
            }
            Err(e) => {
                err.push_str("Error: Decoding config file failed:\n");
                err.push_str(format!("{e}\n").as_str());
                err.push_str("Please check your config file.");
                Err(err)
            }
        }
    }
}

/// Loads a config by checking the client-supplied options and if appropriate, the
/// file system (including searching the file system for overrides).
pub fn load_config<O: CliOptions>(
    file_path: Option<&Path>,
    options: Option<O>,
) -> Result<(Config, Option<PathBuf>), Error> {
    let over_ride = match options {
        Some(ref opts) => config_path(opts)?,
        None => None,
    };

    let result = if let Some(over_ride) = over_ride {
        Config::from_toml_path(over_ride.as_ref()).map(|p| (p, Some(over_ride.to_owned())))
    } else if let Some(file_path) = file_path {
        Config::from_resolved_toml_path(file_path)
    } else {
        Ok((Config::default(), None))
    };

    result.map(|(mut c, p)| {
        if let Some(options) = options {
            options.apply_to(&mut c);
        }
        (c, p)
    })
}

// Check for the presence of known config file names (`movefmt.toml, `.movefmt.toml`) in `dir`
//
// Return the path if a config file exists, empty if no file exists, and Error for IO errors
fn get_toml_path(dir: &Path) -> Result<Option<PathBuf>, Error> {
    const CONFIG_FILE_NAMES: [&str; 2] = [".movefmt.toml", "movefmt.toml"];
    for config_file_name in &CONFIG_FILE_NAMES {
        let config_file = dir.join(config_file_name);
        match fs::metadata(&config_file) {
            // Only return if it's a file to handle the unlikely situation of a directory named
            // `movefmt.toml`.
            Ok(ref md) if md.is_file() => return Ok(Some(config_file)),
            // Return the error if it's something other than `NotFound`; otherwise we didn't
            // find the project file yet, and continue searching.
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    let ctx = format!("Failed to get metadata for config file {:?}", &config_file);
                    let err = anyhow::Error::new(e).context(ctx);
                    return Err(Error::new(ErrorKind::Other, err));
                }
            }
            _ => {}
        }
    }
    Ok(None)
}

fn config_path(options: &dyn CliOptions) -> Result<Option<PathBuf>, Error> {
    let config_path_not_found = |path: &str| -> Result<Option<PathBuf>, Error> {
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Error: unable to find a config file for the given path: `{}`",
                path
            ),
        ))
    };

    // Read the config_path and convert to parent dir if a file is provided.
    // If a config file cannot be found from the given path, return error.
    match options.config_path() {
        Some(path) if !path.exists() => config_path_not_found(path.to_str().unwrap()),
        Some(path) if path.is_dir() => {
            let config_file_path = get_toml_path(path)?;
            if config_file_path.is_some() {
                Ok(config_file_path)
            } else {
                config_path_not_found(path.to_str().unwrap())
            }
        }
        path => Ok(path.map(ToOwned::to_owned)),
    }
}
