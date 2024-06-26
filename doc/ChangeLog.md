# aptos movefmt ChangeLog

<!--lint disable maximum-line-length no-literal-urls prohibited-strings-->

<table>
<tr>
<th>Stable</th>
</tr>
<tr>
<td>
<a href="#v1.0.2">v1.0.2</a><br/>
<a href="#v1.0.1">v1.0.1</a><br/>
<a href="#v1.0.0">v1.0.0</a><br/>
<a href="#v1.0.0.beta">v1.0.0.beta</a><br/>
</td>
</tr>
</table>


<a id="v1.0.2"></a>

## 2024-6-20, Version v1.0.2

### Features
- Fixed a bug about tailing comment when break line on call's last parameter
- Fixed issue10 and issue11
- Optimize formatting for long exp
- Optimize formatting for branch without block
- Optimize indentation where parameters that are lambda block in function calls


### TODO
- Optimize formatting for complex exp
- Optimize formatting for big pragmas
- Optimize skipping code block in more scenarios

### Commits
* [[`9a32d39`](https://github.com/movebit/movefmt/commit/9a32d397e57a820abe2da65f8d82613a3bdb0250)] - add test case (edy)
* [[`95cd461`](https://github.com/movebit/movefmt/commit/95cd461309968a3c759fc59c67d2aa5dd999a454)] - fix issue11 for deleting last comma in fun_call with single line (edy)
* [[`fcdfa8c`](https://github.com/movebit/movefmt/commit/fcdfa8ce5b981b047bf55a72147ef7d1b9241973)] - optimize get_break_mode_begin_nested() (edy)
* [[`2223c0d`](https://github.com/movebit/movefmt/commit/2223c0d4ec6b65d9b23787dcf2d5b8e80ef70227)] - fix issue10 for fun call (edy)
* [[`d686c72`](https://github.com/movebit/movefmt/commit/d686c7270313e7ab0cba1f6e26b2445f28c28398)] - optimize formatting for branch without block; optimize let assign with branch (edy)
* [[`1c68b43`](https://github.com/movebit/movefmt/commit/1c68b43ae2039294bfe765817aedfe5c5c3d9965)] - opimize indent when lambda as a parameter within fun_call (edy)


<a id="v1.0.1"></a>

## 2024-6-7, Version v1.0.1

### Features
- Fixed some bugs about adding space
- Fixed issue7 and issue8
- Optimize line breaks in various scenarios
- Optimize indentation where parameters that are lambda block in function calls
- Optimize formatting performance, such as very huge vector
- Support skipping fun body by adding attribute `#[fmt::skip]`

### TODO
- Optimize indentation in more scenarios
- Optimize skipping code block in more scenarios

### Commits
* [[`60afc64`](https://github.com/movebit/movefmt/commit/60afc6404301ad05ad88f6c40ac302a395f279b3)] - Merge remote-tracking branch 'origin/fea/optimize_fun_call' into develop (edy)
* [[`c6884f3`](https://github.com/movebit/movefmt/commit/c6884f3b7f0bc1b54404da6d3094739ebf73ef41)] - Merge remote-tracking branch 'origin/fix/err_space2' into develop (edy)
* [[`87d3504`](https://github.com/movebit/movefmt/commit/87d35049896bb0fc7f88769ed106265863370ce3)] - optimize fmt performance for big vec[] (edy)
* [[`400b3aa`](https://github.com/movebit/movefmt/commit/400b3aa2fdd97645ab50968f7bfef50128d4dd9d)] - cargo fmt (edy)
* [[`a5d5501`](https://github.com/movebit/movefmt/commit/a5d55017f255a07c6c6b2798cece132444d77ae3)] - fix issue8 (edy)
* [[`86c188a`](https://github.com/movebit/movefmt/commit/86c188a4aa9d880b347457e5c6e97448102b5b8b)] - Fixed the issue where a * *b was incorrectly formatted as a ** b. (hapeeeeee)
* [[`ab3e3c2`](https://github.com/movebit/movefmt/commit/ab3e3c2d7e1568446cd20a92dcba3b0a17fef10d)] - add testunit for break line after last para in func call (hapeeeeee)
* [[`cc6ccc5`](https://github.com/movebit/movefmt/commit/cc6ccc56527179107e36d6e45a33f5be0ee7433e)] - optimize for break line and add comma after last para in fun call (hapeeeeee)
* [[`b8a1042`](https://github.com/movebit/movefmt/commit/b8a10425f4fe0b07e95ad6b9f8916be895b91aae)] - fixed issue7 (edy)
* [[`7e10ac0`](https://github.com/movebit/movefmt/commit/7e10ac0c8c4abc132364ff3818c2cf9babc5d975)] - optimize exp with multi '&&' or '||' (edy)
* [[`ab2931b`](https://github.com/movebit/movefmt/commit/ab2931b9d599d85fc09b70c104d78fe1669578b4)] - optimize break line about spec header's para_list (edy)
* [[`f0c70c2`](https://github.com/movebit/movefmt/commit/f0c70c2f24e4c81b840ec8bb688007809e440271)] - optimize indent where parameters that are lambda block in function calls (edy)
* [[`f78feba`](https://github.com/movebit/movefmt/commit/f78febaa9ad4eb0d514369559c3b46d443d43df3)] - a space before '@' and a space after return (hapeeeeee)
* [[`9312d91`](https://github.com/movebit/movefmt/commit/9312d910db5b34984d60b863ac05e230b1fa399e)] - add should_skip_this_fun_body() (edy)
* [[`ee8d740`](https://github.com/movebit/movefmt/commit/ee8d740ca2dc54ebb4ba35f072434152f0cae284)] - update changelog, add features and notes (edy)


<a id="v1.0.0"></a>

## 2024-5-17, Version v1.0.0

### Features
- Support new syntax { for loop; receiver style call }
- Support running movefmt without a target file
- Optimize line breaks in various scenarios
- Optimize multiple empty line folding
- Fixed some bugs

### Notes
We have formatted all the Move files in the aptos-core repository, and here are some records.
```
edy@edydeMBP-4 aptos-core % movefmt -v
no file argument is supplied, movefmt runs on current directory by default, 
formatting all .move files within it......

----------------------------------------------------------------------------

Current directory: "/Users/edy/workspace/movebit/aptos-core"
options = GetOptsOptions { quiet: false, verbose: true, config_path: None, emit_mode: None, inline_config: {} }
Formatting /Users/edy/workspace/movebit/aptos-core/crates/aptos/debug-move-example/sources/DebugDemo.move
Spent 0.004 secs in the parsing phase, and 0.002 secs in the formatting phase
Formatting /Users/edy/workspace/movebit/aptos-core/crates/aptos/src/move_tool/aptos_dep_example/pack2/sources/m.move
Spent 0.000 secs in the parsing phase, and 0.000 secs in the formatting phase
......
Formatting /Users/edy/workspace/movebit/aptos-core/api/src/tests/move/pack_exceed_limit/sources/exceed_limit.move
Spent 0.001 secs in the parsing phase, and 0.003 secs in the formatting phase
124 files skipped because of parse failed
3515 files successfully formatted
edy@edydeMBP-4 aptos-core % 
```

Out of the 3515 files, we have the following before and after formatting:
1. There are 57 files with more than 200 lines of difference or a total character difference exceeding 512.
2. There are 334 files with a difference in the number of lines between 20 and 200, and a total character difference less than 512.
3. There are 928 files with fewer than 20 lines of difference.
```
edy@edydeMBP-4 aptos-core % git diff --numstat --word-diff=porcelain | awk '
BEGIN {
    FS="\t"
}
# Calculate line diff and character diff for each file
{
    if (NR % 2 == 1) {
        # On odd lines, parse the diff output
        add = $1
        del = $2
        file = $3
    } else {
        # On even lines, parse the diff output
        split($0, arr, /[+-]/)
        total_chars = length(arr[1]) + length(arr[2])
        if ((200 <= add + del) || (total_chars >= 512)) {                     
            print file
        }
    }
}' |  wc -l
      57
edy@edydeMBP-4 aptos-core % git diff --numstat --word-diff=porcelain | awk '
BEGIN {
    FS="\t"
}
# Calculate line diff and character diff for each file
{
    if (NR % 2 == 1) {
        # On odd lines, parse the diff output
        add = $1
        del = $2
        file = $3
    } else {
        # On even lines, parse the diff output
        split($0, arr, /[+-]/)
        total_chars = length(arr[1]) + length(arr[2])
        if ((20 <= add + del) && (add + del <= 200) && (total_chars <= 512)) {
            print file
        }
    }
}' | wc -l 
     334
edy@edydeMBP-4 aptos-core % git diff --numstat --word-diff=porcelain | awk '
BEGIN {
    FS="\t"
}
# Calculate line diff and character diff for each file
{
    if (NR % 2 == 1) {
        # On odd lines, parse the diff output
        add = $1
        del = $2
        file = $3
    } else {
        # On even lines, parse the diff output
        split($0, arr, /[+-]/)
        total_chars = length(arr[1]) + length(arr[2])
        if (add + del <= 20) {
            print file
        }
    }
}' |  wc -l
     928
edy@edydeMBP-4 aptos-core % 
```


### Commits

* [[`a00c73fe`](https://github.com/movebit/movefmt/commit/a00c73fe4842e9eba30b038d744e0829a116bda4)] - do cargo fmt (robinlzw)
* [[`64839f36`](https://github.com/movebit/movefmt/commit/64839f36df83bd31864e1cb70ac38e17ef645303)] - fix ident problem when multi address or multi module in a move file; optimize branch_fmt (robinlzw)
* [[`a83a8a2a`](https://github.com/movebit/movefmt/commit/a83a8a2af2f55d5d28d806057037e400f0d619a2)] - optimize bind statement; update breaking line by bin_op_exp (robinlzw)
* [[`14ddbb29`](https://github.com/movebit/movefmt/commit/14ddbb29dcaec932f132388c1d5bd58b9fbc72b6)] - optimize let_fmt (robinlzw)
* [[`9a45658f`](https://github.com/movebit/movefmt/commit/9a45658ffb4b89d40f492221ea6da5c9f8dda65a)] - if nest_type is spec, should change line after '{' (robinlzw)
* [[`9d7b31ae`](https://github.com/movebit/movefmt/commit/9d7b31ae53f8f28f753c02fcb696c5242276350a)] - allow Tok::AtSign split line (robinlzw)
* [[`ce3a23c6`](https://github.com/movebit/movefmt/commit/ce3a23c6bc24365161ccd0cb44d14cf92f2f277e)] - optimize need_space() about ',' (robinlzw)
* [[`3d2a6c1c`](https://github.com/movebit/movefmt/commit/3d2a6c1c85addbe15a56e793f70eec377b49e67b)] - optimize need_space() about 'aborts_with', '*' (robinlzw)
* [[`45536fa5`](https://github.com/movebit/movefmt/commit/45536fa528943771c3fd54ce2a1dbfbaa190ffe7)] - add check fn_call's para num (robinlzw)
* [[`9ca65873`](https://github.com/movebit/movefmt/commit/9ca658736a254a5696676f95c8da7b498a90281c)] - for call_fn(), don't add new line for last para (robinlzw)
* [[`0d1a18a0`](https://github.com/movebit/movefmt/commit/0d1a18a0965a3558c0b0d0b34c57a424f4d7ccf6)] - optimize: fn header; call in spec (robinlzw)
* [[`00d53565`](https://github.com/movebit/movefmt/commit/00d53565715644e55693122f154cb4d2611d72b5)] - fix bug: no indent when multi module in address (robinlzw)
* [[`52e7abf8`](https://github.com/movebit/movefmt/commit/52e7abf8f8f233fef73509544770575541f659a4)] - opimize vec[] with too many elements (robinlzw)
* [[`c7a774b8`](https://github.com/movebit/movefmt/commit/c7a774b827d72f82d9288e3c07b724865cacccb4)] - optimize format_simple_token() (robinlzw)
* [[`6d510eed`](https://github.com/movebit/movefmt/commit/6d510eed6411428d13583e6392ba5049dbad2184)] - add need_break_cur_line_when_trim_blank_lines() (robinlzw)
* [[`e93211f9`](https://github.com/movebit/movefmt/commit/e93211f93e62e61a045cc7f782283865ff771e7c)] - add rust-toolcahin; do cargo fmt (robinlzw)
* [[`9d15920d`](https://github.com/movebit/movefmt/commit/9d15920d4ad3447c4a74ec327c30082f2cacc7b1)] - add let_fmt module (robinlzw)
* [[`35479993`](https://github.com/movebit/movefmt/commit/35479993aa25d127d244b074d493be0db170a590)] - optimize need_new_line() (robinlzw)
* [[`e7d05e9c`](https://github.com/movebit/movefmt/commit/e7d05e9cbf344ff535bb7fcc1e369227fba1e1c2)] - optimize adding space with Tok::Amp (robinlzw)
* [[`06a16ebc`](https://github.com/movebit/movefmt/commit/06a16ebc6c7828ebcb2ed88c76645f69f9dbdf52)] - optimize get_code_buf_len() (robinlzw)
* [[`30ba77be`](https://github.com/movebit/movefmt/commit/30ba77beda45101d0e017a2f2be93471b29f9fc1)] - optimize 'for' loop and 'in' (robinlzw)
* [[`6db0c59d`](https://github.com/movebit/movefmt/commit/6db0c59d8f1cee8cebd52a58391d4b48434a0e0c)] - optimize changing line where brace in 'while' paren (robinlzw)
* [[`88e7c989`](https://github.com/movebit/movefmt/commit/88e7c98963e3cc88525cf31483f0ba075af4d9f7)] - optimize ability check; which resulting in a line break error in the struct field (robinlzw)
* [[`a15b8f3b`](https://github.com/movebit/movefmt/commit/a15b8f3b2b461a42e5f0911034bd697ef0d62439)] - optimize break line about for_loop's brace (robinlzw)
* [[`797ab2eb`](https://github.com/movebit/movefmt/commit/797ab2eb43963f0a7f1e5f27a896da55713bc257)] - support fn link call (robinlzw)
* [[`9d9fe189`](https://github.com/movebit/movefmt/commit/9d9fe18919a90e99e39aa08eb05921afd51a0a1a)] - support Receiver call (robinlzw)
* [[`f0388f18`](https://github.com/movebit/movefmt/commit/f0388f1886508c587490f9064c03a31c5de6f4d0)] - support for loop (robinlzw)
* [[`b4754411`](https://github.com/movebit/movefmt/commit/b47544111b2ca4dfd014c2cc012bcc72a1963ec2)] - 1.support default formatting current dirctory; 2.optimize add_space bewteen '/' and '*'; 3.optimize bottom_half_before_kind_end(), add check for last '//'comment (robinlzw)
* [[`06f2daf9`](https://github.com/movebit/movefmt/commit/06f2daf949649e1b4f9f6b1fb1df64ab7620a26e)] - add check_logic_op for break line (robinlzw)
* [[`0381a5c9`](https://github.com/movebit/movefmt/commit/0381a5c9ca1e3c9e0a5bf86caa4f9136bfe16ca9)] - optimize indent after calling new_line_when_over_limits (robinlzw)
* [[`2bf7f82c`](https://github.com/movebit/movefmt/commit/2bf7f82ca12800f4af8796fe8d3c53f303a5695e)] - optimize break line in spec pragma (robinlzw)
* [[`a3e29fd6`](https://github.com/movebit/movefmt/commit/a3e29fd6f6b715962de1b65192f48d84d70641cf)] - optimize format_nested_token(); optimize break line within fun call (robinlzw)
* [[`b91150b6`](https://github.com/movebit/movefmt/commit/b91150b6a7f59d69a1b2fa0981ffd5fa156a68ba)] - optimize get_new_line_mode_begin_nested() (robinlzw)
* [[`a502614c`](https://github.com/movebit/movefmt/commit/a502614c0de751126eae46f2dcf4c90d37a8faee)] - optimize logic about add_space_around_brace (robinlzw)
* [[`d94da4a5`](https://github.com/movebit/movefmt/commit/d94da4a5e323435fec82a90b4ae06c6d5ed5a0a2)] - fix space err in: (TokType::Sign, TokType::Number), (invariant, TokType::Sign) (robinlzw)
* [[`28a55ff5`](https://github.com/movebit/movefmt/commit/28a55ff52096655e835e6ad748b0ee23a328716f)] - fix bug: '/' be deleted which located in block comment; optimize code: support nested block comment (robinlzw)
* [[`bcd850cb`](https://github.com/movebit/movefmt/commit/bcd850cb6d74e926bc1381ae8f5be8edc6b62b5c)] - add module: use_fmt.rs (robinlzw)
* [[`b8524cbe`](https://github.com/movebit/movefmt/commit/b8524cbed2c069ba0859c4738db1075f00987d0d)] - add test case for issue3 (robinlzw)

<a id="v1.0.0.beta"></a>

## 2024-3-29, Version v1.0.0.beta
