use std::{process::ExitCode, env, io, fs};
use vm::executor;

fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();

    let (source, path) = match args.len() {
        1 => (
            // read from stdin
            io::read_to_string(io::stdin()).unwrap(),
            None
        ),
        2 => (
            // read from file
            fs::read_to_string(&args[1]).unwrap(),
            Some(fs::canonicalize(&args[1]).unwrap().into())
        ),
        _ => {
            eprintln!("Usage: {} [file]", args[0]);
            return ExitCode::from(1);
        }
    };
    eprintln!("Path: {:?}", path);
    eprintln!();
    eprintln!();

    let program = compiler::compile_chars(source.chars()).unwrap();
    program.print();

    match executor::execute_program(program, path) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error executing script: {:?}", e);
            ExitCode::from(1)
        }
    }
}
