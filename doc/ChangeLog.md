# aptos movefmt ChangeLog

<!--lint disable maximum-line-length no-literal-urls prohibited-strings-->

<table>
<tr>
<th>Stable</th>
</tr>
<tr>
<td>
<a href="#v1.4.3">v1.4.3</a><br/>
<a href="#v1.4.2">v1.4.2</a><br/>
<a href="#v1.4.1">v1.4.1</a><br/>
<a href="#v1.4.0">v1.4.0</a><br/>
<a href="#v1.3.9">v1.3.9</a><br/>
<a href="#v1.3.8">v1.3.8</a><br/>
<a href="#v1.3.7">v1.3.7</a><br/>
<a href="#v1.3.6">v1.3.6</a><br/>
<a href="#v1.3.5">v1.3.5</a><br/>
<a href="#v1.3.4">v1.3.4</a><br/>
<a href="#v1.3.3">v1.3.3</a><br/>
<a href="#v1.3.2">v1.3.2</a><br/>
<a href="#v1.3.1">v1.3.1</a><br/>
<a href="#v1.3.0">v1.3.0</a><br/>
<a href="#v1.2.9">v1.2.9</a><br/>
<a href="#v1.2.8">v1.2.8</a><br/>
<a href="#v1.2.7">v1.2.7</a><br/>
<a href="#v1.2.6">v1.2.6</a><br/>
<a href="#v1.2.5">v1.2.5</a><br/>
<a href="#v1.2.4">v1.2.4</a><br/>
<a href="#v1.2.3">v1.2.3</a><br/>
<a href="#v1.2.2">v1.2.2</a><br/>
<a href="#v1.2.1">v1.2.1</a><br/>
<a href="#v1.2.0">v1.2.0</a><br/>
<a href="#v1.0.9">v1.0.9</a><br/>
<a href="#v1.0.8">v1.0.8</a><br/>
<a href="#v1.0.7">v1.0.7</a><br/>
<a href="#v1.0.6">v1.0.6</a><br/>
<a href="#v1.0.5">v1.0.5</a><br/>
<a href="#v1.0.4">v1.0.4</a><br/>
<a href="#v1.0.3">v1.0.3</a><br/>
<a href="#v1.0.2">v1.0.2</a><br/>
<a href="#v1.0.1">v1.0.1</a><br/>
<a href="#v1.0.0">v1.0.0</a><br/>
<a href="#v1.0.0.beta">v1.0.0.beta</a><br/>
</td>
</tr>
</table>


<a id="v1.4.3"></a>
## 2025-11-28, Version v1.4.3
* [[`6e597b6`](https://github.com/movebit/movefmt/commit/6e597b6a50aef06effbf87865841157a8e0bd4cc)] - fix bug on normal brace block
* [[`703b839`](https://github.com/movebit/movefmt/commit/703b839b8e473f0c9a386f59cac271f404c3fa6a)] - optimize add_comments()
* [[`b9c37b0`](https://github.com/movebit/movefmt/commit/b9c37b08fc6aad60461d4eab7494c3227d423d50)] - optimize formatting on spec
* [[`a5f3f15`](https://github.com/movebit/movefmt/commit/a5f3f15bce726007fcaa7c85b37d6b6597964ddc)] - optimize formatting on inline fun
* [[`1573230`](https://github.com/movebit/movefmt/commit/157323053f4b2c05234b2fc229def5dadb475cb5)] - optmize formatting on comments before block token
* [[`cc4c2ab`](https://github.com/movebit/movefmt/commit/cc4c2abe16fc639e7f1ef26b176ffcc2e57b62b6)] - optimize format_simple_token()
* [[`eab1d2e`](https://github.com/movebit/movefmt/commit/eab1d2eaeec9e0df891d7a32bd6a766208528bad)] - optimize big block token when it's on the same line as the previous ending token
* [[`1ff6ee8`](https://github.com/movebit/movefmt/commit/1ff6ee888e5c247cd3837713fbb924e59de5322c)] - remove big_block_fmt


<a id="v1.4.2"></a>
## 2025-11-21, Version v1.4.2
* [[`56c87ff`](https://github.com/movebit/movefmt/commit/56c87ff560e87e0bd2a8985cedc753da1052c8a6)] - optimize process_blank_lines_before_simple_token_v2()
* [[`84cb0ea`](https://github.com/movebit/movefmt/commit/84cb0ea0e33d54c9e1b090a85077d6b03dc13d18)] - fix bug on skip block, try replace big_block_fmt with process_blank_lines_before_simple_token_v2()
* [[`0f2f294`](https://github.com/movebit/movefmt/commit/0f2f294d0265b1d0d5d47d36b7e0be6975525533)] - optimize process_blank_lines_before_simple_token(), especially regarding having multiple modules in one move file
* [[`404c6ec`](https://github.com/movebit/movefmt/commit/404c6ece17bca2a90201d037a2bb0bce0ef3ca3f)] - optimize format_token_trees()
* [[`816c48a`](https://github.com/movebit/movefmt/commit/816c48a9829104b465fb34932886a6896d08e00c)] - optimize core logic
* [[`8d19c64`](https://github.com/movebit/movefmt/commit/8d19c647f7722acbf4148b341eada572d6b8aa49)] - optimize utils
* [[`3379f7e`](https://github.com/movebit/movefmt/commit/3379f7eedf82daec1e670545bb3b56f41edb68f2)] - optimize tune_module_buf()


<a id="v1.4.1"></a>
## 2025-11-14, Version v1.4.1
* [[`f8a5a38`](https://github.com/movebit/movefmt/commit/f8a5a38b7636f2b59d55b14956cfe737fcd72849)] - update aptos-core dependence 
* [[`1064211`](https://github.com/movebit/movefmt/commit/106421112539e4f925e23a49ee6c197b374cef40)] - optimize get_break_mode_of_fun_call() 
* [[`98173b7`](https://github.com/movebit/movefmt/commit/98173b7a8d0f48c43c099f31903adf04b5c3fd0d)] - improve formatting on Bracket and Lambda Nested 
* [[`ede8e2a`](https://github.com/movebit/movefmt/commit/ede8e2a6567598d00804d3eba88abef2b18e17d5)] - optimize get_break_mode_begin_paren() 
* [[`160d1d5`](https://github.com/movebit/movefmt/commit/160d1d5c9145e5ff7a7ceb39970da61280e1446e)] - optimize get_kind_len_after_trim_space() 


<a id="v1.4.0"></a>
## 2025-11-07, Version v1.4.0
* [[`349d06f`](https://github.com/movebit/movefmt/commit/349d06f7cec81134e8d7b7494df13eb8ad5c59d9)] - optimize get_break_mode_begin_paren()
* [[`12dd0ce`](https://github.com/movebit/movefmt/commit/12dd0cec0e500daed8d784728704f2d1d592a5af)] - optimize get_break_mode_begin_nested()
* [[`7184fb9`](https://github.com/movebit/movefmt/commit/7184fb9919b4e7ac396153015ec88673eebb1e76)] - optimize format_dot_exp_chain()
* [[`ee9d74a`](https://github.com/movebit/movefmt/commit/ee9d74acd5fc9a084464698db6caaa849656f2f6)] - optimize process_fn_header()
* [[`b2fc6a1`](https://github.com/movebit/movefmt/commit/b2fc6a1d6e0354b19cfb2cbdb32dedd67237c71d)] - optimize fmt_big_block()
* [[`aaa4fb3`](https://github.com/movebit/movefmt/commit/aaa4fb3ec054ecd741a9b5adc7b4ebd991c17c4a)] - optimize code
* [[`3abcd9b`](https://github.com/movebit/movefmt/commit/3abcd9b3312058eb489888fad6347edddd5beb44)] - fix bug #86: crash on formatting nested closure
* [[`77518a5`](https://github.com/movebit/movefmt/commit/77518a50e0c9b2d15f5e371edf202f3e131bec69)] - optimize token_tree


<a id="v1.3.9"></a>
## 2025-10-27, Version v1.3.9
* [[`b3fa23c`](https://github.com/movebit/movefmt/commit/b3fa23c8890731997c243f9ca347ebd075a5d54c)] - fix bug on Exp_::Quant
* [[`8a9276b`](https://github.com/movebit/movefmt/commit/8a9276be038171e179bdba19b8051ad83e345f6a)] - add test case about signed_int; supporting new syntax(signed int)


<a id="v1.3.8"></a>
## 2025-10-24, Version v1.3.8
* [[`48c6a4d`](https://github.com/movebit/movefmt/commit/48c6a4d032ff217ffcc3e581940df82a6ad6e92e)] - fix potential bugs, update aptos-core dependence
* [[`51a3789`](https://github.com/movebit/movefmt/commit/51a3789a871a64381a02ba53e6ed085065ef42ed)] - optimize check_cur_token_is_long_bin_op()
* [[`d2de508`](https://github.com/movebit/movefmt/commit/d2de5084d087055b77f13d849c3481af704f3a3a)] - optimize need_new_line_after_cur_tok_finished()
* [[`3bd1c9b`](https://github.com/movebit/movefmt/commit/3bd1c9b68aae909d6edd20c09bb25cfd719409f3)] - optimize the processing about access specifier
* [[`be023c5`](https://github.com/movebit/movefmt/commit/be023c5acf43575dda9e5e993d9af2408fb5bd26)] - remove fmt_fun(), optimize code
* [[`d86c1bf`](https://github.com/movebit/movefmt/commit/d86c1bf600c461060d432cfcadb7581c23b142f9)] - optimize get_break_mode_begin_paren()
* [[`e53a362`](https://github.com/movebit/movefmt/commit/e53a3620b0f2a9f1549a495117ee706265840618)] - optimize get_code_buf_len(); add process_fun_ret_ty()


<a id="v1.3.7"></a>
## 2025-10-17, Version v1.3.7
* [[`f1c7912`](https://github.com/movebit/movefmt/commit/f1c79123be047757632ac15ab4cd6d2cb9cc5a41)] - update aptos-core dependence
* [[`6dbb30b`](https://github.com/movebit/movefmt/commit/6dbb30b600420396f1e5a7a9c85d396b3e86d785)] - optimize warn log with color
* [[`9d124ce`](https://github.com/movebit/movefmt/commit/9d124ce2558a71cd006f79ffd9673709fc60a9c7)] - fix bug #85: format error on spec syntax
* [[`927c7b2`](https://github.com/movebit/movefmt/commit/927c7b262faaed4d43eb18b5663637c67ce32fad)] - fix bug #84: format error on long Bind exp


<a id="v1.3.6"></a>
## 2025-10-09, Version v1.3.6
* [[`f450dbb`](https://github.com/movebit/movefmt/commit/f450dbbe89fdb4eb1106d03b09146f0968155c53)] - improve fmt_simple_token_core()
* [[`e7bfecb`](https://github.com/movebit/movefmt/commit/e7bfecb4abdc6887a6c4428a256abec37b5844e8)] - fix bug #83; improve fmt_simple_token_core()
* [[`bc49e98`](https://github.com/movebit/movefmt/commit/bc49e98ce73e52f8a558370acbd7117217a18188)] - add is_fun_return_colon(); remove process_fun_header_too_long()


<a id="v1.3.5"></a>
## 2025-09-25, Version v1.3.5
* [[`e75ba33`](https://github.com/movebit/movefmt/commit/e75ba3362a140361eb1ad5d6d1c1095ffd7a36f0)] - optimize process_fun_header_too_long()
* [[`5db94a8`](https://github.com/movebit/movefmt/commit/5db94a804d0fccfea7840aeead8c28e59be119c8)] - optimize process_fun_ret_ty()
* [[`5289b8c`](https://github.com/movebit/movefmt/commit/5289b8c4cbde3d1e0a7b39d935d5380d5d9826da)] - optimize process_block_comment_before_fun()
* [[`790e86a`](https://github.com/movebit/movefmt/commit/790e86a7c816b6259b94c51698e78f84c7291851)] - remove DotChainParserV1


<a id="v1.3.4"></a>
## 2025-09-19, Version v1.3.4
* [[`fa09168`](https://github.com/movebit/movefmt/commit/fa091684f5724a7203fa9bbbff3c8fb7c202afd4)] - remove format_dot_exp_chain_v1()
* [[`0f53dce`](https://github.com/movebit/movefmt/commit/0f53dce74b5fd05290113fe686146d890df2a25a)] - remove fun_header_specifier_fmt_original()
* [[`ab62c7b`](https://github.com/movebit/movefmt/commit/ab62c7b93cc6a7bad2a506aa963dc9be2644cb08)] - fix bug: issue #82 {multi-line fun specifier}
* [[`0fe3dbd`](https://github.com/movebit/movefmt/commit/0fe3dbdf074204fcbb8fde464664a8a04afa8b8c)] - optimize fun_header_specifier_fmt(), speed improved 60%


<a id="v1.3.3"></a>
## 2025-09-11, Version v1.3.3
* [[`2ce5415`](https://github.com/movebit/movefmt/commit/2ce54157b83c4bfbf728cd613b3fbf5c9e663816)] - fix bug #81 {abnormal behavior on short field chain} (robinlzw)
* [[`1fcb404`](https://github.com/movebit/movefmt/commit/1fcb404ba77c6f350d512f3b021534b9a8842018)] - add collect_specifier_args(), optimize fun_header_specifier_fmt() (robinlzw)
* [[`cb4b4f6`](https://github.com/movebit/movefmt/commit/cb4b4f666a8de79a7edcc29139d96d497de11621)] - add is_fun_specifiers() (robinlzw)
* [[`ebed1ba`](https://github.com/movebit/movefmt/commit/ebed1ba21fa52a209fe81d0b50d4cfab6e177835)] - remove process_fun_annotation() from fun_fmt (robinlzw)
* [[`70872d9`](https://github.com/movebit/movefmt/commit/70872d95a835133283e02571a21bd8ae19cc4a1e)] - remove link_call_exp_vec from call_fmt (robinlzw)
* [[`b121e6a`](https://github.com/movebit/movefmt/commit/b121e6ab77881d5a41dbfbe41c2bf6ded32d2c2a)] - add format_dot_exp_chain_v2() (robinlzw)
* [[`27a5fbf`](https://github.com/movebit/movefmt/commit/27a5fbf78035495dd0b1b59e23e8d9612eadd09d)] - add DotChainParserV2 (robinlzw)


<a id="v1.3.2"></a>
## 2025-09-05, Version v1.3.2
* [[`34db7d1`](https://github.com/movebit/movefmt/commit/34db7d1237450093e3f2b74a16ec0c91c114e720)] - adjust code structure (robinlzw)
* [[`e93617f`](https://github.com/movebit/movefmt/commit/e93617f981f1f05582507cb8d7f18403f08fc87c)] - fix bug #79 {complex mixed dot chain} (robinlzw)
* [[`fc8959a`](https://github.com/movebit/movefmt/commit/fc8959ad047ccb2812ab096fc9c4e2c99d18763e)] - add DotChainParser for expr_fmt (robinlzw)
* [[`c921b0c`](https://github.com/movebit/movefmt/commit/c921b0c0c65c46fc176063670da555b9ff224661)] - improve test_dot_link (robinlzw)


<a id="v1.3.1"></a>
## 2025-08-29, Version v1.3.1
* [[`a7e4f88`](https://github.com/movebit/movefmt/commit/a7e4f88567736953dc3aa34bfe741708ebb64268)] - fix bug: issue#80 (robinlzw)
* [[`62b6097`](https://github.com/movebit/movefmt/commit/62b60976f0d06aa7fce0cb26bfa3f15fa15c6a32)] - first try: fix issue #79 (robinlzw)
* [[`b152418`](https://github.com/movebit/movefmt/commit/b152418503c13234e546cf08b35386dcd3ab881b)] - optimize format_single_token() and format_nested_elements() (robinlzw)
* [[`94acfec`](https://github.com/movebit/movefmt/commit/94acfecc4dbfb980f108c3e7f3fbe26ef132801a)] - add format_dot_exp_chain() (robinlzw)
* [[`e189b15`](https://github.com/movebit/movefmt/commit/e189b15ce55f265f15cde3dcaa0315a885b2cbdb)] - optimize core; delete is_statement_start_token() (robinlzw)
* [[`4b05d4a`](https://github.com/movebit/movefmt/commit/4b05d4a61cf16076a6cf37e6117f1e493b1c239a)] - optimize process_fn_header() (robinlzw)
* [[`d5f8d7e`](https://github.com/movebit/movefmt/commit/d5f8d7e6acfee352eac721aa2ab94c6b6caf1286)] - optimize is_in_link_call() (robinlzw)
* [[`a7f48f7`](https://github.com/movebit/movefmt/commit/a7f48f73e0267897cc5b73563caae3623be7ed33)] - delete redundant loop in is_in_link_call() (robinlzw)
* [[`b0e0633`](https://github.com/movebit/movefmt/commit/b0e06333b224ce83ec4114661213c030fe85e187)] - add ut for bug #issue79 (robinlzw)


<a id="v1.3.0"></a>
## 2025-08-22, Version v1.3.0
* [[`72d1c70`](https://github.com/movebit/movefmt/commit/72d1c701b11d555ed9ece59c54158088ed56e6b9)] - optimize expr_fmt (robinlzw)
* [[`50d88ac`](https://github.com/movebit/movefmt/commit/50d88acc1b6e39fb41c64232df2bc1923949d785)] - add experimental module: fmt_state (robinlzw)
* [[`021fc6c`](https://github.com/movebit/movefmt/commit/021fc6c07284f951235d41a5be7356aa8158883b)] - optmize code (robinlzw)
* [[`7057bbd`](https://github.com/movebit/movefmt/commit/7057bbdf18dcb3171d6a695261fa54ded401098c)] - optimize magic number, add is_statement_start_token() (robinlzw)
* [[`cace7ec`](https://github.com/movebit/movefmt/commit/cace7ecb74e35fc641989b12c1609c43afc584c2)] - update rustc edition (robinlzw)


<a id="v1.2.9"></a>
## 2025-08-15, Version v1.2.9
* [[`2626798`](https://github.com/movebit/movefmt/commit/2626798e088ed5c7a7bbe44796c2da48e401ed40)] - upgrade toolchain, remove experimental code, optimize app (robinlzw)
* [[`a75e8f1`](https://github.com/movebit/movefmt/commit/a75e8f17a5d0302c68403740c82452ea21292af1)] - optimize expr_fmt::nedd_space() (robinlzw)
* [[`fdf4934`](https://github.com/movebit/movefmt/commit/fdf493484d107f6bba4d108b71fd8ac84e0b9ad7)] - optimize format_nested_token() (robinlzw)


<a id="v1.2.8"></a>
## 2025-08-08, Version v1.2.8
* [[`84d43e2`](https://github.com/movebit/movefmt/commit/84d43e2c9e38ccad862b698eb5d1d14e10ee68af)] - update aptos-core dependence (robinlzw)
* [[`d251dc8`](https://github.com/movebit/movefmt/commit/d251dc882320789b71771f17f73fdba06e082c90)] - upgrade SyntaxHandler (robinlzw)
* [[`1e9f2c0`](https://github.com/movebit/movefmt/commit/1e9f2c0b6923409ff0c8d65c22919378249a45c2)] - add syntax_trait mod, first try improving by SyntaxHandlerV2 (robinlzw)
* [[`f712069`](https://github.com/movebit/movefmt/commit/f712069cadd5bfee47ecae21e91e82d1f45cd0f5)] - add checking on env::var(MOVEFMT_LOG) (robinlzw)
* [[`a03ae68`](https://github.com/movebit/movefmt/commit/a03ae68142372935eddb106fa1196262c40ec541)] - optimize SyntaxHandler (robinlzw)
* [[`a1f291c`](https://github.com/movebit/movefmt/commit/a1f291cfac5e9fa72ada735d73dc3760cf1a79b6)] - optimize preprocess() for syntax handler (robinlzw)
* [[`0b6ab32`](https://github.com/movebit/movefmt/commit/0b6ab32d8a6bdcbf8fdd95cb6ab4d4119097c9ef)] - optimize CallExtractor (robinlzw)


<a id="v1.2.7"></a>
## 2025-07-25, Version v1.2.7
* [[`344643e`](https://github.com/movebit/movefmt/commit/344643ed90713aa428c94db757f4d5c1a73f8c42)] - update aptos-core dependence (robinlzw)
* [[`036722d`](https://github.com/movebit/movefmt/commit/036722d2734c48c2e813a2dcab1e79e0c568cdd0)] - add option  for solving issue#77 (robinlzw)


<a id="v1.2.6"></a>
## 2025-07-15, Version v1.2.6
* [[`34f93b1`](https://github.com/movebit/movefmt/commit/34f93b10cdc25b02b2946ff50e805fbef69d3d3e)] - fix bug issue#76: wrong checking on chain call, leading to incorrect formatting (robinlzw)
* [[`88249ce`](https://github.com/movebit/movefmt/commit/88249cefb8478b4edc8902c94cf0a078e5c2dd2e)] - add UT for judging link_call (robinlzw)
* [[`97a0468`](https://github.com/movebit/movefmt/commit/97a04689461c85c3e19f1dfd7d5daac80c0796b5)] - remove useless log (robinlzw)
* [[`f267af2`](https://github.com/movebit/movefmt/commit/f267af291be90a803ec5090c0eb8c4b378b34760)] - add bug move file (robinlzw)


<a id="v1.2.5"></a>
## 2025-07-14, Version v1.2.5
* [[`23f49f5`](https://github.com/movebit/movefmt/commit/23f49f5fbe0361a0eaf3a0da9b8be4bf7f52a99f)] - fix issue#75: Optimize fn format_token_trees() (robinlzw)
* [[`b596127`](https://github.com/movebit/movefmt/commit/b596127fba348f7b05422fe6198fdaf80dd37213)] - optimize fn get_break_mode_begin_nested() (robinlzw)
* [[`b6726c0`](https://github.com/movebit/movefmt/commit/b6726c083b500e8f796e200479c9721c2f42f2c5)] - add fn get_pre_simple_tok() (robinlzw)
* [[`ce5f91a`](https://github.com/movebit/movefmt/commit/ce5f91a77280c9bcf96c0aa3d66b5954b36e9a24)] - optimize fn get_break_mode_begin_paren() (robinlzw)
* [[`08534f6`](https://github.com/movebit/movefmt/commit/08534f6dc77ef83f92d03658e7490bdaa067797b)] - first try: Optimize fn get_break_mode_begin_nested() (robinlzw)


<a id="v1.2.4"></a>
## 2025-07-08, Version v1.2.4
### Features
* [[`5a77082`](https://github.com/movebit/movefmt/commit/5a7708226265b5802ba86d9966be762547d8340b)] - update aptos-core dependence (robinlzw)
* [[`a5fe78e`](https://github.com/movebit/movefmt/commit/a5fe78e551a0b94bb6b0d66bf4333b5c26ac1c52)] - delete obsolete code (robinlzw)
* [[`552bf62`](https://github.com/movebit/movefmt/commit/552bf62430997191e45cd7cf29c09ae451ee0cf5)] - optimize fn judge_add_space_around_brace() (robinlzw)
* [[`367d94a`](https://github.com/movebit/movefmt/commit/367d94a8a566efb5056cd123f8ee2ec3c183a178)] - add fn need_skip_nested_token(); optimize fn format_nested_token() (robinlzw)
* [[`c912122`](https://github.com/movebit/movefmt/commit/c9121221353ebdbdc6c47b6228b85dec63c0499c)] - optimize skip_module (robinlzw)


<a id="v1.2.3"></a>
## 2025-06-27, Version v1.2.3
### Features
* [[`438977e`](https://github.com/movebit/movefmt/commit/438977e52925b238e8e5d16ebafb14a804e62e82)] - issue#73: supporting skip struct (robinlzw)
* [[`4420c91`](https://github.com/movebit/movefmt/commit/4420c917514fd47422dd1156b1a38d58fc5b6638)] - Optimize the skipping fun_body feature (robinlzw)
* [[`de8e138`](https://github.com/movebit/movefmt/commit/de8e138525a7e9828e713b7e85867e1236ba3e26)] - skip_fmt support Struct (robinlzw)
* [[`a777b4e`](https://github.com/movebit/movefmt/commit/a777b4e18a824928cb472dc54dd397e731df6ba1)] - config cross building on aarch64-apple-darwin (robinlzw)


<a id="v1.2.2"></a>
## 2025-06-17, Version v1.2.2
### Features
* [[`f1d314b`](https://github.com/movebit/movefmt/commit/f1d314bb0710b2ca1c5aa54c24c5059dfb3cd0b4)] - try to support mutli-threads for write files (hapeeeeee)
(hapeeeeee)
* [[`61d42d0`](https://github.com/movebit/movefmt/commit/61d42d0e41ae2db115cdab3bf2a866dc2b9adb97)] - generate token tree impl multi-threads (hapeeeeee)
* [[`f3770d0`](https://github.com/movebit/movefmt/commit/f3770d07e31bb27da1b8dd7c684876003787bdff)] - Add preprocess trait for syntax extractor (hapeeeeee)
* [[`e4f4381`](https://github.com/movebit/movefmt/commit/e4f43819227e137fe4194d9184600d7c532cf65b)] - dependence: add rayon for mutil-threads (hapeeeeee)
* [[`477d729`](https://github.com/movebit/movefmt/commit/477d729d7ce37fc62afc4bbf46922198b523fae9)] - optimize log about file argument and check option (robinlzw)
* [[`27c3c92`](https://github.com/movebit/movefmt/commit/27c3c924c48688afaac56b22007edc823e1722f5)] - fix issue#67: auto_apply_package recognize aptos project (robinlzw)
* [[`42683f2`](https://github.com/movebit/movefmt/commit/42683f2cdcf490d2c6b9ec52dae7610ff7227041)] - fix issue#62: return error code when fmt failed (robinlzw)

<a id="v1.2.1"></a>

## 2025-05-22, Version v1.2.1
### Features

Finish issue#39, issue#61

### Commits

* [[`4d82dbc`](https://github.com/movebit/movefmt/commit/4d82dbc9768ad12a1d5eb3e6e6d4fab536632696)] - do cargo fmt; add doc (robinlzw)
* [[`1aef607`](https://github.com/movebit/movefmt/commit/1aef607c4fd9f94536aea8b81b80808722c1824c)] - Add cmd-opt '-i' to avoid conflict between the 'receive-from-stdin' and 'pre-commit' features (robinlzw)
* [[`046c61a`](https://github.com/movebit/movefmt/commit/046c61ae95e9b2db6754b71eb5ae451f7c3c2ee7)] - update doc for `pre-commit-hooks` (hapeeeeee)
* [[`85589b8`](https://github.com/movebit/movefmt/commit/85589b8a49216fead88992b84e2d21e7ca789f4f)] - update doc for Github CI (hapeeeeee)
* [[`0de9044`](https://github.com/movebit/movefmt/commit/0de9044e40d1630a28d8f99963a873f540702255)] - add .pre-commit-hooks.yaml (hapeeeeee)
* [[`9f50959`](https://github.com/movebit/movefmt/commit/9f509591666bfaa6472b1d96b0e7a85a3c61a026)] - add pass files false (hapeeeeee)
* [[`9df8332`](https://github.com/movebit/movefmt/commit/9df8332a48e902f01e6fe8b7ac3f0205cb2652be)] - .pre-commit-hooks.yaml (hapeeeeee)
* [[`c8d44eb`](https://github.com/movebit/movefmt/commit/c8d44ebc17eb4a6efff945b76f919f565acb90de)] - first try on #issue61: receive code text from stdin (robinlzw)
* [[`e2db7fc`](https://github.com/movebit/movefmt/commit/e2db7fcd371c6e64677c99a3480027ea973c5c56)] - remove beta log; optimize warn log (robinlzw)
* [[`475b8b9`](https://github.com/movebit/movefmt/commit/475b8b92f5074ffc346f89f6aa1690d27863b059)] - optimize code and comment (robinlzw)

<a id="v1.2.0"></a>

## 2025-05-15, Version v1.2.0

### Bug

Fix Issue: movefmt.toml not work #59

Fix Issue: Verbose level not work in movefmt.toml #64

### Features

Close Issue: Add pre-commit support for CI use [[issue39](https://github.com/movebit/movefmt/issues/39)]

Close Issue: Use Semantic Versioning [[issue40](https://github.com/movebit/movefmt/issues/40)]

Issue: Support auto-discovery of directories containing Move.toml [[issue63](https://github.com/movebit/movefmt/issues/63)]


 - In `movefmt.toml`, set `auto_apply_package = true` to automatically detect and format all .move files that belong to a Move-Package in the specified directory (or current directory by default). Files that are not part of a Move-Package will be skipped.

Issue:movefmt for github actions CI check [[issue65](https://github.com/movebit/movefmt/issues/65)]

 - published a publicly available repository(https://github.com/movebit/movefmt-workflow) for the Move formatter workflow

### TODO
- Optimize code: Improve `SyntaxExtractor` trait
- Support skipping code block on struct

### Commits
* [[`45f2da9`](https://github.com/movebit/movefmt/commit/45f2da9dcabbd0bf43bd70c783a95ef24b3ace79)] - Add simple-project for test `auto_apply_package`
* [[`1e00711`](https://github.com/movebit/movefmt/commit/1e007111f2a773279f09e78ab879d9c12ecb67f4)] - Bug: File which from command line not be escaped
* [[`68e038c`](https://github.com/movebit/movefmt/commit/68e038c8f3ab552948790abb57f3a3ac694e8e67)] - Update aptos-core to lastest
* [[`cd53fec`](https://github.com/movebit/movefmt/commit/cd53fecf8d7668feefeddfc536f8010479c7a940)] - `Emit:diff`: support github CI log
* [[`2e7161e`](https://github.com/movebit/movefmt/commit/2e7161e12b80b578375cc2034df41686b1838ac1)] - Add TestCase for `auto_discover_project` in `movefmt.toml`
* [[`9e25ca3`](https://github.com/movebit/movefmt/commit/9e25ca3a1896bc0c30b750a82d54a038c9053609)] - Bug fix for issue64: `Verbose` level not work in `movefmt.toml`
* [[`853f035`](https://github.com/movebit/movefmt/commit/853f03540e7ce431a78d1956969ca4ad046e2b7a)] - Support auto-dicover-project feature
* [[`d618f88`](https://github.com/movebit/movefmt/commit/d618f888609cd22b997c03e2c511ce07a56ec557)] - Add format check GitHub Action


<a id="v1.0.9"></a>

## 2025-04-30, Version v1.0.9

### Features
- Feature: `movefmt.toml` add `skip_formatting_dirs` to skip all files in expected dir
- Fixed bug: issue#59

### TODO
- Optimize code: Improve `SyntaxExtractor` trait
- Support skipping code block on struct

### Commits
* [[`9c8ad8b`](https://github.com/movebit/movefmt/commit/2105b46cc6d32adf73212054e6e9778ba9c8ad8b)] - fix issue#59: `movefmt.toml` only work in `Verbose::Verbose`
* [[`0b2dbac`](https://github.com/movebit/movefmt/commit/41eebb40cbf82a62840dc2a514ae54c910b2dbac)] - feature: `movefmt.toml` add `skip_formatting_dirs` to skip all files in expected dir
* [[`51a29da`](https://github.com/movebit/movefmt/commit/4189ebc06d3cd89d0001a4189f0a3e61651a29da)] - optimize code; fix bug on should_escape()

<a id="v1.0.8"></a>

## 2025-04-03, Version v1.0.8

### Features
- Fixed bug: issue#46, issue#47, issue#48, issue#52
- Optimize code: Add `SyntaxExtractor` trait

### TODO
- Support skipping code block on struct

### Commits
* [[`cba1ba7`](https://github.com/movebit/movefmt/commit/cba1ba7c3ecda391d74da2b2217b5654fe5602f7)] - fix issue#46: The let statement may have one extra line break.
* [[`b56cda9`](https://github.com/movebit/movefmt/commit/b56cda99cf2e6a57b1a499fbcab3101abb85dd32)] - fix issue#47: [Bug] Compound assignment behavior
* [[`b56cda9`](https://github.com/movebit/movefmt/commit/cba1ba7c3ecda391d74da2b2217b5654fe5602f7)] - fix issue#48: [Bug] Line overflow
* [[`b56cda9`](https://github.com/movebit/movefmt/commit/4ba8cbbde9fb66eff2628bd03412f7e9143c3fc3)] - optize code: abstract syntax extractor trait


<a id="v1.0.7"></a>

## 2024-12-23, Version v1.0.7

### Features
- Fixed bug: issue#42, issue#43
- Optimize code: issue#44
- Upgrade aptos-core dependency
- Supported compound assignments([binop]=)

### TODO
- Support skipping code block on struct

### Commits
* [[`f7c2a0b`](https://github.com/movebit/movefmt/commit/f7c2a0bf15b905dc07f751bf3e9b8cfaeda1c53c)] - optimize UT
* [[`7ff7c83`](https://github.com/movebit/movefmt/commit/7ff7c83b953d1e7ff748d37a34fa9280a3c56de5)] - pass unit test
* [[`aa992d5`](https://github.com/movebit/movefmt/commit/aa992d51803a48d7da5f9576aef32b9b4ce82fc9)] - fix issue#45: add unit test for assign with binop
* [[`c26ecc3`](https://github.com/movebit/movefmt/commit/c26ecc3234acff66d5ae20e4c626a9ab29210957)] - fix issue#45: Update the Aptos version in Cargo.toml and ensure that movefmt compiles successfully.
* [[`f4df9fe`](https://github.com/movebit/movefmt/commit/f4df9fefd8f9916ef968ba4ab04c4353ab44a971)] - fix issue#44: fix bug -- No print options and format file result when -q or --quiet is specified
* [[`30c19eb`](https://github.com/movebit/movefmt/commit/30c19ebe946c1fa36741a933b1b85e67c57230fc)] - fix issue#43: fix bug -- indent errors when comments appear in if statements
* [[`06a6ccb`](https://github.com/movebit/movefmt/commit/06a6ccb543b7540c5dce3521e174f07f1344d8f3)] - optimize fn maybe_begin_of_if_else()
* [[`968d002`](https://github.com/movebit/movefmt/commit/968d00237d950b0d8f01820e4070289c97590a1e)] - update test case for issue43
* [[`a55548a`](https://github.com/movebit/movefmt/commit/a55548a9e1c1682062e69e778301584f7c03ed54)] - add test case for issue43
* [[`310493f`](https://github.com/movebit/movefmt/commit/310493f6e46fba58eb5bd4111c9b660506cba262)] - fix issue#42: bizarre ternary assignment outcomes
* [[`792be11`](https://github.com/movebit/movefmt/commit/792be112060ecb345d6ddb369245fd8eee291f8c)] - optmize need_space()


<a id="v1.0.6"></a>

## 2024-10-28, Version v1.0.6

### Features
- Optimize code
- Fixed bug #41

### TODO
- Support skipping code block on struct

### Commits
* [[`aeda535`](https://github.com/movebit/movefmt/commit/aeda5355d06b14d3924454eb87355d96f324c142)] - optimize expr_fmt (robin)
* [[`62b127d`](https://github.com/movebit/movefmt/commit/62b127d57ebca213d51e1762c2c21313202fe2f5)] - add test cases; optimize expr_fmt::need_space() (robin)
* [[`a1dabaa`](https://github.com/movebit/movefmt/commit/a1dabaaa76724e74be3310695e5ff5fc41873fc6)] - Update expr_fmt.rs (xiaozhang)
* [[`77fbcad`](https://github.com/movebit/movefmt/commit/77fbcad8a9cfc3d3610124c26be07963aa494511)] - fix issue#41: invalid removal of space for mut ref (xiaozhang)


<a id="v1.0.5"></a>

## 2024-9-20, Version v1.0.5

### Features
- Support new syntax 'enum'
- Fixed 2 issues{#35, #37}

### TODO
- Support skipping code block on struct
- Add cli.option{--package-path}

### Commits
* [[`ccff6a9`](https://github.com/movebit/movefmt/commit/ccff6a9e1c57458d58fd9d25b6759a1e3cb77874)] - optimize code (robin)
* [[`bf887e4`](https://github.com/movebit/movefmt/commit/bf887e4c64b1aee7639b9c9917a8e5204a7e6467)] - first try: optimize formatting about fun call (robin)
* [[`d22a79e`](https://github.com/movebit/movefmt/commit/d22a79ed6c7b858a04aca72e6b1304b76900d320)] - fix issue#37: fix bug -- wrong break line about generic type with empty pack body (robin)
* [[`939eb42`](https://github.com/movebit/movefmt/commit/939eb42601670874a5fe2ce52e8e6ecd622fba31)] - fix issue#36: support new syntax about enum (robin)
* [[`4716ba2`](https://github.com/movebit/movefmt/commit/4716ba26c9b7eb484f395471ef82a5eeffb60f26)] - first try: support new syntax about enum (robin)
* [[`7bb0fad`](https://github.com/movebit/movefmt/commit/7bb0fad27b1035ef721de86bd340c5dc2bf6767a)] - set_lang_v2(true), update test case (robin)
* [[`4751ac4`](https://github.com/movebit/movefmt/commit/4751ac473e1e8beb222cc3c5364b4a044c6e086b)] - upgrade aptos-core to latest; add test case about enum (robin)
* [[`4ed6a03`](https://github.com/movebit/movefmt/commit/4ed6a031a1f0cbc1c4f904bbbfcc3c966fe0e979)] - optimize tests on win (robin)


<a id="v1.0.4"></a>

## 2024-8-26, Version v1.0.4

### Features
- Fixed 6 bugs and 14 issues{#9, #13, #14, #16, [#19 ~ #27], #30}
- Optimize the output order of DIFF option

### TODO
- Support new syntax 'enum'
- Support skipping code block on struct
- Add cli.option{--package-path}

### Commits
* [[`066c799`](https://github.com/movebit/movefmt/commit/066c799440e4b51aeaeb7ba492c4ce573c79cb0c)] - improve performance on big vector (edy)
* [[`88248ec`](https://github.com/movebit/movefmt/commit/88248ec4e8d72f561bce89665676996c791ce6e9)] - fix bug: issue#34 (edy)
* [[`d799b34`](https://github.com/movebit/movefmt/commit/d799b3497ee9a3f96df3b56139985bf82708d633)] - fix bug: issue#33 (edy)
* [[`0a119f6`](https://github.com/movebit/movefmt/commit/0a119f6ff85d7e6d6dbc491ed876eaec29c54434)] - fix bug: issue#32 (rblzw)
* [[`b622ecc`](https://github.com/movebit/movefmt/commit/b622ecca9990965c37dfc5433b113d8db797d6b1)] - fix issue#25: optimize complex assign exp (rblzw)
* [[`e74bdf1`](https://github.com/movebit/movefmt/commit/e74bdf1804ecd20526e45b259bf27302603c7843)] - fix bug: issue#29 (edy)
* [[`5710ffd`](https://github.com/movebit/movefmt/commit/5710ffd5092d9d3d2204e277243be323c80d5490)] - fix issue#28 and issue#30 (edy)
* [[`99486e2`](https://github.com/movebit/movefmt/commit/99486e2bba68b82bf82abacc507c7abd9841d61f)] - fix issue#23: optimize complex big vector (rblzw)
* [[`9240377`](https://github.com/movebit/movefmt/commit/9240377a7783e07212e06306abbfe07398ba05a7)] - fix issue#22 (rblzw)
* [[`9bd032f`](https://github.com/movebit/movefmt/commit/9bd032f78ff65333589a1368948f40370a4f7ed5)] - fix issue#20: support option[prefer_one_line_for_short_call_para_list] in the movefmt.toml (edy)
* [[`fb07fa4`](https://github.com/movebit/movefmt/commit/fb07fa41b5e59ff3d4f75f146af38c262ff22e03)] - fix issue27; optimize break line about multi address{module{}} (edy)
* [[`25e509c`](https://github.com/movebit/movefmt/commit/25e509c0fe90ed88c4601a41c223b98441907137)] - first try: fix issue27 (edy)
* [[`bc4e888`](https://github.com/movebit/movefmt/commit/bc4e888b4cd281d08aca21692a81c74d8c4f9d44)] - fix issue26: wrong space added after 'apply' (edy)
* [[`e39d16a`](https://github.com/movebit/movefmt/commit/e39d16ad022fb92c4dcb96825ea36374d70b1e47)] - support option[prefer_one_line_for_short_fn_header_para_list] in the movefmt.toml (edy)
* [[`537ddf6`](https://github.com/movebit/movefmt/commit/537ddf60ed2fe3c921c8e1dd90163014c3f0ef2d)] - optimize: native fn; colon before fn's return_ty; <> in return_ty (edy)
* [[`ef0e47d`](https://github.com/movebit/movefmt/commit/ef0e47dc9602b84de6495ea91713cff4a8b7e04e)] - fix bug: add space bewteen '^' and '(/{' (edy)
* [[`90d6e83`](https://github.com/movebit/movefmt/commit/90d6e83837fea215879f750c6c6c33a68fbee654)] - fix issue#21 (edy)
* [[`ceaa220`](https://github.com/movebit/movefmt/commit/ceaa220698c544d363ef71570cb6f48d0ffad2af)] - fix issue#14 (edy)
* [[`434143d`](https://github.com/movebit/movefmt/commit/434143d85bd12db791e680d96d76d0421d933f84)] - optimize get_break_mode_begin_paren() and need_new_line_after_branch() (edy)
* [[`f73ec06`](https://github.com/movebit/movefmt/commit/f73ec0653a84349f1539ad016b7114aa34b21e56)] - fix bug: add space between '|' and '(/{' (edy)
* [[`bd9b665`](https://github.com/movebit/movefmt/commit/bd9b66509df170f6d9c2d8413363db044ca3eed1)] - optimize complex exp: first element is nested token_tree in () (edy)
* [[`862516b`](https://github.com/movebit/movefmt/commit/862516b7aa4e40845df66e0f215a44ec37ae84bf)] - fix issue#13 (edy)
* [[`1c37341`](https://github.com/movebit/movefmt/commit/1c3734109de6dafd59f2c68d0e79846352743c92)] - fix issue#19 (edy)
* [[`0726d23`](https://github.com/movebit/movefmt/commit/0726d235002f52514f47a2b2fd9394b8679274bb)] - adjust the output order of DIFF option (edy)
* [[`89ba14f`](https://github.com/movebit/movefmt/commit/89ba14f9e37c998774887b49d3a4176ae11a8ff5)] - optimize indentation (edy)
* [[`b8446e1`](https://github.com/movebit/movefmt/commit/b8446e117851f1655198ac74932f367c8e3d46dd)] - second try: optimize indentation (edy)
* [[`7381df4`](https://github.com/movebit/movefmt/commit/7381df48d5109a8eff96eb8edc1e05224192f3fd)] - first try: optimize indentation (rblzw)
* [[`aa0f861`](https://github.com/movebit/movefmt/commit/aa0f86101965a21ba7dcdfccc0e4d6ca02974f87)] - fixed #issue9: Long lines for pragmas (rblzw)


<a id="v1.0.3"></a>

## 2024-7-16, Version v1.0.3

### Features
- Fixed 3 bugs and issues{#15, #17, #18}
- Optimize formatting for complex exp
- Optimize error prompt
- Add config.option[prefer_one_line_for_short_branch_blk] and cli.option{--file-path, --dir-path}
- Update EmitMode

### TODO
- Optimize skipping code block in more scenarios
- Optimize issues{#9, #13, #14, #16, #19, #20, #21}

### Commits
* [[`37a14e6`](https://github.com/movebit/movefmt/commit/37a14e6aced7364cadb911c914f6337a1b86e51f)] - optimize error prompt (rblzw)
* [[`ba7784e`](https://github.com/movebit/movefmt/commit/ba7784e8c6a525f52dc7d875abb3365b27043030)] - Update EmitMode (edy)
* [[`d32b55f`](https://github.com/movebit/movefmt/commit/d32b55f89b14c13539c9c7282f245ee081e8ad2f)] - optimize exp break line (edy)
* [[`78d285a`](https://github.com/movebit/movefmt/commit/78d285a19153c5688bbd2751ff6d9d8bcbf83e8d)] - fix bug: add space when next_token is '*' or '&' (edy)
* [[`1a7c922`](https://github.com/movebit/movefmt/commit/1a7c922e013518a3baef97aab0059aec7858cfbf)] - fix bug: line break error when variable name same with ability (edy)
* [[`6e163fc`](https://github.com/movebit/movefmt/commit/6e163fceb493d46d358787f10fdfa87ea60715f3)] - optimize main.rs (qpzmV)
* [[`dfd5705`](https://github.com/movebit/movefmt/commit/dfd57058f524d47d2e3083c7e07fce829793e575)] - Update how_to_use.md (qpzmV)
* [[`0d15592`](https://github.com/movebit/movefmt/commit/0d1559215128552b104348e0ed74648803556b07)] - add option: --file-path, --dir-path (edy)
* [[`72950e3`](https://github.com/movebit/movefmt/commit/72950e32ebc6f33e52cc197641ab2d838a63faef)] - Add warn msg, currently in beta testing version (qpzmV)
* [[`4657588`](https://github.com/movebit/movefmt/commit/4657588a1f8aa026798635e7da37ce9fc44ef983)] - Update Cargo.toml (qpzmV)
* [[`ca29595`](https://github.com/movebit/movefmt/commit/ca29595713e54d0635bac3a3eb4add2277d83fa9)] - fixed #issue17: Optimize the formatting logic of complex expression (edy)
* [[`85a727d`](https://github.com/movebit/movefmt/commit/85a727d6ecac301228f9ff1d58adea98d56f003c)] - support option[prefer_one_line_for_short_branch_blk] in the movefmt.toml (edy)
* [[`3181b03`](https://github.com/movebit/movefmt/commit/3181b037e97a7c27879f3ef831773d8251c77de0)] - fixed bug[#issue15]: branch statement block contains comments (edy)


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
