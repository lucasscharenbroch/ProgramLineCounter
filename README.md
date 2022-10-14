# Program Line Counter
A console-based file-info-utility written in Rust.

## Features
- Filetypes, comment types, and directories entered by the user
- Automatic directory exploration
- Per-file and total counts of:
  - Program lines (any line with non-comment non-whitespace text)
  - Comment lines (any line with only comment and whitespace text)
  - Blank lines (lines with only whitespace text)
  - Comments
