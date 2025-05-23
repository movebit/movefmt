use crate::shape::{Indent, Shape};
use crate::string::{rewrite_string, StringFormat};
use crate::utils::{count_newlines, last_line_width, trim_left_preserve_layout, unicode_str_width};
use configurations::config::Config;
use itertools::{multipeek, MultiPeek};
use lazy_static::lazy_static;
use regex::Regex;
use std::{self, borrow::Cow, iter};

lazy_static! {
    /// A regex matching reference doc links.
    ///
    /// ```markdown
    /// /// An [example].
    /// ///
    /// /// [example]: this::is::a::link
    /// ```
    static ref REFERENCE_LINK_URL: Regex = Regex::new(r"^\[.+\]\s?:").unwrap();
}

fn is_custom_comment(comment: &str) -> bool {
    if !comment.starts_with("//") {
        false
    } else if let Some(c) = comment.chars().nth(2) {
        !c.is_alphanumeric() && !c.is_whitespace()
    } else {
        false
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CommentStyle<'a> {
    DoubleSlash,
    TripleSlash,
    Doc,
    SingleBullet,
    DoubleBullet,
    Exclamation,
    Custom(&'a str),
}

fn custom_opener(s: &str) -> &str {
    s.lines().next().map_or("", |first_line| {
        first_line
            .find(' ')
            .map_or(first_line, |space_index| &first_line[0..=space_index])
    })
}

impl<'a> CommentStyle<'a> {
    /// Returns `true` if the commenting style cannot span multiple lines.
    pub fn is_line_comment(&self) -> bool {
        matches!(
            self,
            CommentStyle::DoubleSlash
                | CommentStyle::TripleSlash
                | CommentStyle::Doc
                | CommentStyle::Custom(_)
        )
    }

    /// Returns `true` if the commenting style can span multiple lines.
    pub fn is_block_comment(&self) -> bool {
        matches!(
            self,
            CommentStyle::SingleBullet | CommentStyle::DoubleBullet | CommentStyle::Exclamation
        )
    }

    /// Returns `true` if the commenting style is for documentation.
    pub fn is_doc_comment(&self) -> bool {
        matches!(*self, CommentStyle::TripleSlash | CommentStyle::Doc)
    }

    pub fn opener(&self) -> &'a str {
        match *self {
            CommentStyle::DoubleSlash => "// ",
            CommentStyle::TripleSlash => "/// ",
            CommentStyle::Doc => "//! ",
            CommentStyle::SingleBullet => "/* ",
            CommentStyle::DoubleBullet => "/** ",
            CommentStyle::Exclamation => "/*! ",
            CommentStyle::Custom(opener) => opener,
        }
    }

    pub fn closer(&self) -> &'a str {
        match *self {
            CommentStyle::DoubleSlash
            | CommentStyle::TripleSlash
            | CommentStyle::Custom(..)
            | CommentStyle::Doc => "",
            CommentStyle::SingleBullet | CommentStyle::DoubleBullet | CommentStyle::Exclamation => {
                " */"
            }
        }
    }

    pub fn line_start(&self) -> &'a str {
        match *self {
            CommentStyle::DoubleSlash => "// ",
            CommentStyle::TripleSlash => "/// ",
            CommentStyle::Doc => "//! ",
            CommentStyle::SingleBullet | CommentStyle::DoubleBullet | CommentStyle::Exclamation => {
                " * "
            }
            CommentStyle::Custom(opener) => opener,
        }
    }

    pub fn to_str_tuplet(&self) -> (&'a str, &'a str, &'a str) {
        (self.opener(), self.closer(), self.line_start())
    }
}

pub fn comment_style(orig: &str, normalize_comments: bool) -> CommentStyle<'_> {
    if !normalize_comments {
        if orig.starts_with("/**") && !orig.starts_with("/**/") {
            CommentStyle::DoubleBullet
        } else if orig.starts_with("/*!") {
            CommentStyle::Exclamation
        } else if orig.starts_with("/*") {
            CommentStyle::SingleBullet
        } else if orig.starts_with("///") && orig.chars().nth(3).map_or(true, |c| c != '/') {
            CommentStyle::TripleSlash
        } else if orig.starts_with("//!") {
            CommentStyle::Doc
        } else if is_custom_comment(orig) {
            CommentStyle::Custom(custom_opener(orig))
        } else {
            CommentStyle::DoubleSlash
        }
    } else if (orig.starts_with("///") && orig.chars().nth(3).map_or(true, |c| c != '/'))
        || (orig.starts_with("/**") && !orig.starts_with("/**/"))
    {
        CommentStyle::TripleSlash
    } else if orig.starts_with("//!") || orig.starts_with("/*!") {
        CommentStyle::Doc
    } else if is_custom_comment(orig) {
        CommentStyle::Custom(custom_opener(orig))
    } else {
        CommentStyle::DoubleSlash
    }
}

pub fn rewrite_comment(
    orig: &str,
    block_style: bool,
    shape: Shape,
    config: &Config,
) -> Option<String> {
    identify_comment(orig, block_style, shape, config, false)
}

fn identify_comment(
    orig: &str,
    block_style: bool,
    shape: Shape,
    config: &Config,
    is_doc_comment: bool,
) -> Option<String> {
    let style = comment_style(orig, false);

    // Computes the byte length of line taking into account a newline if the line is part of a
    // paragraph.
    fn compute_len(orig: &str, line: &str) -> usize {
        if orig.len() > line.len() {
            if orig.as_bytes()[line.len()] == b'\r' {
                line.len() + 2
            } else {
                line.len() + 1
            }
        } else {
            line.len()
        }
    }

    // Get the first group of line comments having the same commenting style.
    //
    // Returns a tuple with:
    // - a boolean indicating if there is a blank line
    // - a number indicating the size of the first group of comments
    fn consume_same_line_comments(
        style: CommentStyle<'_>,
        orig: &str,
        line_start: &str,
    ) -> (bool, usize) {
        let mut first_group_ending = 0;
        let mut hbl = false;

        for line in orig.lines() {
            let trimmed_line = line.trim_start();
            if trimmed_line.is_empty() {
                hbl = true;
                break;
            } else if trimmed_line.starts_with(line_start)
                || comment_style(trimmed_line, false) == style
            {
                first_group_ending += compute_len(&orig[first_group_ending..], line);
            } else {
                break;
            }
        }
        (hbl, first_group_ending)
    }

    let (has_bare_lines, first_group_ending) = match style {
        CommentStyle::DoubleSlash | CommentStyle::TripleSlash | CommentStyle::Doc => {
            let line_start = style.line_start().trim_start();
            consume_same_line_comments(style, orig, line_start)
        }
        CommentStyle::Custom(opener) => {
            let trimmed_opener = opener.trim_end();
            consume_same_line_comments(style, orig, trimmed_opener)
        }

        CommentStyle::DoubleBullet | CommentStyle::SingleBullet | CommentStyle::Exclamation => {
            let closer = style.closer().trim_start();
            let mut count = orig.matches(closer).count();
            let mut closing_symbol_offset = 0;
            let mut hbl = false;
            let mut first = true;
            for line in orig.lines() {
                closing_symbol_offset += compute_len(&orig[closing_symbol_offset..], line);
                let mut trimmed_line = line.trim_start();
                if !trimmed_line.starts_with('*')
                    && !trimmed_line.starts_with("//")
                    && !trimmed_line.starts_with("/*")
                {
                    hbl = true;
                }

                // Remove opener from consideration when searching for closer
                if first {
                    let opener = style.opener().trim_end();
                    trimmed_line = &trimmed_line[opener.len()..];
                    first = false;
                }
                if trimmed_line.ends_with(closer) {
                    count -= 1;
                    if count == 0 {
                        break;
                    }
                }
            }
            (hbl, closing_symbol_offset)
        }
    };

    let (first_group, rest) = orig.split_at(first_group_ending);
    let rewritten_first_group = if has_bare_lines && style.is_block_comment() {
        trim_left_preserve_layout(first_group, shape.indent, config)?
    } else if !(
        // `format_code_in_doc_comments` should only take effect on doc comments,
        // so we only consider it when this comment block is a doc comment block.
        is_doc_comment
    ) {
        light_rewrite_comment(first_group, shape.indent, config, is_doc_comment)
    } else {
        rewrite_comment_inner(
            first_group,
            block_style,
            style,
            shape,
            config,
            is_doc_comment || style.is_doc_comment(),
        )?
    };
    if rest.is_empty() {
        Some(rewritten_first_group)
    } else {
        identify_comment(
            rest.trim_start(),
            block_style,
            shape,
            config,
            is_doc_comment,
        )
        .map(|rest_str| {
            let ret_cmt_str = format!(
                "{}\n{}{}{}",
                rewritten_first_group,
                // insert back the blank line
                if has_bare_lines && style.is_line_comment() {
                    "\n"
                } else {
                    ""
                },
                shape.indent.to_string(config),
                rest_str
            );
            tracing::info!("ret_cmt_str = {}", ret_cmt_str);
            ret_cmt_str
        })
    }
}

/// Enum indicating if the code block contains move based on attributes
enum CodeBlockAttribute {
    Move,
    NotMove,
}

impl CodeBlockAttribute {
    /// Parse comma separated attributes list. Return Move only if all
    /// attributes are valid Move attributes
    fn new(attributes: &str) -> CodeBlockAttribute {
        for attribute in attributes.split(',') {
            match attribute.trim() {
                "" | "run" => (),
                _ => return CodeBlockAttribute::NotMove,
            }
        }
        CodeBlockAttribute::Move
    }
}

struct CommentRewrite<'a> {
    result: String,
    code_block_buffer: String,
    is_prev_line_multi_line: bool,
    code_block_attr: Option<CodeBlockAttribute>,
    comment_line_separator: String,
    indent_str: String,
    max_width: usize,
    fmt_indent: Indent,
    fmt: StringFormat<'a>,

    opener: String,
    closer: String,
    line_start: String,
}

impl<'a> CommentRewrite<'a> {
    fn new(
        orig: &'a str,
        block_style: bool,
        shape: Shape,
        config: &'a Config,
    ) -> CommentRewrite<'a> {
        let ((opener, closer, line_start), _) = if block_style {
            (
                CommentStyle::SingleBullet.to_str_tuplet(),
                CommentStyle::SingleBullet,
            )
        } else {
            let style = comment_style(orig, false);
            (style.to_str_tuplet(), style)
        };

        let max_width = shape
            .width
            .checked_sub(closer.len() + opener.len())
            .unwrap_or(1);
        let indent_str = shape.indent.to_string_with_newline(config).to_string();

        let mut cr = CommentRewrite {
            result: String::with_capacity(orig.len() * 2),
            code_block_buffer: String::with_capacity(128),
            is_prev_line_multi_line: false,
            code_block_attr: None,
            comment_line_separator: format!("{indent_str}{line_start}"),
            max_width,
            indent_str,
            fmt_indent: shape.indent,

            fmt: StringFormat {
                opener: "",
                closer: "",
                line_start,
                line_end: "",
                shape: Shape::legacy(max_width, shape.indent),
                trim_end: true,
                config,
            },

            opener: opener.to_owned(),
            closer: closer.to_owned(),
            line_start: line_start.to_owned(),
        };
        cr.result.push_str(opener);
        cr
    }

    fn join_block(s: &str, sep: &str) -> String {
        let mut result = String::with_capacity(s.len() + 128);
        let mut iter = s.lines().peekable();
        while let Some(line) = iter.next() {
            result.push_str(line);
            result.push_str(match iter.peek() {
                Some(next_line) if next_line.is_empty() => sep.trim_end(),
                Some(..) => sep,
                None => "",
            });
        }
        result
    }

    fn finish(mut self) -> String {
        if !self.code_block_buffer.is_empty() {
            // There is a code block that is not properly enclosed by backticks.
            // We will leave them untouched.
            self.result.push_str(&self.comment_line_separator);
            self.result.push_str(&Self::join_block(
                &trim_custom_comment_prefix(&self.code_block_buffer),
                &self.comment_line_separator,
            ));
        }

        self.result.push_str(&self.closer);
        if self.result.ends_with(&self.opener) && self.opener.ends_with(' ') {
            // Trailing space.
            self.result.pop();
        }

        self.result
    }

    fn handle_line(
        &mut self,
        orig: &'a str,
        i: usize,
        line: &'a str,
        has_leading_whitespace: bool,
        is_doc_comment: bool,
    ) -> bool {
        let num_newlines = count_newlines(orig);
        let is_last = i == num_newlines;
        if self.code_block_attr.is_some() {
            self.code_block_buffer
                .push_str(&hide_sharp_behind_comment(line));
            self.code_block_buffer.push('\n');
            return false;
        }

        self.code_block_attr = None;
        if let Some(stripped) = line.strip_prefix("```") {
            self.code_block_attr = Some(CodeBlockAttribute::new(stripped))
        }

        if self.result == self.opener {
            let force_leading_whitespace = &self.opener == "/* " && count_newlines(orig) == 0;
            if !has_leading_whitespace && !force_leading_whitespace && self.result.ends_with(' ') {
                self.result.pop();
            }
            if line.is_empty() {
                return false;
            }
        } else if self.is_prev_line_multi_line && !line.is_empty() {
            self.result.push(' ')
        } else if is_last && line.is_empty() {
            // trailing blank lines are unwanted
            if !self.closer.is_empty() {
                self.result.push_str(&self.indent_str);
            }
            return true;
        } else {
            self.result.push_str(&self.comment_line_separator);
            if !has_leading_whitespace && self.result.ends_with(' ') {
                self.result.pop();
            }
        }

        let is_markdown_header_doc_comment = is_doc_comment && line.starts_with("#");

        // We only want to wrap the comment if:
        // 1) wrap_comments = true is configured
        // 2) The comment is not the start of a markdown header doc comment
        // 3) The comment width exceeds the shape's width
        // 4) No URLS were found in the comment
        // If this changes, the documentation in ../Configurations.md#wrap_comments
        // should be changed accordingly.
        let should_wrap_comment = !is_markdown_header_doc_comment
            && unicode_str_width(line) > self.fmt.shape.width
            && !has_url(line)
            && !is_table_item(line);

        if should_wrap_comment {
            match rewrite_string(line, &self.fmt, self.max_width) {
                Some(ref s) => {
                    self.is_prev_line_multi_line = s.contains('\n');
                    self.result.push_str(s);
                }
                None if self.is_prev_line_multi_line => {
                    // We failed to put the current `line` next to the previous `line`.
                    // Remove the trailing space, then start rewrite on the next line.
                    self.result.pop();
                    self.result.push_str(&self.comment_line_separator);
                    self.fmt.shape = Shape::legacy(self.max_width, self.fmt_indent);
                    match rewrite_string(line, &self.fmt, self.max_width) {
                        Some(ref s) => {
                            self.is_prev_line_multi_line = s.contains('\n');
                            self.result.push_str(s);
                        }
                        None => {
                            self.is_prev_line_multi_line = false;
                            self.result.push_str(line);
                        }
                    }
                }
                None => {
                    self.is_prev_line_multi_line = false;
                    self.result.push_str(line);
                }
            }

            self.fmt.shape = if self.is_prev_line_multi_line {
                // 1 = " "
                let offset = 1 + last_line_width(&self.result) - self.line_start.len();
                Shape {
                    width: self.max_width.saturating_sub(offset),
                    indent: self.fmt_indent,
                    offset: self.fmt.shape.offset + offset,
                }
            } else {
                Shape::legacy(self.max_width, self.fmt_indent)
            };
        } else {
            if line.is_empty() && self.result.ends_with(' ') && !is_last {
                // Remove space if this is an empty comment or a doc comment.
                self.result.pop();
            }
            self.result.push_str(line);
            self.fmt.shape = Shape::legacy(self.max_width, self.fmt_indent);
            self.is_prev_line_multi_line = false;
        }

        false
    }
}

fn rewrite_comment_inner(
    orig: &str,
    block_style: bool,
    style: CommentStyle<'_>,
    shape: Shape,
    config: &Config,
    is_doc_comment: bool,
) -> Option<String> {
    let mut rewriter = CommentRewrite::new(orig, block_style, shape, config);

    let line_breaks = count_newlines(orig.trim_end());
    let lines = orig
        .lines()
        .enumerate()
        .map(|(i, mut line)| {
            line = trim_end_unless_two_whitespaces(line.trim_start(), is_doc_comment);
            // Drop old closer.
            if i == line_breaks && line.ends_with("*/") && !line.starts_with("//") {
                line = line[..(line.len() - 2)].trim_end();
            }

            line
        })
        .map(|s| left_trim_comment_line(s, &style))
        .map(|(line, has_leading_whitespace)| {
            if orig.starts_with("/*") && line_breaks == 0 {
                (line.trim_start(), has_leading_whitespace)
            } else {
                (line, has_leading_whitespace)
            }
        });

    for (i, (line, has_leading_whitespace)) in lines.enumerate() {
        if rewriter.handle_line(orig, i, line, has_leading_whitespace, is_doc_comment) {
            break;
        }
    }

    Some(rewriter.finish())
}

const MOVEFMT_CUSTOM_COMMENT_PREFIX: &str = "//#### ";

fn hide_sharp_behind_comment(s: &str) -> Cow<'_, str> {
    let s_trimmed = s.trim();
    if s_trimmed.starts_with("# ") || s_trimmed == "#" {
        Cow::from(format!("{MOVEFMT_CUSTOM_COMMENT_PREFIX}{s}"))
    } else {
        Cow::from(s)
    }
}

fn trim_custom_comment_prefix(s: &str) -> String {
    s.lines()
        .map(|line| {
            let left_trimmed = line.trim_start();
            if left_trimmed.starts_with(MOVEFMT_CUSTOM_COMMENT_PREFIX) {
                left_trimmed.trim_start_matches(MOVEFMT_CUSTOM_COMMENT_PREFIX)
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns `true` if the given string MAY include URLs or alike.
fn has_url(s: &str) -> bool {
    // This function may return false positive, but should get its job done in most cases.
    s.contains("https://")
        || s.contains("http://")
        || s.contains("ftp://")
        || s.contains("file://")
        || REFERENCE_LINK_URL.is_match(s)
}

/// Returns true if the given string may be part of a Markdown table.
fn is_table_item(mut s: &str) -> bool {
    // This function may return false positive, but should get its job done in most cases (i.e.
    // markdown tables with two column delimiters).
    s = s.trim_start();
    return s.starts_with('|')
        && match s.rfind('|') {
            Some(0) | None => false,
            _ => true,
        };
}

/// Trim trailing whitespaces unless they consist of two or more whitespaces.
fn trim_end_unless_two_whitespaces(s: &str, is_doc_comment: bool) -> &str {
    if is_doc_comment && s.ends_with("  ") {
        s
    } else {
        s.trim_end()
    }
}

/// Trims whitespace and aligns to indent, but otherwise does not change comments.
fn light_rewrite_comment(
    orig: &str,
    offset: Indent,
    config: &Config,
    is_doc_comment: bool,
) -> String {
    let lines: Vec<&str> = orig
        .lines()
        .map(|l| {
            // This is basically just l.trim(), but in the case that a line starts
            // with `*` we want to leave one space before it, so it aligns with the
            // `*` in `/*`.
            let first_non_whitespace = l.find(|c| !char::is_whitespace(c));
            let left_trimmed = if let Some(fnw) = first_non_whitespace {
                // if l.as_bytes()[fnw] == b'*' && fnw > 0 {
                //     &l[fnw - 1..]
                // } else {
                //     &l[fnw..]
                // }
                &l[fnw..]
            } else {
                ""
            };
            // Preserve markdown's double-space line break syntax in doc comment.
            trim_end_unless_two_whitespaces(left_trimmed, is_doc_comment)
        })
        .collect();
    lines.join(&format!("\n{}", offset.to_string(config)))
}

/// Trims comment characters and possibly a single space from the left of a string.
/// Does not trim all whitespace. If a single space is trimmed from the left of the string,
/// this function returns true.
fn left_trim_comment_line<'a>(line: &'a str, style: &CommentStyle<'_>) -> (&'a str, bool) {
    if line.starts_with("//! ")
        || line.starts_with("/// ")
        || line.starts_with("/*! ")
        || line.starts_with("/** ")
    {
        (&line[4..], true)
    } else if let CommentStyle::Custom(opener) = *style {
        if let Some(stripped) = line.strip_prefix(opener) {
            (stripped, true)
        } else {
            (&line[opener.trim_end().len()..], false)
        }
    } else if line.starts_with("/* ")
        || line.starts_with("// ")
        || line.starts_with("//!")
        || line.starts_with("///")
        || line.starts_with("** ")
        || line.starts_with("/*!")
        || (line.starts_with("/**") && !line.starts_with("/**/"))
    {
        (&line[3..], line.chars().nth(2).unwrap() == ' ')
    } else if line.starts_with("/*")
        || line.starts_with("* ")
        || line.starts_with("//")
        || line.starts_with("**")
    {
        (&line[2..], line.chars().nth(1).unwrap() == ' ')
    } else if let Some(stripped) = line.strip_prefix('*') {
        (stripped, false)
    } else {
        (line, line.starts_with(' '))
    }
}

pub trait FindUncommented {
    fn find_uncommented(&self, pat: &str) -> Option<usize>;
    fn find_last_uncommented(&self, pat: &str) -> Option<usize>;
}

impl FindUncommented for str {
    fn find_uncommented(&self, pat: &str) -> Option<usize> {
        let mut needle_iter = pat.chars();
        for (kind, (i, b)) in CharClasses::new(self.char_indices()) {
            match needle_iter.next() {
                None => {
                    return Some(i - pat.len());
                }
                Some(c) => match kind {
                    FullCodeCharKind::Normal | FullCodeCharKind::InString if b == c => {}
                    _ => {
                        needle_iter = pat.chars();
                    }
                },
            }
        }

        // Handle case where the pattern is a suffix of the search string
        match needle_iter.next() {
            Some(_) => None,
            None => Some(self.len() - pat.len()),
        }
    }

    fn find_last_uncommented(&self, pat: &str) -> Option<usize> {
        if let Some(left) = self.find_uncommented(pat) {
            let mut result = left;
            // add 1 to use find_last_uncommented for &str after pat
            while let Some(next) = self[(result + 1)..].find_last_uncommented(pat) {
                result += next + 1;
            }
            Some(result)
        } else {
            None
        }
    }
}

// Returns the first byte position after the first comment. The given string
// is expected to be prefixed by a comment, including delimiters.
// Good: `/* /* inner */ outer */ code();`
// Bad:  `code(); // hello\n world!`
pub fn find_comment_end(s: &str) -> Option<usize> {
    let mut iter = CharClasses::new(s.char_indices());
    for (kind, (i, _c)) in &mut iter {
        if kind == FullCodeCharKind::Normal || kind == FullCodeCharKind::InString {
            return Some(i);
        }
    }

    // Handle case where the comment ends at the end of `s`.
    if iter.status == CharClassesStatus::Normal {
        Some(s.len())
    } else {
        None
    }
}

/// Returns `true` if text contains any comment.
pub fn contains_comment(text: &str) -> bool {
    CharClasses::new(text.chars()).any(|(kind, _)| kind.is_comment())
}

pub struct CharClasses<T>
where
    T: Iterator,
    T::Item: RichChar,
{
    base: MultiPeek<T>,
    status: CharClassesStatus,
}

pub trait RichChar {
    fn get_char(&self) -> char;
}

impl RichChar for char {
    fn get_char(&self) -> char {
        *self
    }
}

impl RichChar for (usize, char) {
    fn get_char(&self) -> char {
        self.1
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum CharClassesStatus {
    Normal,
    /// Character is within a string
    LitString,
    LitStringEscape,
    /// Character is within a raw string
    LitRawString(u32),
    RawStringPrefix(u32),
    RawStringSuffix(u32),
    LitChar,
    LitCharEscape,
    /// Character inside a block comment, with the integer indicating the nesting deepness of the
    /// comment
    BlockComment(u32),
    /// Character inside a block-commented string, with the integer indicating the nesting deepness
    /// of the comment
    StringInBlockComment(u32),
    /// Status when the '/' has been consumed, but not yet the '*', deepness is
    /// the new deepness (after the comment opening).
    BlockCommentOpening(u32),
    /// Status when the '*' has been consumed, but not yet the '/', deepness is
    /// the new deepness (after the comment closing).
    BlockCommentClosing(u32),
    /// Character is within a line comment
    LineComment,
}

/// Distinguish between functional part of code and comments
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CodeCharKind {
    Normal,
    Comment,
}

/// Distinguish between functional part of code and comments,
/// describing opening and closing of comments for ease when chunking
/// code from tagged characters
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FullCodeCharKind {
    Normal,
    /// The first character of a comment, there is only one for a comment (always '/')
    StartComment,
    /// Any character inside a comment including the second character of comment
    /// marks ("//", "/*")
    InComment,
    /// Last character of a comment, '\n' for a line comment, '/' for a block comment.
    EndComment,
    /// Start of a mutlitine string inside a comment
    StartStringCommented,
    /// End of a mutlitine string inside a comment
    EndStringCommented,
    /// Inside a commented string
    InStringCommented,
    /// Start of a mutlitine string
    StartString,
    /// End of a mutlitine string
    EndString,
    /// Inside a string.
    InString,
}

impl FullCodeCharKind {
    pub fn is_comment(self) -> bool {
        match self {
            FullCodeCharKind::StartComment
            | FullCodeCharKind::InComment
            | FullCodeCharKind::EndComment
            | FullCodeCharKind::StartStringCommented
            | FullCodeCharKind::InStringCommented
            | FullCodeCharKind::EndStringCommented => true,
            _ => false,
        }
    }

    /// Returns true if the character is inside a comment
    pub fn inside_comment(self) -> bool {
        match self {
            FullCodeCharKind::InComment
            | FullCodeCharKind::StartStringCommented
            | FullCodeCharKind::InStringCommented
            | FullCodeCharKind::EndStringCommented => true,
            _ => false,
        }
    }

    fn to_codecharkind(self) -> CodeCharKind {
        if self.is_comment() {
            CodeCharKind::Comment
        } else {
            CodeCharKind::Normal
        }
    }
}

impl<T> CharClasses<T>
where
    T: Iterator,
    T::Item: RichChar,
{
    pub fn new(base: T) -> CharClasses<T> {
        CharClasses {
            base: multipeek(base),
            status: CharClassesStatus::Normal,
        }
    }
}

fn is_raw_string_suffix<T>(iter: &mut MultiPeek<T>, count: u32) -> bool
where
    T: Iterator,
    T::Item: RichChar,
{
    for _ in 0..count {
        match iter.peek() {
            Some(c) if c.get_char() == '#' => continue,
            _ => return false,
        }
    }
    true
}

impl<T> Iterator for CharClasses<T>
where
    T: Iterator,
    T::Item: RichChar,
{
    type Item = (FullCodeCharKind, T::Item);

    fn next(&mut self) -> Option<(FullCodeCharKind, T::Item)> {
        let item = self.base.next()?;
        let chr = item.get_char();
        let mut char_kind = FullCodeCharKind::Normal;
        self.status = match self.status {
            CharClassesStatus::LitRawString(sharps) => {
                char_kind = FullCodeCharKind::InString;
                match chr {
                    '"' => {
                        if sharps == 0 {
                            char_kind = FullCodeCharKind::Normal;
                            CharClassesStatus::Normal
                        } else if is_raw_string_suffix(&mut self.base, sharps) {
                            CharClassesStatus::RawStringSuffix(sharps)
                        } else {
                            CharClassesStatus::LitRawString(sharps)
                        }
                    }
                    _ => CharClassesStatus::LitRawString(sharps),
                }
            }
            CharClassesStatus::RawStringPrefix(sharps) => {
                char_kind = FullCodeCharKind::InString;
                match chr {
                    '#' => CharClassesStatus::RawStringPrefix(sharps + 1),
                    '"' => CharClassesStatus::LitRawString(sharps),
                    _ => CharClassesStatus::Normal, // Unreachable.
                }
            }
            CharClassesStatus::RawStringSuffix(sharps) => {
                match chr {
                    '#' => {
                        if sharps == 1 {
                            CharClassesStatus::Normal
                        } else {
                            char_kind = FullCodeCharKind::InString;
                            CharClassesStatus::RawStringSuffix(sharps - 1)
                        }
                    }
                    _ => CharClassesStatus::Normal, // Unreachable
                }
            }
            CharClassesStatus::LitString => {
                char_kind = FullCodeCharKind::InString;
                match chr {
                    '"' => CharClassesStatus::Normal,
                    '\\' => CharClassesStatus::LitStringEscape,
                    _ => CharClassesStatus::LitString,
                }
            }
            CharClassesStatus::LitStringEscape => {
                char_kind = FullCodeCharKind::InString;
                CharClassesStatus::LitString
            }
            CharClassesStatus::LitChar => match chr {
                '\\' => CharClassesStatus::LitCharEscape,
                '\'' => CharClassesStatus::Normal,
                _ => CharClassesStatus::LitChar,
            },
            CharClassesStatus::LitCharEscape => CharClassesStatus::LitChar,
            CharClassesStatus::Normal => match chr {
                'r' => match self.base.peek().map(RichChar::get_char) {
                    Some('#') | Some('"') => {
                        char_kind = FullCodeCharKind::InString;
                        CharClassesStatus::RawStringPrefix(0)
                    }
                    _ => CharClassesStatus::Normal,
                },
                '"' => {
                    char_kind = FullCodeCharKind::InString;
                    CharClassesStatus::LitString
                }
                '\'' => {
                    // HACK: Work around mut borrow.
                    match self.base.peek() {
                        Some(next) if next.get_char() == '\\' => {
                            self.status = CharClassesStatus::LitChar;
                            return Some((char_kind, item));
                        }
                        _ => (),
                    }

                    match self.base.peek() {
                        Some(next) if next.get_char() == '\'' => CharClassesStatus::LitChar,
                        _ => CharClassesStatus::Normal,
                    }
                }
                '/' => match self.base.peek() {
                    Some(next) if next.get_char() == '*' => {
                        self.status = CharClassesStatus::BlockCommentOpening(1);
                        return Some((FullCodeCharKind::StartComment, item));
                    }
                    Some(next) if next.get_char() == '/' => {
                        self.status = CharClassesStatus::LineComment;
                        return Some((FullCodeCharKind::StartComment, item));
                    }
                    _ => CharClassesStatus::Normal,
                },
                _ => CharClassesStatus::Normal,
            },
            CharClassesStatus::StringInBlockComment(deepness) => {
                char_kind = FullCodeCharKind::InStringCommented;
                if chr == '"' {
                    CharClassesStatus::BlockComment(deepness)
                } else if chr == '*' && self.base.peek().map(RichChar::get_char) == Some('/') {
                    char_kind = FullCodeCharKind::InComment;
                    CharClassesStatus::BlockCommentClosing(deepness - 1)
                } else {
                    CharClassesStatus::StringInBlockComment(deepness)
                }
            }
            CharClassesStatus::BlockComment(deepness) => {
                assert_ne!(deepness, 0);
                char_kind = FullCodeCharKind::InComment;
                match self.base.peek() {
                    Some(next) if next.get_char() == '/' && chr == '*' => {
                        CharClassesStatus::BlockCommentClosing(deepness - 1)
                    }
                    Some(next) if next.get_char() == '*' && chr == '/' => {
                        CharClassesStatus::BlockCommentOpening(deepness + 1)
                    }
                    _ if chr == '"' => CharClassesStatus::StringInBlockComment(deepness),
                    _ => self.status,
                }
            }
            CharClassesStatus::BlockCommentOpening(deepness) => {
                assert_eq!(chr, '*');
                self.status = CharClassesStatus::BlockComment(deepness);
                return Some((FullCodeCharKind::InComment, item));
            }
            CharClassesStatus::BlockCommentClosing(deepness) => {
                assert_eq!(chr, '/');
                if deepness == 0 {
                    self.status = CharClassesStatus::Normal;
                    return Some((FullCodeCharKind::EndComment, item));
                } else {
                    self.status = CharClassesStatus::BlockComment(deepness);
                    return Some((FullCodeCharKind::InComment, item));
                }
            }
            CharClassesStatus::LineComment => match chr {
                '\n' => {
                    self.status = CharClassesStatus::Normal;
                    return Some((FullCodeCharKind::EndComment, item));
                }
                _ => {
                    self.status = CharClassesStatus::LineComment;
                    return Some((FullCodeCharKind::InComment, item));
                }
            },
        };
        Some((char_kind, item))
    }
}

/// An iterator over the lines of a string, paired with the char kind at the
/// end of the line.
pub struct LineClasses<'a> {
    base: iter::Peekable<CharClasses<std::str::Chars<'a>>>,
    kind: FullCodeCharKind,
}

impl<'a> LineClasses<'a> {
    pub fn new(s: &'a str) -> Self {
        LineClasses {
            base: CharClasses::new(s.chars()).peekable(),
            kind: FullCodeCharKind::Normal,
        }
    }
}

impl<'a> Iterator for LineClasses<'a> {
    type Item = (FullCodeCharKind, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.base.peek()?;

        let mut line = String::new();

        let start_kind = match self.base.peek() {
            Some((kind, _)) => *kind,
            None => unreachable!(),
        };

        for (kind, c) in self.base.by_ref() {
            // needed to set the kind of the ending character on the last line
            self.kind = kind;
            if c == '\n' {
                self.kind = match (start_kind, kind) {
                    (FullCodeCharKind::Normal, FullCodeCharKind::InString) => {
                        FullCodeCharKind::StartString
                    }
                    (FullCodeCharKind::InString, FullCodeCharKind::Normal) => {
                        FullCodeCharKind::EndString
                    }
                    (FullCodeCharKind::InComment, FullCodeCharKind::InStringCommented) => {
                        FullCodeCharKind::StartStringCommented
                    }
                    (FullCodeCharKind::InStringCommented, FullCodeCharKind::InComment) => {
                        FullCodeCharKind::EndStringCommented
                    }
                    _ => kind,
                };
                break;
            }
            line.push(c);
        }

        // Workaround for CRLF newline.
        if line.ends_with('\r') {
            line.pop();
        }

        Some((self.kind, line))
    }
}

pub struct CommentCodeSlices<'a> {
    slice: &'a str,
    last_slice_kind: CodeCharKind,
    last_slice_end: usize,
}

impl<'a> CommentCodeSlices<'a> {
    pub fn new(slice: &'a str) -> CommentCodeSlices<'a> {
        CommentCodeSlices {
            slice,
            last_slice_kind: CodeCharKind::Comment,
            last_slice_end: 0,
        }
    }
}

impl<'a> Iterator for CommentCodeSlices<'a> {
    type Item = (CodeCharKind, usize, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        if self.last_slice_end == self.slice.len() {
            return None;
        }

        let mut sub_slice_end = self.last_slice_end;
        let mut first_whitespace = None;
        let subslice = &self.slice[self.last_slice_end..];
        let mut iter = CharClasses::new(subslice.char_indices());

        for (kind, (i, c)) in &mut iter {
            let is_comment_connector = self.last_slice_kind == CodeCharKind::Normal
                && &subslice[..2] == "//"
                && [' ', '\t'].contains(&c);

            if is_comment_connector && first_whitespace.is_none() {
                first_whitespace = Some(i);
            }

            if kind.to_codecharkind() == self.last_slice_kind && !is_comment_connector {
                let last_index = match first_whitespace {
                    Some(j) => j,
                    None => i,
                };
                sub_slice_end = self.last_slice_end + last_index;
                break;
            }

            if !is_comment_connector {
                first_whitespace = None;
            }
        }

        if let (None, true) = (iter.next(), sub_slice_end == self.last_slice_end) {
            // This was the last subslice.
            sub_slice_end = match first_whitespace {
                Some(i) => self.last_slice_end + i,
                None => self.slice.len(),
            };
        }

        let kind = match self.last_slice_kind {
            CodeCharKind::Comment => CodeCharKind::Normal,
            CodeCharKind::Normal => CodeCharKind::Comment,
        };
        let res = (
            kind,
            self.last_slice_end,
            &self.slice[self.last_slice_end..sub_slice_end],
        );
        self.last_slice_end = sub_slice_end;
        self.last_slice_kind = kind;

        Some(res)
    }
}

pub fn filter_normal_code(code: &str) -> String {
    let mut buffer = String::with_capacity(code.len());
    LineClasses::new(code).for_each(|(kind, line)| match kind {
        FullCodeCharKind::Normal
        | FullCodeCharKind::StartString
        | FullCodeCharKind::InString
        | FullCodeCharKind::EndString => {
            buffer.push_str(&line);
            buffer.push('\n');
        }
        _ => (),
    });
    if !code.ends_with('\n') && buffer.ends_with('\n') {
        buffer.pop();
    }
    buffer
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shape::{Indent, Shape};

    #[test]
    fn char_classes() {
        let mut iter = CharClasses::new("//\n\n".chars());

        assert_eq!((FullCodeCharKind::StartComment, '/'), iter.next().unwrap());
        assert_eq!((FullCodeCharKind::InComment, '/'), iter.next().unwrap());
        assert_eq!((FullCodeCharKind::EndComment, '\n'), iter.next().unwrap());
        assert_eq!((FullCodeCharKind::Normal, '\n'), iter.next().unwrap());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn comment_code_slices() {
        let input = "code(); /* test */ 1 + 1";
        let mut iter = CommentCodeSlices::new(input);

        assert_eq!((CodeCharKind::Normal, 0, "code(); "), iter.next().unwrap());
        assert_eq!(
            (CodeCharKind::Comment, 8, "/* test */"),
            iter.next().unwrap()
        );
        assert_eq!((CodeCharKind::Normal, 18, " 1 + 1"), iter.next().unwrap());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_contains_comment() {
        assert_eq!(contains_comment("abc"), false);
        assert_eq!(contains_comment("abc // qsdf"), true);
        assert_eq!(contains_comment("abc /* kqsdf"), true);
        assert_eq!(contains_comment("abc \" /* */\" qsdf"), false);
    }

    #[test]
    fn test_find_uncommented() {
        fn check(haystack: &str, needle: &str, expected: Option<usize>) {
            assert_eq!(expected, haystack.find_uncommented(needle));
        }

        check("/*/ */test", "test", Some(6));
        check("//test\ntest", "test", Some(7));
        check("/* comment only */", "whatever", None);
        check(
            "/* comment */ some text /* more commentary */ result",
            "result",
            Some(46),
        );
        check("sup // sup", "p", Some(2));
        check("sup", "x", None);
        check(r#"π? /**/ π is nice!"#, r#"π is nice"#, Some(9));
        check("/*sup yo? \n sup*/ sup", "p", Some(20));
        check("hel/*lohello*/lo", "hello", None);
        check("acb", "ab", None);
        check(",/*A*/ ", ",", Some(0));
        check("abc", "abc", Some(0));
        check("/* abc */", "abc", None);
        check("/**/abc/* */", "abc", Some(4));
        check("\"/* abc */\"", "abc", Some(4));
        check("\"/* abc", "abc", Some(4));
    }

    #[test]
    fn test_filter_normal_code() {
        let s = r#"
fn main() {
    println!("hello, world");
}
"#;
        assert_eq!(s, filter_normal_code(s));
        let s_with_comment = r#"
fn main() {
    // hello, world
    println!("hello, world");
}
"#;
        assert_eq!(s, filter_normal_code(s_with_comment));
    }

    #[test]
    fn test_rewrite_comment_1() {
        let orig = "/* This is a multi-line\n * comment */\n\n// This is a single-line comment";
        let block_style = false;
        // let style = CommentStyle::SingleBullet;
        let indent = Indent::new(0, 0);
        let shape = Shape {
            width: 100,
            indent,
            offset: 0,
        };
        let config = &Config::default();
        // let is_doc_comment = false;

        // let rewritten_comment = rewrite_comment_inner(orig, block_style, style, shape, config, is_doc_comment);

        // if let Some(comment) = rewritten_comment {
        //     println!("{}", comment);
        // }
        // /* This is a multi-line
        // * comment */
        // *
        // * This is a single-line comment */
        if let Some(comment) = rewrite_comment(orig, block_style, shape, config) {
            println!("{}", comment);
        }
    }
}
