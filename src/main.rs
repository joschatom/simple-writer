use std::{path::PathBuf, io::{Read, Write as _}};

pub struct State{
    lines: Vec<String>,
    buffer_name: String,
}

#[derive(Debug, Clone)]
enum File{
    New,
    Open(PathBuf),
}

fn main() {
    // Not really needed... but it's nice to have.
    if let Err(..) = std::panic::catch_unwind(run) {
        eprintln!("A Critical Error occurred. See the error above for more information.");
    }
}

pub fn run(){
    println!(include_str!("../header.txt"));

    let file: File;
    if std::env::args().len() > 1 {
        file = File::Open(std::env::args().nth(1).expect("Error: Unknown argument").try_into().expect("Error: Invalid argument"));
    } else {
        file = File::New;

    }

    
    let mut state = State{
        lines: Vec::new(),
        buffer_name: String::from("test.txt"),
    };

    let mut current_line;

    state.lines = file.disk_read(&mut state);

    current_line = state.lines.len() - 1;

    loop{
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Error: Unable to read line");
        match &input.chars().nth(0){
            Some('+') => {
                let line = input.chars().skip(1).collect();

                if current_line >= state.lines.len() - 1{
                    state.lines.push(line);
                } else {
                    state.lines.insert(current_line, line);
                }
            
                current_line += 1;
            },
            Some('q') => break,
            Some('n') => {
                state.buffer_name = input.chars().skip(2).collect();
            },
            Some('w') => {
                let mut file = file.open(&mut state).expect("Error: Unable to open file");
                for line in &state.lines{
                    file.write_all(line.as_bytes()).expect("Error: Unable to write to file");
                }
            },
            Some('p') => {
                for (ln, lc) in state.lines.iter().enumerate(){
                    println!(" {}| {}", ln, lc);
                }
            },
            Some('l') => {
                println!("Current line: {}", current_line);
            }
            Some('g') => {
                let line = input.chars().skip(2).collect::<String>().parse::<usize>().expect("Error: Invalid line number");
                if line > state.lines.len() - 1{
                    eprintln!("Error: Line number out of range");
                } else {
                    current_line = line;
                }
            },
            Some('-') => {
                if current_line > 0{
                    state.lines.remove(current_line);
                    current_line -= 1;
                } else {
                    eprintln!("Error: Buffer is empty");
                }
            },
            Some('o') => {
                let path = input.chars().skip(2).collect::<String>();
                state.lines = File::Open(path.try_into().expect("Error: Invalid file path")).disk_read(&mut state);

            },
            Some('c') => {
                state.lines.clear();
                state.buffer_name = String::new();
            },
            Some('?') => {
                println!("+ - Add line");
                println!("- - Remove line");
                println!("q - Quit");
                println!("n - Set buffer name");
                println!("p - Print buffer");
                println!("g - Go to line");
                println!("o - Open file");
                println!("c - Clear buffer");
                println!("w - Write to file");
                println!("? - Help");
            },
            Some(_) => eprintln!("Error: Unknown line prefix, use ? for help."),
            None => break,
        }
    }
}


impl File{

    fn disk_read(&self, _: &mut State) -> Vec<String>{
        match self {
            File::New => Vec::new(),
            File::Open(path) => {
                let mut file = std::fs::File::open(path).expect("Error: Unable to open file");
                let mut contents = String::new();
                file.read_to_string(&mut contents).expect("Error: Unable to read file");
                contents.lines().map(|s| s.to_owned()).collect()  
            }
        }
    }

    fn open(&self, state: &mut State) -> std::io::Result<std::fs::File>{
        match self {
            File::New => {
                std::fs::File::options()
                    .write(true)
                    .read(true)
                    .create_new(true)
                    .open(state.buffer_name.clone())
            }
            File::Open(path) => {
                std::fs::File::options()
                    .write(true)
                    .read(true)
                    .open(path)
            }
        }
    }
}