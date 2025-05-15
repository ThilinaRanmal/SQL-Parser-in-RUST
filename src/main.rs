use std::io::{self, Write};
use crate::parser::Parser;

mod token;
mod tokenizer;
mod statement;
mod parser;

fn main() -> io::Result<()> {
    println!("Welcome to the SQL Parser!");
    println!("Enter SQL queries (press Ctrl+C to exit)");
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        match Parser::new(input).parse_statement() {
            Ok(statement) => println!("{:#?}", statement),
            Err(error) => eprintln!("Error: {}", error),
        }
    }
}
