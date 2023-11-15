## Background

A formatting tool, also known as a pretty-printer, prints the AST of the corresponding language into a beautifully formatted string.

Implementing a formatter for programming language always with a lot of work. First of all, a language AST always has a lot of variant data structure, like move expression.

```rust
pub enum Exp_ {
    Value(Value),
    // move(x)
    Move(Var),
    // copy(x)
    Copy(Var),
    // [m::]n[<t1, .., tn>]
    Name(NameAccessChain, Option<Vec<Type>>),

    // f(earg,*)
    // f!(earg,*)
    Call(NameAccessChain, bool, Option<Vec<Type>>, Spanned<Vec<Exp>>),

    // tn {f1: e1, ... , f_n: e_n }
    Pack(NameAccessChain, Option<Vec<Type>>, Vec<(Field, Exp)>),

    // vector [ e1, ..., e_n ]
    // vector<t> [e1, ..., en ]
    Vector(
        /* name loc */ Loc,
        Option<Vec<Type>>,
        Spanned<Vec<Exp>>,
    ),

    // if (eb) et else ef
    IfElse(Box<Exp>, Box<Exp>, Option<Box<Exp>>),
    // while (eb) eloop
    While(Box<Exp>, Box<Exp>),
    // loop eloop
    Loop(Box<Exp>),

    // { seq }
    Block(Sequence),
    // fun (x1, ..., xn) e
    Lambda(BindList, Box<Exp>), // spec only
    // forall/exists x1 : e1, ..., xn [{ t1, .., tk } *] [where cond]: en.
    Quant(
        QuantKind,
        BindWithRangeList,
        Vec<Vec<Exp>>,
        Option<Box<Exp>>,
        Box<Exp>,
    ), // spec only
    // (e1, ..., en)
    ExpList(Vec<Exp>),
    // ()
    Unit,
    
    ...
}
```
This is just `Expression` variants. There are also `Function`,`Module`,`Struct`,`Spec`,etc. Implement a formatter you have to deal all the data structure.


## Challenge of movefmt 
### Spec
Spec is the abbreviation for Move specification language in AST, a subset of the Move language which supports specification of the behavior of Move programs. It contains many grammar elements such as modules, type system, functions, declaration statements, quantifier expressions, and so on.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecApplyPattern_ {
    pub visibility: Option<Visibility>,
    pub name_pattern: Vec<SpecApplyFragment>,
    pub type_parameters: Vec<(Name, Vec<Ability>)>,
}

pub type SpecApplyPattern = Spanned<SpecApplyPattern_>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecApplyFragment_ {
    Wildcard,
    NamePart(Name),
}

pub type SpecApplyFragment = Spanned<SpecApplyFragment_>;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SpecBlockMember_ {
    Condition {
        kind: SpecConditionKind,
        properties: Vec<PragmaProperty>,
        exp: Exp,
        additional_exps: Vec<Exp>,
    },
    Function {
        uninterpreted: bool,
        name: FunctionName,
        signature: FunctionSignature,
        body: FunctionBody,
    },
    Variable {
        is_global: bool,
        name: Name,
        type_parameters: Vec<(Name, Vec<Ability>)>,
        type_: Type,
        init: Option<Exp>,
    },
    Let {
        name: Name,
        post_state: bool,
        def: Exp,
    },
    Update {
        lhs: Exp,
        rhs: Exp,
    },
    Include {
        properties: Vec<PragmaProperty>,
        exp: Exp,
    },
    Apply {
        exp: Exp,
        patterns: Vec<SpecApplyPattern>,
        exclusion_patterns: Vec<SpecApplyPattern>,
    },
    Pragma {
        properties: Vec<PragmaProperty>,
    },
}

pub type SpecBlockMember = Spanned<SpecBlockMember_>;

// Specification condition kind.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SpecConditionKind_ {
    Assert,
    Assume,
    Decreases,
    AbortsIf,
    AbortsWith,
    SucceedsIf,
    Modifies,
    Emits,
    Ensures,
    Requires,
    Invariant(Vec<(Name, Vec<Ability>)>),
    InvariantUpdate(Vec<(Name, Vec<Ability>)>),
    Axiom(Vec<(Name, Vec<Ability>)>),
}
pub type SpecConditionKind = Spanned<SpecConditionKind_>;

```
It is a very complex language feature that movefmt needs to support.

### Comment
Another complex thing about formatter is `Comments`. Move programming language support three forms of comments.
* Block Comment -> /**/
* Line Comment -> // 
* Documentation Comment -> ///

`Comments` can write anywhere in source code.

In order to keep user's comments you have to keep comments in AST like below.
```rust
    // f(earg,*)
    // f!(earg,*)
    Call(NameAccessChain,
        Vec<Comment> , // keep in AST.
        bool, Option<Vec<Type>>, Spanned<Vec<Exp>>),
```
This will make things below ugly.

* AST Definition.
* Parse AST.
* All routine that accept AST.

In general We need keep a `Vec<Comment>` in every AST structure. But in the move-compiler module, after syntax analysis, comments are filtered and there is no comment information on the AST.


Is there a way to slove this puzzle?


## MOVEFMT SOLUTION
再进行下一节的方案细节前,本小节,我们讲下我们的大致思路.
1.两大核心结构
FormatContext,TokenTree
2.其他重要结构
FormatEnv,FormatConfig
3.注释处理结构
CommentExtrator,Comment
4.重写主逻辑结构
Fmt

整体逻辑就是:


1.step1:从源码中得到各种数据结构

FormatConfig 存放格式规则, 
通过词法分析得到 TokenTree,
通过语法分析拿到 AST,
通过 CommentExtrator 拿到 Comment.

2.step2:遍历 TokenTree,进行格式化
Fmt 在遍历 TokenTree 的时候,会结合 move_compiler::parser::ast::Definition,
计算出当前代码块的 FormatEnv,即要知道当前代码块是函数块还是结构体块或是use语句块等等.
FormatEnv 会存在 FormatContext 结构里.然后根据 FormatEnv, Fmt 会调用不同
的formatter,比如 use_fmt, expr_fmt 等等.

3.step3:
在一个具体的 FormatContext 上下文里,按 FormatConfig 去做具体的格式化.


首先,我们会设计一个结构 TokenTree, 把来自 parser 返回的 AST 分为两类,
一类是代码块,即由"()","{}"包裹的代码组成的 NestedToken, 
其余的就都是 SimpleToken.

显而易见, NestedToken 是一个嵌套结构.
而任何一个 move module,在忽略模块名的情况下,本质上就是一个 NestedToken.
例如:
```rust
module econia::incentives {  // NestedToken1
    // ...
    {  // NestedToken2
        {  // NestedToken3
            // ...
        }
    }
    // ...
}
```
SimpleToken 有如下这些, 共4个:
    "module", "econia", "::", "incentives"

剩下的整个 module 主体,就都放入了 NestedToken1 中.
而 NestedToken1 内部还嵌套着 NestedToken2 --> NestedToken3.


为什么要增加 TokenTree 这样一个抽象结构:

1.原因一:
所谓的对代码进行格式化,无非就是对一个代码文件里的从外到内的逐层的代码块做格式化.
即对 TokenTree 做格式化.所以,我们采用了 TokenTree 的结构来对 parser 的 AST 做进一步的抽象,以
更加贴近格式化的业务本质.

2.原因二:
TokenTree 的数据直接来源于 Lexer, 而 Lexer 对每一个 token 都记录了位置和类型及符号名,
既可以很方便在 token 之间加空格,也很方便在合适的地方加换行符.简而言之,可以更加精细化得控制
输出格式.


## Detail design
### comment parse and process
#### Data structure
```rust
pub struct Comment {
    pub start_offset: u32,
    pub content: String,
}

pub enum CommentKind {
    /// "//"
    InlineComment,
    /// "///"
    DocComment,
    /// "/**/"
    BlockComment,
}

pub struct CommentExtrator {
    pub comments: Vec<Comment>,
}
```

#### Extrator algorithm
```rust
pub enum ExtratorCommentState {
    /// init state
    Init,
    /// `/` has been seen,maybe a comment.
    OneSlash,
    /// `///` has been seen,inline comment.
    InlineComment,
    /// `/*` has been seen,block comment.
    BlockComment,
    /// in state `BlockComment`,`*` has been seen,maybe exit the `BlockComment`.
    OneStar,
    /// `"` has been seen.
    Quote,
}
```

>pseudocode about extracting all kinds of comments
```text
CommentExtrator:
    如果 input_string 的长度小于等于 1,返回空的 CommentExtrator 实例.
    
    初始化:
    创建一个状态机,初始状态为 "未开始".
    创建一个整数深度计数器,初始值为 0.
    创建一个字符串变量来存储当前正在处理的注释内容.
    创建一个 Comment 对象的列表,用于存储提取出的所有注释.
    
    循环遍历输入字符串中的每个字符:
        根据状态机的状态和当前处理的字符,执行相应的操作:
        
        如果状态是 "未开始":
            如果当前字符是 '/',将状态更改为 "单斜杠".
            如果当前字符是 '"',将状态更改为 "引号".
            
        如果状态是 "单斜杠":
            如果下一个字符也是 '/', 将当前字符添加到注释内容中,并将状态更改为 "行内注释".
            如果下一个字符是 '*', 将两个字符都添加到注释内容中,并将深度计数器加一,然后将状态更改为 "块注释".
            否则,如果深度计数器为 0,则将状态更改为 "未开始";否则,将状态更改为 "块注释".
            
        如果状态是 "块注释":
            如果当前字符是 '*',将状态更改为 "单星".
            如果当前字符是 '/',将状态更改为 "单斜杠".
            否则,将当前字符添加到注释内容中.
            
        如果状态是 "单星":
            如果下一个字符是 '/', 将当前字符和下一个字符添加到注释内容中,并调用 make_comment 函数.
            如果下一个字符是 '*',将当前字符添加到注释内容中,并将状态更改为 "块注释".
            否则,将当前字符添加到注释内容中,并将状态更改为 "块注释".
            
        如果状态是 "行内注释":
            如果当前字符是 '\n' 或者已经到达了输入字符串的末尾:
                如果当前字符不是 '\n',将它添加到注释内容中.
                调用 make_comment 函数.
            否则,将当前字符添加到注释内容中.
            
        如果状态是 "引号":
            处理转义引号或反斜线的情况:
                如果当前字符是 '\\' 并且下一个字符是 '"', 则跳过这两个字符.
                如果当前字符是 '\\' 并且下一个字符是 '\\', 则跳过这两个字符.
                如果当前字符是 '"',则将状态更改为 "未开始".
                如果当前字符是 '\n',则抛出错误.
                
    返回一个新的 CommentExtrator 实例,其中包含提取出的所有注释.
```
首先检查输入字符串是否为空或只有一个字符(在这种情况下没有注释可提取),然后创建一个 State Machine 来跟踪当前处于哪种解析状态(例如:正在处理行内注释,块注释等).

该程序使用一种基于状态机的方法来逐个字符地扫描输入字符串以查找可能存在的所有类型的注释.根据当前的状态和读取到的下一个字符,程序会做出不同的反应并更新当前状态.

当遇到可能是一个新的注释开头时,程序会在指定的位置记录下这个注释开始的位置,并从该位置开始收集注释的内容.

程序还引入了一个叫做 depth 的变量来帮助识别嵌套在其他代码段内的多层块注释,并确保正确地解析它们.

在整个过程结束后,程序会收集并返回已找到的所有注释对象(包括每个注释的位置信息和实际内容).


#### other functionality

1. `is_custom_comment(comment: &str) -> bool`: 判断给定的字符串 `comment` 是否符合自定义注释的格式.自定义注释的格式要求以 `//` 开头,后面紧跟一个非字母数字字符或非空白字符.

2. `custom_opener(s: &str) -> &str`: 从输入字符串 `s` 中提取第一行的开头部分,直到遇到第一个空格字符为止.如果输入字符串为空或没有空格字符,则返回一个空字符串.

3. `trim_end_unless_two_whitespaces(s: &str, is_doc_comment: bool) -> &str`: 去除字符串 `s` 的尾部空格,除非它们由两个或更多空格组成.

4. `left_trim_comment_line(line: &str, style: &CommentStyle<'_>) -> (&str, bool)`: 对给定的注释字符串进行左对齐处理,并返回是否删除了前导空格.

5. `find_uncommented(pat: &str) -> Option<usize>` 和 `find_last_uncommented(pat: &str) -> Option<usize>`: 在字符串中查找未被注释的子字符串,并返回其起始位置.

6. `contains_comment(text: &str) -> bool`: 判断给定的字符串 `text` 是否包含任何注释.

7. `find_comment_end(s: &str) -> Option<usize>`: 在字符串 `s` 中找到第一个注释之后的位置,并返回其字节位置.

8. `CharClasses`: 一个迭代器,用于区分代码中的功能性部分和注释部分.

9. `LineClasses`: 一个迭代器,用于遍历字符串中的功能代码和注释部分,返回每行的字符类型.

10. `UngroupedCommentCodeSlices`: 一个迭代器,用于遍历注释中的代码片段,将其分开.

11. `CommentCodeSlices`: 一个迭代器,用于在字符串中迭代出功能部分和注释部分的子串.

12. `filter_normal_code(code: &str) -> String`: 过滤掉给定代码字符串中的注释,返回只包含功能代码的字符串.

13. `CommentReducer`: 一个迭代器,用于遍历注释中的有效字符.

14. `consume_same_line_comments`: 处理同一行里的多个注释.


### TokenTree
Simplify the AST into a much simpler tree type, which we refer to as TokenTree.
```rust
function(a) {
    if (a) { 
        return 1
    } else {
        return 2
    }
}
```

`TokenTree` mainly contains two category.

* `SimpleToken` in previous code snippet,`if`,`return`,`a` are `SimpleToken`.
* `Nested` in previous code snippet, paired `()` and paired `{}` will form a `Nested` Token.

So a source program may represents like this.

```rust
pub enum TokenTree {
    SimpleToken {
        content: String,
        pos: u32,  // start position of file buffer.
    },
    Nested {
        elements: Vec<TokenTree>,
        kind: NestKind,
    },
}

pub type AST = Vec<TokenTree>;
```

Instead of dealing a lot data structure we simple the puzzle to dump `Vec<TokenTree>`. `TokenTree` is just another very simple version of `AST`.

`TokenTree` is very easy to parse,simple as.
```rust
...
if(tok == Tok::LParent){ // current token is `(`
    parse_nested(Tok::LParent);    
}
...
```

#### Handling ambiguity
Right now verything looks fine. But There are some token can be both `SimpleToken` and `Nested`. 
Typical for a language support type parameter like `Vec<X>`.

A Token `<` can be `type parameter sign` or `mathematic less than`. This can be solved by consult the `AST` before parse `TokenTree`.

Because we are writting a formatter for exist programming language. It is always easy for us to get the real `AST`. We can traval the `AST` the decide `<` is either a `SimpleToken` or `Nested`.

### Config
1.indent_size default been 2, user can set 2 or 4 by two ways:

1).command's parameter in terminal

2).vs-plugin seeting page, we'll integrate it into the aptos move analyzer later on


2.Users can enter -d on the terminal to format the move project to which the current directory belongs.
And enter -f on the terminal to format the specified single move file.


```rust
pub struct FormatConfig {
    pub indent_size: usize,
}
```

### FormatContext
1.The FormatEnv structure marks which syntax type is currently being processed.

2.The FormatContext structure holds the content of the move file being processed.
```rust
pub enum FormatEnv {
    FormatUse,
    FormatStruct,
    FormatExp,
    FormatTuple,
    FormatList,
    FormatLambda,
    FormatFun,
    FormatSpecModule,
    FormatSpecStruct,
    FormatSpecFun,
    FormatDefault,
}

pub struct FormatContext {
    pub content: String,
    pub env: FormatEnv,
}
  
impl FormatContext {
    pub fn new(content: String, env: FormatEnv) -> Self {  
        FormatContext { content, env }
    }

    pub fn set_env(&mut self, env: FormatEnv) {  
        self.env = env;  
    }  
}
```

### Format main idea
格式化某个文件的入口是 `Format::format_token_trees()` 函数,在内部遍历 `token_tree`, 
并根据当前 `TokenTree::Nested` 信息实时更新 `FormatContext` ,然后根据 `FormatContext` 进行处理.

在更深层次,我们会根据不同的 `FormatEnv`, 有不同的 `rewrite trait` 进行重写操作.

其中在每一个FormatContext场景,每处理一个 `TokenTree::SimpleToken`, 都会搜索并处理其前面的注释信息,对注释做一次局部处理;
每遇到代码块结束,即'}'符号,会对整个代码块做注释的一次全局处理.

Here are some internal interfaces of the module: 

1).`same_line_else_kw_and_brace` 用于判断某个字符串是否与 else 关键字和后面的大括号在同一行.

2).`allow_single_line_let_else_block` 用于判断 let 语句和 else 语句是否可以在同一行.

3).`single_line_fn` 判断函数是否可以单行展示.

4).`rewrite_fn_base` 重写函数头.

5).`rewrite_params` 重写函数的参数列表.

......



### Overall process 
`Vec<TokenTree>` is a tree type, It is very easy to decide how many ident,etc. And comment can pour into `result` base on the `pos` relate to `SimpleToken`.`pos`.

eg: format a single move file.
```rust
    let content = content.as_ref();
    let attrs: BTreeSet<String> = BTreeSet::new();
    let mut env = CompilationEnv::new(Flags::testing(), attrs);
    let filehash = FileHash::empty();
    let (defs, _) = parse_file_string(&mut env, filehash, &content)?;
    let lexer = Lexer::new(&content, filehash);
    let parse = crate::token_tree::Parser::new(lexer, &defs);
    let parse_result = parse.parse_tokens();
    let ce = CommentExtrator::new(content).unwrap();
    let mut t = FileLineMappingOneFile::default();
    t.update(&content);

    let f = Format::new(config, ce, t, parse_result, 
        FormatContext::new(content.to_string(), FormatEnv::FormatDefault));
    f.format_token_trees();
```

steps:

1).Call `parse_file_string` in move-compiler function to obtain the original AST of this move file.

2).Call `parse.parse_tokens()` to obtain `Vec<TokenTree>`.

3).Call `CommentExtrator` to obtain `Vec<Comment>`.

4).Call `format_token_trees` to obtain `String` which contains formatted file content.
