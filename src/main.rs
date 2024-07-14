use std::{
    env,
    error::Error,
    fmt,
    io::{self, Write},
};

enum RatexError {
    UnknownTokenError(String),
}

impl fmt::Display for RatexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownTokenError(token) => {
                write!(f, "Unknown Token: {}", token)
            }
        }
    }
}

impl fmt::Debug for RatexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownTokenError(token) => {
                write!(f, "Unknown Token: {}", token)
            }
        }
    }
}

impl Error for RatexError {}

enum RatexToken {
    Number,
}

impl fmt::Display for RatexToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number => {
                write!(f, "Number")
            }
        }
    }
}

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

        run(prompt)?;
    }

    Ok(())
}

fn run(code: String) -> Result<(), RatexError> {
    if code == "exit" {
        return Err(RatexError::UnknownTokenError("exit".to_owned()));
    }
    let tokens: Vec<RatexToken> = scan_tokens(code)?;

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}

fn scan_tokens(code: String) -> Result<Vec<RatexToken>, RatexError> {
    Ok(vec![RatexToken::Number])
}
