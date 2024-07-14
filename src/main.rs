use std::{
    env,
    error::Error,
    fmt,
    io::{self, Write},
};

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
                println!("Bye bye!")
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }
    } else {
        let result = run_prompt();
        match result {
            Ok(()) => {
                println!("Bye bye!")
            }
            Err(e) => {
                println!("Error: {e}")
            }
        }
    }
}

fn run_file(path: String) -> Result<(), Box<dyn Error>> {
    let file = std::fs::read_to_string(path)?;
    run(file)
}

fn run_prompt() -> Result<(), Box<dyn Error>> {
    println!("Prompt mode");

    loop {
        let mut prompt = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut prompt)?;

        if prompt.is_empty() {
            break;
        };

        run(prompt)?;
    }

    Ok(())
}

fn run(code: String) -> Result<(), Box<dyn Error>> {
    let tokens: Vec<RatexToken> = scan_tokens(code)?;

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}

fn scan_tokens(code: String) -> Result<Vec<RatexToken>, Box<dyn Error>> {
    println!("{code}");
    Ok(vec![RatexToken::Number])
}
