use crate::token::{Keyword, Token};
use std::iter::Peekable;
use std::str::Chars;

pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.input.next();
        }
    }

    fn read_number(&mut self, first_digit: char) -> Token {
        let mut number = first_digit.to_string();
        
        while let Some(&c) = self.input.peek() {
            if !c.is_digit(10) {
                break;
            }
            number.push(c);
            self.input.next();
        }

        match number.parse::<u64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Invalid(first_digit),
        }
    }

    fn read_identifier_or_keyword(&mut self, first_char: char) -> Token {
        let mut identifier = first_char.to_string();
        
        while let Some(&c) = self.input.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            identifier.push(c);
            self.input.next();
        }

        // Convert to uppercase for case-insensitive comparison
        let upper_identifier = identifier.to_uppercase();
        
        match upper_identifier.as_str() {
            "SELECT" => Token::Keyword(Keyword::Select),
            "CREATE" => Token::Keyword(Keyword::Create),
            "TABLE" => Token::Keyword(Keyword::Table),
            "WHERE" => Token::Keyword(Keyword::Where),
            "ORDER" => Token::Keyword(Keyword::Order),
            "BY" => Token::Keyword(Keyword::By),
            "ASC" => Token::Keyword(Keyword::Asc),
            "DESC" => Token::Keyword(Keyword::Desc),
            "FROM" => Token::Keyword(Keyword::From),
            "AND" => Token::Keyword(Keyword::And),
            "OR" => Token::Keyword(Keyword::Or),
            "NOT" => Token::Keyword(Keyword::Not),
            "TRUE" => Token::Keyword(Keyword::True),
            "FALSE" => Token::Keyword(Keyword::False),
            "PRIMARY" => Token::Keyword(Keyword::Primary),
            "KEY" => Token::Keyword(Keyword::Key),
            "CHECK" => Token::Keyword(Keyword::Check),
            "INT" => Token::Keyword(Keyword::Int),
            "BOOL" => Token::Keyword(Keyword::Bool),
            "VARCHAR" => Token::Keyword(Keyword::Varchar),
            "NULL" => Token::Keyword(Keyword::Null),
            _ => Token::Identifier(identifier),
        }
    }

    fn read_string(&mut self, quote_char: char) -> Token {
        let mut string = String::new();
        
        while let Some(c) = self.input.next() {
            if c == quote_char {
                return Token::String(string);
            }
            string.push(c);
        }
        
        // If we get here, the string was not properly terminated
        Token::Invalid(quote_char)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let next_char = self.input.next()?;

        let token = match next_char {
            '0'..='9' => self.read_number(next_char),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(next_char),
            '\'' | '"' => self.read_string(next_char),
            '(' => Token::LeftParentheses,
            ')' => Token::RightParentheses,
            '>' => {
                if let Some(&'=') = self.input.peek() {
                    self.input.next();
                    Token::GreaterThanOrEqual
                } else {
                    Token::GreaterThan
                }
            },
            '<' => {
                if let Some(&'=') = self.input.peek() {
                    self.input.next();
                    Token::LessThanOrEqual
                } else {
                    Token::LessThan
                }
            },
            '=' => Token::Equal,
            '!' => {
                if let Some(&'=') = self.input.peek() {
                    self.input.next();
                    Token::NotEqual
                } else {
                    Token::Invalid('!')
                }
            },
            '*' => Token::Star,
            '/' => Token::Divide,
            '-' => Token::Minus,
            '+' => Token::Plus,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            c => Token::Invalid(c),
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "SELECT * FROM users;";
        let tokenizer = Tokenizer::new(input);
        let tokens: Vec<Token> = tokenizer.collect();
        
        assert_eq!(tokens, vec![
            Token::Keyword(Keyword::Select),
            Token::Star,
            Token::Keyword(Keyword::From),
            Token::Identifier("users".to_string()),
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_string_literals() {
        let input = "'hello' \"world\"";
        let tokenizer = Tokenizer::new(input);
        let tokens: Vec<Token> = tokenizer.collect();
        
        assert_eq!(tokens, vec![
            Token::String("hello".to_string()),
            Token::String("world".to_string()),
        ]);
    }

    #[test]
    fn test_numbers_and_operators() {
        let input = "42 >= 30";
        let tokenizer = Tokenizer::new(input);
        let tokens: Vec<Token> = tokenizer.collect();
        
        assert_eq!(tokens, vec![
            Token::Number(42),
            Token::GreaterThanOrEqual,
            Token::Number(30),
        ]);
    }
}
