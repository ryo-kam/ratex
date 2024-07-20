use std::{
    env,
    io::{self, Write},
};

mod ast;
mod environment;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;

use ast::Stmt;
use interpreter::RatexInterpreter;
use parser::Parser;
use scanner::Scanner;

use crate::error::RatexError;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: ratex [script]");
    } else if args.len() == 2 {
        run_file(
            env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .to_owned()
                + "/"
                + &args[1].clone(),
        );
        println!("Done!")
    } else {
        let result = run_prompt();
        match result {
            Ok(()) => {
                println!("Done!")
            }
            Err(e) => {
                println!("Error: {e}")
            }
        }
    }
}

fn run_file(path: String) {
    let file = std::fs::read_to_string(path).unwrap();
    run(file)
}

fn run_prompt() -> Result<(), RatexError> {
    println!("Prompt mode");

    loop {
        let mut prompt = String::new();
        print!("> ");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut prompt);

        if prompt.len() == 2 {
            break;
        };

        run(prompt.trim().to_owned());
    }

    Ok(())
}

fn run(code: String) {
    let mut scanner = Scanner::new(code.as_str());

    scanner.scan_tokens();

    let tokens = scanner.tokens;

    let mut parser = Parser::new(tokens);

    let ast: Vec<Stmt> = parser.parse();

    if parser.has_error() {
        println!("Code won't be executed since it has errors.");
    } else {
        let mut interpreter = RatexInterpreter::new();
        interpreter.interpret(ast);
    }
}
