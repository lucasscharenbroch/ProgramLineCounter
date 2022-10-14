use std::io;
use std::fs;
use std::collections::HashSet;

fn read_line(s: &mut String) {
    s.clear();
    io::stdin().read_line(s).expect("Encountered a problem reading stdin!");
}

fn is_letter(char: u8) -> bool {
    const a : u8 = 'a' as u8;
    const z : u8 = 'z' as u8;
    const A : u8 = 'A' as u8;
    const Z : u8 = 'Z' as u8;

    match char {
        a..=z => true,
        A..=Z => true,
        _ => false,
    }
}

fn is_valid_file_type(file_type: &String) -> bool {
    if file_type.len() < 2 { 
        return false;
    }

    for (i, c) in file_type.as_bytes().iter().enumerate() {
        match i {
            0 => if *c != '.' as u8 { return false },
            _ => if !is_letter(*c) { return false },
        }
    }

    true
}

fn is_valid_directory(directory: &String) -> bool {
    if let Ok(_) = fs::read_dir(directory) {
        true
    } else {
        false
    }
}

fn prompt_for_file_types(file_type_vector: &mut Vec<String>) {
    println!("Enter the file types to count:");
    println!("E.G. \".rs\"");
    println!("Enter \"done\" when finished.");

    let mut input = String::new();
    loop {
        read_line(&mut input);
        let input = input.trim().to_string();
        if input.eq_ignore_ascii_case("done") {
            return;
        } else {
            if is_valid_file_type(&input) { 
                file_type_vector.push(input.clone());
            } else {
                println!("\"{input}\" is an invalid filetype!");
            }
        }
    }
}

fn prompt_for_comment_types(comment_type_vector: &mut Vec<CommentType>) {
    let mut input = String::new();
    loop {
        println!("Enter another comment type? (y/n)");
        read_line(&mut input);
        let input = input.trim().to_string();
        if input.eq_ignore_ascii_case("y") {
            read_a_comment_type(comment_type_vector);
        } else if input.eq_ignore_ascii_case("n") {
            return;
        } else {
            println!("invalid input: {}", input);
        }
    }
}

fn prompt_for_directories(directory_vector: &mut Vec<String>) {
    println!("Enter the directories to count, one per line:");
    println!("E.G. \nC:/dev \n../rust");
    println!("Enter \"done\" when finished.");

    let mut input = String::new();

    loop {
        read_line(&mut input);
        let input = input.trim().to_string();

        if input.eq_ignore_ascii_case("done") {
            return;
        } else {
            if !is_valid_directory(&input) {
                println!("invalid directory: \"{}\"!", input);
                continue;
            } 
            directory_vector.push(input);
        }
    }
}

fn read_a_comment_type(comment_type_vector: &mut Vec<CommentType>) {
    println!("Enter the mode (singleLine or multiLine):");
    
    let mut input = String::new();

    let comment_mode = loop {
        read_line(&mut input);
        let input = input.trim().to_string();
        if input.eq_ignore_ascii_case("singleline") {
            break CommentMode::SingleLine;
        } else if input.eq_ignore_ascii_case("multiline") {
            break CommentMode::MultiLine;
        } else {
            println!("invalid mode: {}", input);
            continue;
        }
    };

    println!("Enter comment start:");
    println!("E.G. // or /*");
    read_line(&mut input);
    let comment_start = input.trim().to_string();

    let comment_end = match comment_mode {
        CommentMode::SingleLine => "\n".to_string(),
        CommentMode::MultiLine => {
            println!("Enter comment end:");
            println!("E.G. */");
            read_line(&mut input);
            input.trim().to_string()
        },
    };

    let comment_type = CommentType {
        mode: comment_mode,
        opening_pattern: comment_start,
        closing_pattern: comment_end,
    };

    comment_type_vector.push(comment_type);
}

// finds all nested directories of the directories directory_vector
fn explore_nested_directories(directory_vector: &Vec<String>) -> std::io::Result<HashSet<String>> {
    let mut directory_set: HashSet<String> = HashSet::new();
    let mut directories_to_explore = directory_vector.clone();

    while !directories_to_explore.is_empty() {
        let dir = directories_to_explore.remove(directories_to_explore.len() - 1); // remove las one
        directory_set.insert(dir.clone());

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path_str = entry.path().to_str().unwrap().to_string();
            directory_set.insert(path_str.clone());
            if entry.path().is_dir() {
                directories_to_explore.push(path_str);
            }
        }
    }

    Ok(directory_set)
}

// finds all files in the given directories that have the given file types,
// and reads them to strings
fn read_qualifying_files(directory_vector: &Vec<String>, 
                        file_type_vector: &Vec<String>) -> std::io::Result<Vec<String>> {

    let directory_set = explore_nested_directories(directory_vector)?;
    let mut qualifying_files = Vec::new();

    for dir in directory_set.iter() {
        if let Err(_) = fs::read_dir(dir) { 
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }
            let entry_path_string = entry.path().to_str().unwrap().to_string();
            for file_type in file_type_vector {
                if entry_path_string.ends_with(file_type) {
                    qualifying_files.push(entry_path_string);
                    break;
                }
            }
        }
    }

    Ok(qualifying_files)
}

// reads source code from a string and returns (program_lines, blank_lines, comment_lines, comments)
fn parse_file_string(file_string: String, comment_types: &Vec<CommentType>) -> (u32, u32, u32, u32) {
    let mut program_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut comments = 0;

    let mut line_prog_chars = 0;
    let mut in_comment = false;
    let mut comment_end_pattern: &str = &"";

    // iterate through characters.
    // if not in comment, try to match each comment start to [current...current+comment_start.len]
    //      if a comment is matched, record the ending pattern ('\n' if singleLine), set in_comment to true, inc comments
    //      any non-whitespace char -> increment line_prog_chars
    //      any '\n': increment program_lines (only if line_prog_chars != 0, else inc blank_lines) , reset line_prog_chars
    // if in comment:
    //      any '\n': {increment program_lines or comment_lines, reset line_prog_chars} if comment_end_pattern is \n, in_comment = false
    //      otherwise try to match comment_end_pattern
    
    let mut i = 0;

    'outer:
    while i < file_string.len() {
        let c = file_string.as_bytes()[i as usize];

        if in_comment {
            if file_string[i..].starts_with(comment_end_pattern) {
                in_comment = false;
            }
            if c as char == '\n' {
                if line_prog_chars != 0 {
                    program_lines += 1;
                    line_prog_chars = 0;
                } else {
                    comment_lines += 1;
                }
            }
            i += comment_end_pattern.len();
            continue 'outer;
        } else {
            for comment_type in comment_types {
                if file_string[i..].starts_with(&comment_type.opening_pattern) {
                    comment_end_pattern = if let CommentMode::MultiLine = &comment_type.mode {
                        &comment_type.closing_pattern
                    } else { 
                        &"\n" 
                    };
                    in_comment = true;
                    comments += 1;
                    i += comment_type.opening_pattern.len(); // skip the opening pattern
                    continue 'outer;
                }
            }
            match c as char {
                ' ' => (),
                '\t' => (),
                '\n' => {
                    if line_prog_chars != 0 {
                        program_lines += 1;
                        line_prog_chars = 0;
                    } else {
                        blank_lines += 1;
                    }
                },
                _ =>  line_prog_chars += 1,
            }
        }
        i += 1;
    }

    (program_lines, blank_lines, comment_lines, comments)
}

fn count_lines(file_types: Vec<String>, comment_types: Vec<CommentType>, 
               directories: Vec<String>) -> Result<LineCount, String> {

    let file_vector = if let Ok(fv) = read_qualifying_files(&directories, &file_types) {
        fv
    } else {
        return Err("Encountered trouble reading files!".to_string());
    };


    let mut line_count = LineCount {
        file_names: file_vector,
        program_line_counts : Vec::new(),
        blank_line_counts: Vec::new(),
        comment_line_counts: Vec::new(),
        comment_counts: Vec::new(),
    };

    for file in &line_count.file_names {
        if let Ok(file_string) = fs::read_to_string(file) {
            let (lines, blanks, comment_lines, comments) = 
                parse_file_string(file_string, &comment_types);
            line_count.program_line_counts.push(lines);
            line_count.blank_line_counts.push(blanks);
            line_count.comment_line_counts.push(comment_lines);
            line_count.comment_counts.push(comments);
        } else {
            return Err(format!("problem reading file: {}", file));
        }
    }

    return Ok(line_count);
}

fn print_line_counts(counts: LineCount) {
    let mut program_line_total = 0;
    let mut blank_total = 0;
    let mut comment_line_total = 0;
    let mut comment_total = 0;

    for (i, file_name) in counts.file_names.iter().enumerate() {
        println!("Counts for {}:", file_name);
        println!("  Program lines: {}", counts.program_line_counts[i]);
        println!("  Blank lines:   {}", counts.blank_line_counts[i]);
        println!("  Comment lines: {}", counts.comment_line_counts[i]);
        println!("  Total lines:   {}", counts.program_line_counts[i] + counts.blank_line_counts[i] + 
                                    counts.comment_line_counts[i]);
        println!("  Comments:      {}", counts.comment_counts[i]);

        program_line_total += counts.program_line_counts[i];
        blank_total += counts.blank_line_counts[i];
        comment_line_total += counts.comment_line_counts[i];
        comment_total += counts.comment_counts[i];
    }

    println!("TOTALS:");
    println!("Program lines: {}", program_line_total);
    println!("Blank lines:   {}", blank_total);
    println!("Comment lines: {}", comment_line_total);
    println!("Total lines:   {}", program_line_total + blank_total + comment_line_total); 
    println!("Comments:      {}", comment_total);
}

enum CommentMode {
    SingleLine,
    MultiLine,
}

struct CommentType {
    mode: CommentMode,
    opening_pattern: String,
    closing_pattern: String,
}

struct LineCount {
    file_names: Vec<String>,
    // each element of each count vector corresponds to the a file name
    program_line_counts : Vec<u32>,
    blank_line_counts: Vec<u32>,
    comment_line_counts: Vec<u32>,
    comment_counts: Vec<u32>,
}

fn main() {
    let mut file_types = Vec::new(); // a list of file types, in String format
    prompt_for_file_types(&mut file_types);

    let mut comment_types = Vec::new();
    prompt_for_comment_types(&mut comment_types);

    let mut directories = Vec::new();
    prompt_for_directories(&mut directories);

    println!("reading files...");

    match count_lines(file_types, comment_types, directories) {
        Err(s) => println!("{}", s),
        Ok(counts) => print_line_counts(counts),
    }
}
