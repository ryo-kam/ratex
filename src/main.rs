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
    let mut interpreter = RatexInterpreter::new();

    loop {
        let mut prompt = String::new();
        print!("> ");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut prompt);

        if prompt.len() == 2 {
            break;
        };

        let tokens = Scanner::new(prompt.as_str()).scan_tokens();

        let mut parser = Parser::new(tokens);

        let ast: Vec<Stmt> = parser.parse();

        if !parser.has_error() {
            for statement in ast {
                match statement {
                    Stmt::Expression(expr) => match interpreter.evaluate(expr.expr) {
                        Ok(value) => println!("{}", value),
                        Err(e) => println!("Error: {}", e),
                    },
                    _ => match interpreter.interpret(vec![statement]) {
                        Ok(()) => {}
                        Err(e) => println!("Error: {}", e),
                    },
                }
            }
        }
    }

    Ok(())
}

fn run(code: String) {
    let tokens = Scanner::new(code.as_str()).scan_tokens();

    let mut parser = Parser::new(tokens);

    let ast: Vec<Stmt> = parser.parse();

    if parser.has_error() {
        println!("Code won't be executed since it has errors.");
    } else {
        let mut interpreter = RatexInterpreter::new();
        match interpreter.interpret(ast) {
            Ok(()) => {}
            Err(e) => println!("Error: {}", e),
        }
    }
}
