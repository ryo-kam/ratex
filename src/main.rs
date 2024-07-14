use std::{
    env,
    io::{self, Write},
};

mod error;
use error::RatexErrorType;

use crate::error::RatexError;

mod token;
use crate::token::RatexTokenType;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: ratex [script/filepath]");
    } else if args.len() == 2 {
        let result = run_file(args[0].clone());
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
    if code == "exit" {
        return Err(RatexError {
            source: RatexErrorType::UnknownTokenError(0, "exit".to_owned()),
        });
    } else {
        let tokens: Vec<RatexTokenType> = scan_tokens(code)?;

        for token in tokens {
            println!("{token}");
        }
    }

    Ok(())
}

fn scan_tokens(code: String) -> Result<Vec<RatexTokenType>, RatexError> {
    Ok(vec![RatexTokenType::Number(1.0)])
}
