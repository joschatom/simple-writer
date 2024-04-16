//! # A simple text editor
//! This is a simple text editor written in Rust. It is designed to be used in a terminal.
//! ## Usage
//! ```text
//! Usage: editor [FILE]
//! ```
//! ## Commands
//! ```text
//! + | Add line
//! - | Remove line
//! q | Quit
//! n | Set buffer name
//! p | Print buffer
//! g | Go to line
//! o | Open file
//! c | Clear buffer
//! l | Print current line
//! r | Replace current line
//! R | Replace current line with previous
//! S | Search
//! d | Duplicate line
//! D | Duplicate current line
//! v | Paste from clipboard
//! C | Clear buffer
//! w | Write to file
//! ? | Help
//! ```
//! ## License
//! This project is licensed under the MIT license. See the LICENSE file for more information.
//!
//! ## Contributing
//! Please feel free to contribute to this project. I am open to any suggestions or improvements.
//!
//! ## Contact
//! You can contact me on Discord at `@pro.ton` or via email `joscha.egloff@pm.me`
//!
//! # Summary
//! This is a simple text editor written in Rust. It is designed to be used in a terminal. It is licensed under the MIT license. See the LICENSE file for more information.
//! It is fast and easy to use. It is also very small and lightweight. No useless stuff like a GUI or syntax highlighting. Just a simple text editor. ;)

use clipboard::{ClipboardContext, ClipboardProvider};
use std::{
    io::{Read, Write as _},
    path::PathBuf,
};

pub struct State {
    lines: Vec<String>,
    buffer_name: String,
}

#[derive(Debug, Clone)]
enum File {
    New,
    Open(PathBuf),
}

fn main() {
    // Not really needed... but it's nice to have.
    if let Err(..) = std::panic::catch_unwind(run) {
        eprintln!("┌───────────────────────────────┐");
        eprintln!("│  A Critical Error occurred.   │");
        eprintln!("│  See the error above for      │");
        eprintln!("│  more information.            │");
        eprintln!("└───────────────────────────────┘");
    }
}

pub fn run() {
    println!("{}", include_str!("../header.txt").replace("{version}", env!("CARGO_PKG_VERSION")));

    let file: File;
    if std::env::args().len() > 1 {
        file = File::Open(
            std::env::args()
                .nth(1)
                .expect("┌───────────────────────────────┐\n│  Error: Unknown argument      │\n└───────────────────────────────┘")
                .try_into()
                .expect("┌───────────────────────────────┐\n│  Error: Invalid argument      │\n└───────────────────────────────┘")
        );
    } else {
        file = File::New;
    }

    let mut state = State {
        lines: Vec::new(),
        buffer_name: String::from("test.txt"),
    };

    let mut current_line;

    state.lines = file.disk_read(&mut state);

    current_line = state.lines.len().saturating_sub(1);

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("┌───────────────────────────────┐\n│  Error: Unable to read line   │\n└───────────────────────────────┘");
        match &input.chars().nth(0) {
            Some('+') => {
                let line = input.chars().skip(1).collect();

                if current_line >= state.lines.len() - 1 {
                    state.lines.push(line);
                } else {
                    state.lines.insert(current_line, line);
                }

                current_line += 1;
            }
            Some('q') => break,
            Some('n') => {
                state.buffer_name = input.chars().skip(2).collect();
            }
            Some('w') => {
                let mut file = match file.open(&mut state) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("┌───────────────────────────────┐");
                        eprintln!("│  Error: Unable to open file   │");
                        eprintln!("│  {}                           │", err);
                        eprintln!("└───────────────────────────────┘");
                        continue;
                    }
                };
                for line in &state.lines {
                    if let Err(err) = file.write_all(line.as_bytes()) {
                        eprintln!("┌───────────────────────────────┐");
                        eprintln!("│  Error: Unable to write to    │");
                        eprintln!("│  file - {}                    │", err);
                        eprintln!("└───────────────────────────────┘");
                        continue;
                    }
                }
            }
            Some('p') => {
                for (ln, lc) in state.lines.iter().enumerate() {
                    println!(" {}| {}", ln, lc);
                }
            }
            Some('l') => {
                println!("Current line: {}", current_line);
            }
            Some('g') => {
                let line = match input.chars().skip(2).collect::<String>().parse::<usize>() {
                    Ok(line) => line,
                    Err(err) => {
                        eprintln!("┌───────────────────────────────┐");
                        eprintln!("│  Error: Invalid line number   │");
                        eprintln!("│  {}                           │", err);
                        eprintln!("└───────────────────────────────┘");
                        continue;
                    }
                };
                if line > state.lines.len() - 1 {
                    eprintln!("┌───────────────────────────────┐");
                    eprintln!("│  Error: Line number out of    │");
                    eprintln!("│  range                        │");
                    eprintln!("└───────────────────────────────┘");
                } else {
                    current_line = line;
                }
            }
            Some('r') => {
                let line = input.chars().skip(1).collect();
                state.lines[current_line] = line;
            }
            // REPEAT LAST LINE
            Some('R') => {
                let ln = if (current_line - 1) > 0 {
                    current_line - 2
                } else {
                    0
                };
                let line = state.lines[ln].clone();
                state.lines[current_line] = line;
            }
            // SEARCH
            Some('S') => {
                let line: String = input.chars().skip(1).collect();
                let mut found = false;
                for (ln, lc) in state.lines.iter().enumerate() {
                    if lc.contains(&line) {
                        println!("{}| {}", ln, lc);
                        found = true;
                    }
                }
                if !found {
                    eprintln!("┌───────────────────────────────┐");
                    eprintln!("│  Error: No lines found        │");
                    eprintln!("└───────────────────────────────┘");
                }
            }
            Some('d') => {
                let line = input.chars().skip(1).collect();
                state.lines.insert(current_line + 1, line);
                current_line += 1;
            }
            // copy to clipboard
            Some('c') => {
                let ln = if current_line > 0 {
                    current_line - 1
                } else {
                    0
                };
                let line = state.lines[ln].clone();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(line).unwrap();
            }
            // DUPLICATE LINE
            Some('D') => {
                let ln = if current_line > 0 {
                    current_line - 1
                } else {
                    0
                };
                let line = state.lines[ln].clone();
                state.lines.insert(current_line, line);
                current_line += 1;
            }

            Some('v') => {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let line = ctx.get_contents().unwrap();
                state.lines.insert(current_line, line);
                current_line += 1;
            }
            Some('.') => {
                current_line += 1;
            }
            Some(',') => {
                if current_line > 0 {
                    current_line -= 1;
                } else {
                    eprintln!("┌───────────────────────────────┐");
                    eprintln!("│  Error: Buffer is empty       │");
                    eprintln!("└───────────────────────────────┘");
                }
            }
            Some('-') => {
                if current_line > 0 {
                    state.lines.remove(current_line);
                    current_line -= 1;
                } else {
                    eprintln!("┌───────────────────────────────┐");
                    eprintln!("│  Error: Buffer is empty       │");
                    eprintln!("└───────────────────────────────┘");
                }
            }
            Some('o') => {
                let path: String = input.chars().skip(2).collect();
                state.lines = match path.try_into().map(|path| File::Open(path)) {
                    Ok(file) => file.disk_read(&mut state),
                    Err(err) => {
                        eprintln!("┌───────────────────────────────┐");
                        eprintln!("│  Error: Invalid file path     │");
                        eprintln!("│  {}                           │", err);
                        eprintln!("└───────────────────────────────┘");
                        continue;
                    }
                };
            }
            Some('C') => {
                state.lines.clear();
                state.buffer_name = String::new();
            }
            Some('?') => {
                println!("┌───────────────────────────────┐");
                println!("│           Commands            │");
                println!("├───────────────────────────────┤");
                println!("│  + | Add line                 │");
                println!("│  - | Remove line              │");
                println!("│  q | Quit                     │");
                println!("│  n | Set buffer name          │");
                println!("│  p | Print buffer             │");
                println!("│  g | Go to line               │");
                println!("│  o | Open file                │");
                println!("│  c | Clear buffer             │");
                println!("│  l | Print current line       │");
                println!("│  r | Replace current line     │");
                println!("│  R | Replace current line with│");
                println!("│    | previous                 │");
                println!("│  S | Search                   │");
                println!("│  d | Duplicate line           │");
                println!("│  D | Duplicate current line   │");
                println!("│  v | Paste from clipboard     │");
                println!("│  C | Clear buffer             │");
                println!("│  w | Write to file            │");
                println!("│  ? | Help                     │");
                println!("└───────────────────────────────┘");
            }
            Some(_) => {
                eprintln!("┌───────────────────────────────┐");
                eprintln!("│  Error: Unknown line prefix   │");
                eprintln!("│  Use ? for help.              │");
                eprintln!("└───────────────────────────────┘");
            }
            None => break,
        }
    }
}

impl File {
    fn disk_read(&self, _: &mut State) -> Vec<String> {
        match self {
            File::New => Vec::new(),
            File::Open(path) => {
                let mut file = match std::fs::File::open(path) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("┌───────────────────────────────┐");
                        eprintln!("│  Error: Unable to open file   │");
                        eprintln!("│  {}                           │", err);
                        eprintln!("└───────────────────────────────┘");
                        return Vec::new();
                    }
                };
                let mut contents = String::new();
                if let Err(err) = file.read_to_string(&mut contents) {
                    eprintln!("┌───────────────────────────────┐");
                    eprintln!("│  Error: Unable to read file   │");
                    eprintln!("│  {}                           │", err);
                    eprintln!("└───────────────────────────────┘");
                    return Vec::new();
                }
                contents.lines().map(|s| s.to_owned()).collect()
            }
        }
    }

    fn open(&self, state: &mut State) -> std::io::Result<std::fs::File> {
        match self {
            File::New => std::fs::File::options()
                .write(true)
                .read(true)
                .create_new(true)
                .open(state.buffer_name.clone()),
            File::Open(path) => std::fs::File::options().write(true).read(true).open(path),
        }
    }
}
