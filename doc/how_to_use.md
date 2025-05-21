
## command parameter 
### 1.common
1.1 get version of movefmt

`movefmt -V`

1.2 get help msg

`movefmt -h`

1.3 format source file

`movefmt /path/to/your/file_name.move`

1.4 format source file with printing verbose msg

`movefmt -v /path/to/your/file_name.move`


### 2.--emit
```rust
pub enum EmitMode {
    Overwrite,
    NewFile,
    Stdout,
    Diff,
}
```

2.1 overwrite the source file by (--emit "overwrite")

`movefmt --emit "overwrite" /path/to/your/file_name1.move /path/to/your/file_name2.move`

2.2 generate a new file named "xxx.fmt.out" by (--emit "new_file")

`movefmt --emit "new_file" /path/to/your/file_name1.move /path/to/your/file_name2.move`

2.3 print formatted content to stdout
 
 `movefmt -v --emit="stdout" /path/to/your/file_name.move`

2.4 check diff between origin source file and formatted content
 
 `movefmt -v --emit="diff" /path/to/your/file_name.move`


### 3.--config-path
eg:

`movefmt --config-path=./movefmt.toml -v /path/to/your/file_name.move`

### 4.--print-config
4.1 print default config

`movefmt --print-config default`

eg:

```
max_width = 90
indent_size = 4
hard_tabs = false
tab_spaces = 4
emit_mode = "Overwrite"
verbose = "Normal"
```


4.2 generate a default movefmt.toml config file under current path

`movefmt --print-config default movefmt.toml`


4.3 print current config in movefmt.toml

`movefmt --print-config current movefmt.toml`

eg:

```
max_width = 90
indent_size = 4
hard_tabs = false
tab_spaces = 2
emit_mode = "NewFile"
verbose = "Normal"
```

### 5.--config
eg:

`movefmt --config max_width="90",indent_size="4" -v --emit="stdout" /path/to/your/file_name.move`
or
`movefmt --emit=diff --config=indent_size=2,max_width=20 /path/to/your/file_name.move`

### 6.--file-path
Format the full path of the specified Move file.
eg:

`movefmt --emit="new_file" --file-path=/absolute/path/to/your/input1.move`

### 7.--dir-path
Format all Move files in the specified directory.
eg:

`movefmt --emit="new_file" --dir-path=/absolute/path/to/your/sources`

### 8.example
#### case1: Format two move files, separated by spaces.
```bash
edy@edydeMBP-4 movefmt % export MOVEFMT_LOG=movefmt=WARN
edy@edydeMBP-4 movefmt % movefmt -v tests/issues/issue2/input1.move tests/issues/issue3/input1.move
options = GetOptsOptions { quiet: false, verbose: true, config_path: None, emit_mode: None, inline_config: {} }
Using movefmt local config file /Users/edy/workspace/movebit/movefmt/movefmt.toml for /Users/edy/workspace/movebit/movefmt/tests/issues/issue2/input1.move
2024-03-29T07:45:26.569253Z ERROR movefmt: file '"/Users/edy/workspace/movebit/movefmt/tests/issues/issue2/input1.move"' skipped because of parse not ok
Using movefmt local config file /Users/edy/workspace/movebit/movefmt/movefmt.toml for /Users/edy/workspace/movebit/movefmt/tests/issues/issue3/input1.move
Spent 0.000 secs in the parsing phase, and 0.001 secs in the formatting phase
edy@edydeMBP-4 movefmt % 
```

#### case2: Format with option --file-path or --dir-path
```bash
edy@edydeMBP-4 movefmt % movefmt --emit="new_file" --file-path=tests/complex4/input1.move -v
2024-07-16T09:31:14.237365Z  WARN movefmt: 
            Currently movefmt is still in the beta testing phase.
            The formatting results of the beta version may be incompatible with the official release version.
        
options = GetOptsOptions { quiet: false, verbose: true, config_path: None, emit_mode: Some(NewFile), inline_config: {}, src_file_path: Some("tests/complex4/input1.move"), src_dir_path: None }
Using movefmt local config file /Users/edy/workspace/movebit/movefmt/movefmt.toml for tests/complex4/input1.move
Formatting tests/complex4/input1.move
Spent 0.001 secs in the parsing phase, and 0.001 secs in the formatting phase
1 files successfully formatted
edy@edydeMBP-4 movefmt %


edy@edydeMBP-4 movefmt %
edy@edydeMBP-4 movefmt % movefmt --emit="new_file" --dir-path=tests -v
2024-07-16T09:31:34.969202Z  WARN movefmt: 
            Currently movefmt is still in the beta testing phase.
            The formatting results of the beta version may be incompatible with the official release version.
        
options = GetOptsOptions { quiet: false, verbose: true, config_path: None, emit_mode: Some(NewFile), inline_config: {}, src_file_path: None, src_dir_path: Some("tests") }
Using movefmt local config file /Users/edy/workspace/movebit/movefmt/movefmt.toml for tests/complex/input1.move
Formatting tests/complex/input1.move
Spent 0.000 secs in the parsing phase, and 0.000 secs in the formatting phase
# ...
Formatting tests/issues/issue1/input2.move
Spent 0.000 secs in the parsing phase, and 0.000 secs in the formatting phase
1 files skipped because of parse failed
126 files successfully formatted
edy@edydeMBP-4 movefmt % 

```

#### case3: set environment variables on windows
```bash
# by powershell
$env:MOVEFMT_LOG = "movefmt=DEBUG"

# by cmd
set MOVEFMT_LOG=movefmt=DEBUG

# by git bash
export MOVEFMT_LOG=movefmt=DEBUG
```
