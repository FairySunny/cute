use bytecode::program::Program;
use compiler::{lexer::Lexer, parser};

fn main() {
    let source = include_str!("../test/_sqrt.cute");
    let mut program = Program::new();
    parser::parse(Lexer::new(source.chars()), &mut program);
    program.print();
    let bundle = program.bundle();
    println!("#CP = {}, #F = {}", bundle.constant_pool.len(), bundle.func_list.len());
    vm::vm::run_program(&bundle);
}
