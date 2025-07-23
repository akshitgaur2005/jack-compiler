mod tokenizer;
mod parser;
use std::{env, fs, path::Path};

use tokenizer::tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if path was provided
    if args.len() < 2 {
        println!("Usage: {} <file_or_directory", args[0]);
        return;
    }

    let path = Path::new(&args[1]);

    // Check if path exists
    if !path.exists() {
        println!("Path '{}' does not exist", path.display());
        return;
    }

    let mut tokens = Vec::new();

    if path.is_dir() {
        println!("Operating on Directory: {}", path.display());
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if !entry.file_name().into_string().unwrap().contains(".jack") {
                                continue;
                            }
                            let content = fs::read_to_string(entry.path()).expect(&format!(
                                "Error in reading file {}",
                                entry.path().display()
                            ));

    println!("{content}");
                            let mut this_tokens = tokenizer(&content).unwrap();
                            tokens.append(&mut this_tokens);
                        }
                        Err(e) => {
                            println!("Error in reading dir entry: {}", e);
                            return;
                        }
                    };
                }
            }
            Err(e) => {
                println!("Error in reading directory: {}", e);
                return;
            }
        }
    } else if path.is_file() {
        println!("Operating on file: {}", path.display());
        let content =
            fs::read_to_string(path).expect(&format!("Could not read the file {}", path.display()));
        let mut this_tokens = tokenizer(&content).unwrap();
    println!("{content}");
        tokens.append(&mut this_tokens);
    }

    println!("{:#?}", tokens);
}
