use argh::FromArgs;

# Files-to-Prompt-rs

This tool concatenates numerous files within a directory into a single prompt, optimised for usage with LLMs. Notably, it is developed in Rust and exhibits high performance behaviour

You can refer here for the original repository: https://github.com/simonw/files-to-prompt

Usage: `files-to-prompt [OPTIONS]`

Options are as follows:

- `-d, --dir-path <DIR_PATH>` : Directory path to concatenate the files from. Default value is '.'.
- `-e, --file-patterns <FILE_PATTERNS>` : Specify file patterns to include. Defaults are txt, py, c, go, rs, java, js, html, and ts.
- `-o, --output-file <OUTPUT_FILE>` : Specify the output file name.
- `-m, --mode <MODE>` : State the mode, default is 'stdout'.
- `-l, --lines <LINES>` : Limit the number of lines from the file to add. Default value is -1 which means get all lines from the file.
- `-f, --files <Files>` : Support multiple files to concatenate. Default value is empty. Example: -f file1 -f file2
- `-h, --help` : Display the help instructions.
- `-V, --version` : Print the version of the program.

Example Usage:

`./target/release/files-to-prompt -d $(pwd) -e "rs,json" -l 50`

`./target/release/files-to-prompt -d $(pwd) -e "rs,json" -l 50 | llm -s "Explain this sources code and show results in the table" -m gemini-1.5-pro-latest`

`./target/release/files-to-prompt -d mistral.rs/ -e "rs,json" -l 50`
![alt text](images/image.png)

Support multiple files:
`./target/release/files-to-prompt -f Cargo.toml -f README.md`
![alt text](images/image-1.png)

Build:
cargo build --release
