# Files-to-Prompt-rs
This tool concatenates numerous files within a directory into a single prompt, optimised for usage with LLMs. Notably, it is developed in Rust and exhibits high performance behaviour

You can refer here for the original repository: https://github.com/simonw/files-to-prompt

Usage: `gpt-review-code [OPTIONS]`

Options are as follows:
  - `-d, --dir-path <DIR_PATH>` : Directory path to concatenate the files from. Default value is '.'.
  - `-e, --file-patterns <FILE_PATTERNS>` : Specify file patterns to include. Defaults are txt, py, c, go, rs, java, js, html, and ts.
  - `-o, --output-file <OUTPUT_FILE>` : Specify the output file name.
  - `-m, --mode <MODE>` : State the mode, default is 'stdout'.
  - `-l, --lines <LINES>` : Limit the number of lines from the file to add. Default value is -1 which means get all lines from the file.
  - `-h, --help` : Display the help instructions.
  - `-V, --version` : Print the version of the program.


Example Usage:

`./target/release/gpt-review-code -d $(pwd) -e "rs,json" -l 50`
`./target/release/gpt-review-code -d $(pwd) -e "rs,json" -l 50 | llm -s "Explain this sources code and show results in the table" -m gemini-1.5-pro-latest`

Build:

cargo build --release