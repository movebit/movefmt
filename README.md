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
> https://github.com/movebit/movefmt/blob/develop/doc/how_to_use.md

Alternatively, you can easily use the vscode plugin **aptos-move-analyzer** by installing it. We have integrated **movefmt** into it, which allows you to format the current move file with just one right-click. The VScode plugin **aptos-move-analyzer** is installed on the plugin market page with detailed guidance.
> https://marketplace.visualstudio.com/items?itemName=MoveBit.aptos-move-analyzer

## License

**movefmt**  is released under the open source [Apache License](LICENSE)
