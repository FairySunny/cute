use std::{env, io, fs, process};
use vm::executor;

fn main() {
    let args: Vec<_> = env::args().collect();

    let source = match args.len() {
        1 => {
            // read from stdin
            io::read_to_string(io::stdin())
                .unwrap_or_else(|e| {
                    eprintln!("Error reading from stdin: {e}");
                    process::exit(1);
                })
        }
        2 => {
            // read from file
            fs::read_to_string(&args[1])
                .unwrap_or_else(|e| {
                    eprintln!("Error reading from file: {e}");
                    process::exit(1);
                })
        }
        _ => {
            eprintln!("Usage: {} [file]", args[0]);
            process::exit(1);
        }
    };

    let path = env::current_dir().unwrap().to_str().unwrap().to_owned();

    let program = compiler::compile_chars(source.chars());
    program.print();

    executor::execute_program(program, vec![path])
        .unwrap_or_else(|e| {
            eprintln!("Error executing script: {:?}", e);
            process::exit(1);
        });
}
