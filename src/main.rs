use std::{process::{self, ExitCode}, env, io, fs};
use vm::{executor, types::VMError};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();

    let (source, path) = match args.len() {
        1 => (
            // read from stdin
            io::read_to_string(io::stdin())
                .unwrap_or_else(|e| {
                    eprintln!("Error reading from stdin: {e}");
                    process::exit(1);
                }),
            None
        ),
        2 => (
            // read from file
            fs::read_to_string(&args[1])
                .unwrap_or_else(|e| {
                    eprintln!("Error reading from file: {e}");
                    process::exit(1);
                }),
            Some(fs::canonicalize(&args[1]).unwrap().to_owned())
        ),
        _ => {
            eprintln!("Usage: {} [file]", args[0]);
            process::exit(1);
        }
    };
    eprintln!("Path: {:?}", path);
    eprintln!();
    eprintln!();

    let program = compiler::compile_chars(source.chars());
    program.print();

    match executor::execute_program(program, path) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => match e {
            VMError::Exit(code) => ExitCode::from(code as u8),
            _ => {
                eprintln!("Error executing script: {:?}", e);
                ExitCode::from(1)
            }
        }
    }
}
