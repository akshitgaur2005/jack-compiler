mod tokenizer;
mod parser;

use std::{env, fs, path::Path};
use tokenizer::{tokenizer, Token};
use parser::{Parser, ClassNode};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if path was provided
    if args.len() < 2 {
        println!("Usage: {} <file_or_directory>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);

    // Check if path exists
    if !path.exists() {
        println!("Path '{}' does not exist", path.display());
        return;
    }

    if path.is_dir() {
        println!("Operating on Directory: {}", path.display());
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let file_path = entry.path();
                            if file_path.extension().and_then(|s| s.to_str()) == Some("jack") {
                                println!("--- Processing file: {} ---", file_path.display());
                                process_file(&file_path);
                            }
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
        process_file(path);
    }
}

fn process_file(file_path: &Path) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Could not read the file {}: {}", file_path.display(), e);
            return;
        }
    };

    // 1. Tokenize
    let tokens = match tokenizer(&content) {
        Ok(t) => t,
        Err(e) => {
            println!("Tokenizer error in {}: {}", file_path.display(), e);
            return;
        }
    };
    debug_tokens(&tokens);

    // 2. Parse
    let mut parser = Parser::new(&tokens);
    match parser.parse_class() {
        Ok(ast) => {
            debug_ast(&ast);
        }
        Err(e) => {
            println!("Parser error in {}: {}", file_path.display(), e);
        }
    }
}


fn debug_tokens(tokens: &[Token]) {
    println!("=== TOKENS DEBUG ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
    println!("==================\n");
}

fn debug_ast(ast: &ClassNode) {
    println!("=== AST DEBUG ===");
    println!("{:#?}", ast);
    println!("===============\n");
}
