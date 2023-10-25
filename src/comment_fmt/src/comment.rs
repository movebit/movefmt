use std::{self, borrow::Cow, iter};
use itertools::{multipeek, MultiPeek};
use lazy_static::lazy_static;
use regex::Regex;
use crate::config::Config;
use crate::shape::{Indent, Shape};
use crate::string::{rewrite_string, StringFormat};
use crate::utils::{
    count_newlines, last_line_width, trim_left_preserve_layout,
    unicode_str_width,
};

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

// 该函数的作用是判断给定的字符串comment是否符合自定义注释的格式.
// 自定义注释的格式要求以//开头,后面紧跟一个非字母数字字符或非空白字符.

// 下面是函数的工作原理:

// 首先,通过!comment.starts_with("//")检查字符串comment是否以//开头,如果不是,则返回false.
// 接下来,通过comment.chars().nth(2)获取字符串comment中索引为2的字符.如果索引为2的字符存在,则执行下一步,否则返回false.
// 最后,通过!c.is_alphanumeric() && !c.is_whitespace()判断索引为2的字符是否既不是字母数字字符也不是空白字符.
// 如果是,返回true,表示是自定义注释;否则,返回false.
// 这样,您可以将一个字符串传递给is_custom_comment函数,并得到一个布尔值来判断该字符串是否符合自定义注释的格式.
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

// 从输入字符串s中提取第一行的开头部分,直到遇到第一个空格字符为止.如果输入字符串为空或没有空格字符,则返回一个空字符串.

// 下面是函数的工作原理:
// s.lines()将输入字符串按行拆分为迭代器.
// next()获取迭代器的第一个元素,即第一行.
// map_or("", |first_line| { ... })检查第一行是否存在,如果不存在则返回一个空字符串.
// first_line.find(' ')在第一行中查找第一个空格字符的索引.
// map_or(first_line, |space_index| &first_line[0..=space_index])如果找到了空格字符,则返回从第一行开头到空格字符处的子串,
// 否则返回整个第一行.
// 这样,您可以将输入字符串传递给custom_opener函数,并得到第一行的开头部分作为结果.
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

// 该函数的作用是识别和重写给定的注释字符串orig,并根据配置和注释样式进行格式化.
// 函数根据注释的样式和配置参数进行不同的处理逻辑,并返回一个可选的重写后的字符串.

// 函数内部定义了一些辅助函数和局部变量,用于处理不同类型的注释.根据注释的样式和配置参数,
// 函数使用递归的方式处理注释的多行和嵌套情况,并将重写后的注释字符串进行拼接和返回.

// identify_comment函数是一个用于识别和重写注释的函数.它接受多个参数,包括原始注释字符串orig,
// 注释块样式block_style,格式化的形状shape,配置信息config以及一个布尔值is_doc_comment,表示是否为文档注释.

// 函数的主要目标是将原始注释字符串进行格式化,并返回一个可选的重写后的字符串.下面是函数的实现细节的解释:

// 首先,函数会根据原始注释字符串的风格调用comment_style函数,得到注释风格的枚举类型CommentStyle.
// 接下来,函数会根据注释风格的不同分别处理:
// 对于CommentStyle::DoubleSlash,CommentStyle::TripleSlash和CommentStyle::Doc类型的注释,函数调用
// consume_same_line_comments函数来获取第一组相同风格的行注释,并返回一个元组,其中包含一个布尔值表示是否有空行以及第一组注释的大小.
// 对于CommentStyle::Custom类型的注释,函数调用consume_same_line_comments函数来获取第一组相同风格的自定义注释,
// 并返回一个元组,其中包含一个布尔值表示是否有空行以及第一组注释的大小.
// 对于CommentStyle::DoubleBullet,CommentStyle::SingleBullet和CommentStyle::Exclamation类型的注释,
// 函数会搜索注释的结束符,并跟踪注释的行数和大小.
// 接下来,函数将原始注释字符串分为第一组注释和剩余部分,使用分隔位置first_group_ending.
// 根据配置的不同和注释的风格,函数会选择不同的方式来重写第一组注释:
// 如果配置中禁用了注释的规范化,并且第一组注释中有裸露的行(没有前导的注释标记),并且注释的风格是块注释,
// 则函数调用trim_left_preserve_layout函数来保留第一组注释的布局.
// 如果配置中禁用了注释的规范化,并且没有启用注释的换行,并且(对于文档注释块)没有启用在注释中格式化代码的选项,
// 则函数调用light_rewrite_comment函数来轻量级地重写第一组注释.
// 否则,函数调用rewrite_comment_inner函数来详细地重写第一组注释,根据注释的风格和配置参数进行格式化.
// 最后,如果剩余部分为空,则函数返回重写后的第一组注释.否则,函数递归调用identify_comment函数来处理剩余部分,
// 并将重写后的剩余部分与第一组注释拼接在一起,返回最终的重写后的注释字符串.
// 总体来说,identify_comment函数通过递归和不同的处理逻辑来识别和重写注释,根据注释的样式和配置参数对注释进行格式化.
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
        
        // 首先,获取注释的结束符号,并去除前导空白字符,得到closer变量.
        // 然后,定义了一些变量,包括count,closing_symbol_offset,hbl,first.count用于记录注释中结束符号出现的次数,
        // closing_symbol_offset用于记录注释内容的字节长度,hbl用于标记是否存在裸露的行(即非注释行),first用于标记是否是第一行.
        // 接下来,对原始字符串进行逐行遍历.在每一行中,将closing_symbol_offset增加上当前行的字节长度(通过compute_len函数计算得到).
        // 然后,对当前行进行修剪,去除前导空白字符,并将结果赋值给trimmed_line变量.
        // 接着,检查trimmed_line是否以*,//或/*开头.如果不是,则将hbl设置为true,表示存在裸露的行.
        // 在搜索结束符号时,首先判断是否是第一行.如果是第一行,则去除开启符号(通过style.opener().trim_end()获取)后的部分,
        // 即从trimmed_line中去除开启符号,并将结果重新赋值给trimmed_line.
        // 然后,检查trimmed_line是否以closer结尾.如果是,则将count减1.如果count等于0,表示已经找到了所有的结束符号,跳出循环.
        // 最后,返回一个元组,包括hbl和closing_symbol_offset.其中,hbl表示是否存在裸露的行,closing_symbol_offset表示注释内容的字节长度.
        // for a block comment, search for the closing symbol
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
    let rewritten_first_group =
        if !config.normalize_comments() && has_bare_lines && style.is_block_comment() {
            trim_left_preserve_layout(first_group, shape.indent, config)?
        } else if !config.normalize_comments()
            && !config.wrap_comments()
            && !(
                // `format_code_in_doc_comments` should only take effect on doc comments,
                // so we only consider it when this comment block is a doc comment block.
                is_doc_comment && config.format_code_in_doc_comments()
            )
        {
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
            format!(
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
            )
        })
        /*
        将rest进行trim_start()操作,去除前导空白字符,然后作为参数传递给identify_comment函数,
        同时传递其他参数block_style,shape,config和is_doc_comment.

        接着,通过.map(|rest_str| { ... })对identify_comment函数的返回结果进行处理.在闭包中,
        通过format!宏将重写后的第一组注释,空行符和缩进拼接成一个新的字符串.

        具体拼接的逻辑如下:

        重写后的第一组注释作为第一个参数.
        如果has_bare_lines为true且注释类型为行注释,则在重写后的第一组注释后插入一个空行符(\n);否则不插入空行符.
        使用shape.indent.to_string(config)将缩进形状转换为字符串,并作为第三个参数.
        rest_str作为第四个参数,即剩余部分的重写结果.
        最终,返回的是拼接后的字符串,其中包括重写后的第一组注释,空行符(如果有的话),缩进和剩余部分的重写结果.
        这样就将重写后的注释与剩余部分的重写结果连接在一起. 
        */
    }
}

/// Enum indicating if the code block contains rust based on attributes
enum CodeBlockAttribute {
    Rust,
    NotRust,
}

impl CodeBlockAttribute {
    // 用于解析逗号分隔的属性列表.如果所有属性都是有效的Rust属性,则返回Rust;否则返回NotRust.

    // 在new方法中,使用attributes.split(',')将属性列表拆分为单个属性.然后使用match语句对每个属性进行匹配和处理.

    // 空字符串,"rust","should_panic","no_run","edition2015","edition2018","edition2021"
    // 被认为是有效的Rust属性,不做任何操作.
    // "ignore","compile_fail","text"被认为是不包含Rust代码的属性,直接返回NotRust.
    // 其他属性被认为是不包含Rust代码的属性,直接返回NotRust.
    // 如果所有属性都是有效的Rust属性,则最后返回Rust.
    /// Parse comma separated attributes list. Return rust only if all
    /// attributes are valid rust attributes
    /// See <https://doc.rust-lang.org/rustdoc/print.html#attributes>
    fn new(attributes: &str) -> CodeBlockAttribute {
        for attribute in attributes.split(',') {
            match attribute.trim() {
                "" | "rust" | "should_panic" | "no_run" | "edition2015" | "edition2018"
                | "edition2021" => (),
                "ignore" | "compile_fail" | "text" => return CodeBlockAttribute::NotRust,
                _ => return CodeBlockAttribute::NotRust,
            }
        }
        CodeBlockAttribute::Rust
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
            let style = comment_style(orig, config.normalize_comments());
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

    // orig: &'a str:表示原始的文本字符串.
    // i: usize:表示当前处理的文本行的索引.
    // line: &'a str:表示当前处理的文本行的内容.
    // has_leading_whitespace: bool:表示当前处理的文本行是否有前导空白字符.
    // is_doc_comment: bool:表示当前处理的文本行是否是文档注释.
    // 该函数的主要逻辑如下:

    // 首先,函数会统计原始文本字符串中的换行符数量,并将结果保存在num_newlines变量中.
    // 接着,函数判断当前是否处于一个代码块的属性中(self.code_block_attr是否存在).
    // 如果是,则根据不同的条件对代码块进行处理.
    // 如果当前行以"```"开头,则表示代码块结束,需要将代码块的内容添加到self.result中,并清空self.code_block_buffer.
    // 然后将当前行添加到self.result中,并将self.code_block_attr设置为None.
    // 如果当前行不是以"```"开头,则表示代码块还未结束,需要将当前行添加到self.code_block_buffer中,并在行末尾添加换行符.
    // 最后,返回false表示处理未完成.
    // 如果不处于代码块属性中,则继续进行后续处理.
    // 首先,将self.code_block_attr设置为None.
    // 如果当前行以"```"开头,则表示进入了一个新的代码块属性,需要将该属性保存到self.code_block_attr中.
    // 如果self.result等于self.opener,则表示当前行是第一行,需要根据一些条件进行处理.
    // 如果当前行是空行,则返回false表示处理未完成.
    // 如果self.is_prev_line_multi_line为true并且当前行不为空,则在self.result末尾添加一个空格.
    // 如果是最后一行并且当前行为空,则删除self.result中的尾部空格(如果存在),并返回true表示处理完成.
    // 否则,将self.comment_line_separator添加到self.result中,并根据一些条件处理self.result末尾的空格.
    // 判断当前行是否是Markdown标题文档注释,如果是,则将is_markdown_header_doc_comment设置为true.
    // 根据一些条件判断是否需要对注释进行换行处理.
    // 如果需要进行换行处理,则调用rewrite_string函数对当前行进行重写,并将结果添加到self.result中.
    // 如果无法将当前行与前一行放在同一行,则将self.result末尾的空格删除,并在下一行开始进行重写.
    // 如果无法对当前行进行重写,则将当前行直接添加到self.result中.
    // 更新self.fmt.shape的值,用于下一行的重写.
    // 返回false表示处理未完成.
    // 这段代码的作用是对输入的文本行进行处理,根据不同的条件对注释进行重写,换行等操作,并将结果存储在self.result中.
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
        let should_wrap_comment = self.fmt.config.wrap_comments()
            && !is_markdown_header_doc_comment
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

// 首先,创建一个CommentRewrite实例,用于处理注释的重写操作.
// 统计原始注释字符串的末尾换行符的数量,并将结果保存在line_breaks变量中.
// 对原始注释字符串进行逐行处理.
// 首先,去除行首和行尾的空白字符,并修剪注释行中的内容.
// 如果当前行是最后一行且以"/"结尾(且不以"//"开头),则去除行尾的"/".
// 对修剪后的注释行进行左对齐处理,并记录是否有前导空白字符.
// 根据一些条件判断是否需要对注释行进行进一步处理.
// 如果原始注释以"/*"开头且只有一行,则去除行首的空白字符,并将config.normalize_comments()的结果与
// has_leading_whitespace进行逻辑或运算.
// 否则,保持注释行不变,并将config.normalize_comments()的结果与has_leading_whitespace进行逻辑或运算.
// 对每一行进行处理,调用rewriter.handle_line方法进行注释行的处理,如果返回true,则表示处理完成,退出循环.
// 返回重写后的注释结果,通过Some包装.
// 这段代码的作用是对输入的注释进行重写,根据不同的条件进行修剪,格式化等操作,并返回重写后的结果字符串.
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
                (
                    line.trim_start(),
                    has_leading_whitespace || config.normalize_comments(),
                )
            } else {
                (line, has_leading_whitespace || config.normalize_comments())
            }
        });

    for (i, (line, has_leading_whitespace)) in lines.enumerate() {
        if rewriter.handle_line(orig, i, line, has_leading_whitespace, is_doc_comment) {
            break;
        }
    }

    Some(rewriter.finish())
}

const RUSTFMT_CUSTOM_COMMENT_PREFIX: &str = "//#### ";

fn hide_sharp_behind_comment(s: &str) -> Cow<'_, str> {
    let s_trimmed = s.trim();
    if s_trimmed.starts_with("# ") || s_trimmed == "#" {
        Cow::from(format!("{RUSTFMT_CUSTOM_COMMENT_PREFIX}{s}"))
    } else {
        Cow::from(s)
    }
}

fn trim_custom_comment_prefix(s: &str) -> String {
    s.lines()
        .map(|line| {
            let left_trimmed = line.trim_start();
            if left_trimmed.starts_with(RUSTFMT_CUSTOM_COMMENT_PREFIX) {
                left_trimmed.trim_start_matches(RUSTFMT_CUSTOM_COMMENT_PREFIX)
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
                if l.as_bytes()[fnw] == b'*' && fnw > 0 {
                    &l[fnw - 1..]
                } else {
                    &l[fnw..]
                }
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
    // 在字符串中查找未被注释的子字符串,并返回其起始位置.它通过迭代字符串的字符和给定的模式字符串来进行匹配.
    // 如果找到了完全匹配的子字符串,则返回其起始位置减去模式字符串的长度.如果迭代完毕后仍未找到匹配的子字符串,
    // 则检查模式字符串是否是搜索字符串的后缀,如果是,则返回搜索字符串的长度减去模式字符串的长度;否则返回None.
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

    // 在字符串中查找最后一个未被注释的子字符串,并返回其起始位置.
    // 首先调用find_uncommented方法在整个字符串中查找匹配的子字符串.
    // 如果找到了,则将起始位置保存在result变量中.然后,它在字符串的剩余部分继续调用find_last_uncommented方法,
    // 并将找到的匹配子字符串的起始位置加上1后累加到result中.最后,返回result作为最后一个匹配子字符串的起始位置.
    // 如果未找到匹配的子字符串,则返回None.
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

// 用于找到第一个注释之后的字节位置.传入的字符串预期以注释开头,并包含注释的分隔符.
// 函数首先创建了一个CharClasses迭代器,用于对字符串的字符进行分类.然后通过遍历迭代器的元素,
// 找到第一个属于FullCodeCharKind::Normal或FullCodeCharKind::InString的字符,并返回其字节位置.
// 如果注释在s的末尾结束,则处理这种情况,并返回s的长度作为注释结束的位置.
// 如果迭代器的状态为CharClassesStatus::Normal,则说明没有找到注释,返回None.

// 这个函数用于在字符串中找到第一个注释之后的位置,并提供了一些示例来说明正确和错误的注释用法.
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

// Normal:表示普通字符状态.
// LitString:表示字符在字符串内部.
// LitStringEscape:表示字符在字符串内部的转义序列中.
// LitRawString(u32):表示字符在原始字符串内部,其中u32表示原始字符串的前缀长度.
// RawStringPrefix(u32):表示字符在原始字符串前缀内部,其中u32表示原始字符串的前缀长度.
// RawStringSuffix(u32):表示字符在原始字符串后缀内部,其中u32表示原始字符串的后缀长度.
// LitChar:表示字符在字符字面值内部.
// LitCharEscape:表示字符在字符字面值的转义序列内部.
// BlockComment(u32):表示字符在块注释内部,其中u32表示块注释的嵌套深度.
// StringInBlockComment(u32):表示字符在被块注释包围的字符串内部,其中u32表示块注释的嵌套深度.
// BlockCommentOpening(u32):表示字符在注释开头的状态,其中u32表示注释的嵌套深度(在注释打开之后).
// BlockCommentClosing(u32):表示字符在注释结尾的状态,其中u32表示注释的嵌套深度(在注释关闭之前).
// LineComment:表示字符在行注释内部.
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

/// 遍历字符串中的功能代码和注释部分.字符串的任何部分都可以是功能代码,一个块注释或一个行注释.
/// 注释之间的空白部分被视为功能代码.行注释包含其结束的换行符.
/// slice是一个对输入字符串的引用,表示要遍历的字符串.
/// iter是一个CharClasses类型的迭代器,它对输入字符串的字符进行分类,
/// 并使用Peekable进行包装,以便可以预览下一个元素.
struct UngroupedCommentCodeSlices<'a> {
    slice: &'a str,
    iter: iter::Peekable<CharClasses<std::str::CharIndices<'a>>>,
}

impl<'a> UngroupedCommentCodeSlices<'a> {
    fn new(code: &'a str) -> UngroupedCommentCodeSlices<'a> {
        UngroupedCommentCodeSlices {
            slice: code,
            iter: CharClasses::new(code.char_indices()).peekable(),
        }
    }
}

// 遍历注释中的代码片段,将其分开
// 将字符串分割成功能代码和注释部分,并提供一个迭代器来遍历这些部分
impl<'a> Iterator for UngroupedCommentCodeSlices<'a> {
    type Item = (CodeCharKind, usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (kind, (start_idx, _)) = self.iter.next()?;
        match kind {
            FullCodeCharKind::Normal | FullCodeCharKind::InString => {
                // Consume all the Normal code
                while let Some(&(char_kind, _)) = self.iter.peek() {
                    if char_kind.is_comment() {
                        break;
                    }
                    let _ = self.iter.next();
                }
            }
            FullCodeCharKind::StartComment => {
                // Consume the whole comment
                loop {
                    match self.iter.next() {
                        Some((kind, ..)) if kind.inside_comment() => continue,
                        _ => break,
                    }
                }
            }
            _ => panic!(),
        }
        let slice = match self.iter.peek() {
            Some(&(_, (end_idx, _))) => &self.slice[start_idx..end_idx],
            None => &self.slice[start_idx..],
        };
        Some((
            if kind.is_comment() {
                CodeCharKind::Comment
            } else {
                CodeCharKind::Normal
            },
            start_idx,
            slice,
        ))
    }
}

// 在字符串中迭代出功能部分和注释部分的子串.
// 第一个元素始终是一个可能为空的功能文本子串.行注释包含其结束的换行符.
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
    // 在迭代过程中返回下一个元素
    // 每个元素是一个元组,包含了切片的类型,切片的起始位置和切片的子串
    // 在迭代过程中返回字符串中交替出现的功能部分和注释部分的子串.
    fn next(&mut self) -> Option<Self::Item> {
        // 检查上一个切片的结束位置是否等于字符串的长度,如果是,则返回None,表示迭代结束
        if self.last_slice_end == self.slice.len() {
            return None;
        }

        let mut sub_slice_end = self.last_slice_end;  // 表示当前切片的结束位置
        let mut first_whitespace = None;  // 表示第一个空白字符的位置(如果有的话)
        let subslice = &self.slice[self.last_slice_end..];  // 表示当前切片的子串
        let mut iter = CharClasses::new(subslice.char_indices());

        for (kind, (i, c)) in &mut iter {
            let is_comment_connector = self.last_slice_kind == CodeCharKind::Normal
                && &subslice[..2] == "//"
                && [' ', '\t'].contains(&c);

            // 如果当前字符是注释连接符(//后面的空白字符),并且
            // first_whitespace为空,则将first_whitespace设置为当前索引
            if is_comment_connector && first_whitespace.is_none() {
                first_whitespace = Some(i);
            }

            // 当前字符的类型是否与上一个切片的类型相同,并且不是注释连接符.如果满足条件,
            // 则将sub_slice_end设置为上一个切片的结束位置加上first_whitespace的值,并跳出循环
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

        // 方法检查是否是最后一个子切片.如果是,则将sub_slice_end设置为上一个切片的结束位置 加上
        // first_whitespace的值(如果有的话),或者字符串的长度
        if let (None, true) = (iter.next(), sub_slice_end == self.last_slice_end) {
            // This was the last subslice.
            sub_slice_end = match first_whitespace {
                Some(i) => self.last_slice_end + i,
                None => self.slice.len(),
            };
        }

        // 根据上一个切片的类型,计算当前切片的类型,并构造一个元组(CodeCharKind, usize, &'a str)作为返回值
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

/// Iterator over the 'payload' characters of a comment.
/// It skips whitespace, comment start/end marks, and '*' at the beginning of lines.
/// The comment must be one comment, ie not more than one start mark (no multiple line comments,
/// for example).
struct CommentReducer<'a> {
    is_block: bool,
    at_start_line: bool,
    iter: std::str::Chars<'a>,
}

impl<'a> CommentReducer<'a> {
    fn new(comment: &'a str) -> CommentReducer<'a> {
        let is_block = comment.starts_with("/*");
        let comment = remove_comment_header(comment);
        CommentReducer {
            is_block,
            // There are no supplementary '*' on the first line.
            at_start_line: false,
            iter: comment.chars(),
        }
    }
}

impl<'a> Iterator for CommentReducer<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut c = self.iter.next()?;
            if self.is_block && self.at_start_line {
                while c.is_whitespace() {
                    c = self.iter.next()?;
                }
                // Ignore leading '*'.
                if c == '*' {
                    c = self.iter.next()?;
                }
            } else if c == '\n' {
                self.at_start_line = true;
            }
            if !c.is_whitespace() {
                return Some(c);
            }
        }
    }
}

fn remove_comment_header(comment: &str) -> &str {
    if comment.starts_with("///") || comment.starts_with("//!") {
        &comment[3..]
    } else if let Some(stripped) = comment.strip_prefix("//") {
        stripped
    } else if (comment.starts_with("/**") && !comment.starts_with("/**/"))
        || comment.starts_with("/*!")
    {
        &comment[3..comment.len() - 2]
    } else {
        assert!(
            comment.starts_with("/*"),
            "string '{comment}' is not a comment"
        );
        &comment[2..comment.len() - 2]
    }
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
        // let shape = Shape::legacy(80, IndentStyle::Block, WidthHeuristics::contextual(0, 0));
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
