use std::{
    env,
    io::{self, Write},
};

mod ast;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;

use ast::AstPrinter;
use error::RatexErrorType;
use interpreter::RatexInterpreter;
use parser::Parser;
use scanner::Scanner;

use crate::error::RatexError;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: ratex [script]");
    } else if args.len() == 2 {
        let result = run_file(
            env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                .to_owned()
                + "/"
                + &args[1].clone(),
        );
        match result {
            Ok(()) => {
                println!("Done!")
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }
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

fn run_file(path: String) -> Result<(), RatexError> {
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

        run(prompt.trim().to_owned())?;
    }

    Ok(())
}

fn run(code: String) -> Result<(), RatexError> {
    let mut scanner = Scanner::new(code.as_str());

    scanner.scan_tokens()?;

    let tokens = scanner.tokens;

    let mut parser = Parser::new(tokens);

    let ast = parser.parse().unwrap();

    let mut interpreter = RatexInterpreter {};
    println!("{:?}", interpreter.evaluate(ast));

    Ok(())
}
