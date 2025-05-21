**movefmt** is a formatting tool that is based on the Aptos Move compiler. 

## About Us
MoveBit is a security audit company for the Move ecosystem, with a vision to make the Move ecosystem the most secure Web3. 

The MoveBit team consists of security leaders in academia and enterprise world, with 10 years of security experience, and is the first blockchain security company to leverage formal verification in the Move ecosystem.

## Background
A formatting tool, also known as a pretty-printer, prints the parser's AST of the corresponding language into a beautifully formatted string.


Currently, the community has been discussing the need for a formatting tool for the move language, but due to the fact that implementing a formatting tool is not an easy task, there has not yet been a mature and user-friendly formatting tool for the move language.


Based on this, MoveBit developed a move formatting tool for this purpose. It currently supports the formatting of Move code in Aptos Move projects.


## Build

**movefmt** requires Rust compiler to build. From the root directory, execute the following command.

```
$ git clone https://github.com/movebit/movefmt.git
$ cd movefmt
$ git checkout develop
$ cargo build
```

The resulted binary `movefmt` can be found under the directory `target/debug`.

## Install

Run the following command to install `movefmt`.

```
$ cargo install --git https://github.com/movebit/movefmt --branch develop movefmt
```

On MacOS and Linux, `movefmt` is typically installed in directory `~/.cargo/bin`.
Ensure to have this path in your `PATH` environment variable so `movefmt` can be executed from any location.
This step can be done with the below command.

```
$ export PATH=~/.cargo/bin:$PATH
```

## Usage
If you wish to use this tool independently.
```
# set env variable to see the log
export MOVEFMT_LOG=movefmt=DEBUG

# get help msg
movefmt -h

# format source file with printing verbose msg
movefmt -v /path/to/your/file_name.move
```
More usage you can see at:
> https://github.com/movebit/movefmt/blob/develop/doc/how_to_use.md

Alternatively, you can easily use the vscode plugin **aptos-move-analyzer** by installing it. We have integrated **movefmt** into it, which allows you to format the current move file with just one right-click. The VScode plugin **aptos-move-analyzer** is installed on the plugin market page with detailed guidance.
> https://marketplace.visualstudio.com/items?itemName=MoveBit.aptos-move-analyzer

## Github CI

There are publicly available repository for the Move formatter workflow: https://github.com/movebit/movefmt-workflow.
It allows you to easily integrate `movefmt` checks by configuring a simple `.github/workflows` file.

## Pre-commit hooks

This project supports **pre-commit hooks**. You can easily run the hooks by adding a `.pre-commit-config.yaml` file to your project:

```yaml
repos:
  - repo: https://github.com/movebit/movefmt
    rev: v1.2.1  # or a newer version
    hooks:
      - id: movefmt
        args: ['--config-path', 'path/to/your/movefmt.toml']  # or additional command-line arguments
```

Then run:

```bash
pre-commit install
pre-commit run --all-files
```

## License

**movefmt**  is released under the open source [Apache License](LICENSE)
