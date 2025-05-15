use crate::statement::{BinaryOperator, Expression, UnaryOperator, Statement, TableColumn, DBType, Constraint};
use crate::token::{Keyword, Token};
use crate::tokenizer::Tokenizer;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next().unwrap_or(Token::Eof);
        Self {
            tokenizer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.tokenizer.next().unwrap_or(Token::Eof);
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        if self.current_token == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current_token))
        }
    }

    fn get_precedence(operator: &BinaryOperator) -> u8 {
        match operator {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
            BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual => 4,
            BinaryOperator::Plus | BinaryOperator::Minus => 5,
            BinaryOperator::Multiply | BinaryOperator::Divide => 6,
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.current_token.clone();
        self.advance();

        match token {
            Token::Number(n) => Ok(Expression::Number(n)),
            Token::String(s) => Ok(Expression::String(s)),
            Token::Identifier(id) => Ok(Expression::Identifier(id)),
            Token::Keyword(Keyword::True) => Ok(Expression::Bool(true)),
            Token::Keyword(Keyword::False) => Ok(Expression::Bool(false)),
            Token::Star => Ok(Expression::Identifier("*".to_string())),
            Token::LeftParentheses => {
                let expr = self.parse_expression(0)?;
                if self.current_token != Token::RightParentheses {
                    return Err("Expected closing parenthesis".to_string());
                }
                self.advance();
                Ok(expr)
            },
            Token::Minus => {
                let expr = self.parse_expression(7)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Minus,
                })
            },
            Token::Plus => {
                let expr = self.parse_expression(7)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Plus,
                })
            },
            Token::Keyword(Keyword::Not) => {
                let expr = self.parse_expression(7)?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Not,
                })
            },
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn get_binary_operator(token: &Token) -> Option<BinaryOperator> {
        match token {
            Token::Plus => Some(BinaryOperator::Plus),
            Token::Minus => Some(BinaryOperator::Minus),
            Token::Star => Some(BinaryOperator::Multiply),
            Token::Divide => Some(BinaryOperator::Divide),
            Token::GreaterThan => Some(BinaryOperator::GreaterThan),
            Token::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
            Token::LessThan => Some(BinaryOperator::LessThan),
            Token::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
            Token::Equal => Some(BinaryOperator::Equal),
            Token::NotEqual => Some(BinaryOperator::NotEqual),
            Token::Keyword(Keyword::And) => Some(BinaryOperator::And),
            Token::Keyword(Keyword::Or) => Some(BinaryOperator::Or),
            _ => None,
        }
    }

    pub fn parse_expression(&mut self, precedence: u8) -> Result<Expression, String> {
        let mut left = self.parse_primary()?;

        while let Some(operator) = Self::get_binary_operator(&self.current_token) {
            let op_precedence = Self::get_precedence(&operator);
            if op_precedence <= precedence {
                break;
            }

            self.advance();
            let right = self.parse_expression(op_precedence)?;

            left = Expression::BinaryOperation {
                left_operand: Box::new(left),
                operator,
                right_operand: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_select_columns(&mut self) -> Result<Vec<Expression>, String> {
        let mut columns = Vec::new();
        
        loop {
            columns.push(self.parse_expression(0)?);
            
            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(columns)
    }

    fn parse_orderby(&mut self) -> Result<Vec<Expression>, String> {
        let mut orderby = Vec::new();
        
        loop {
            let expr = self.parse_expression(0)?;
            
            // Check for ASC/DESC
            let expr = match self.current_token {
                Token::Keyword(Keyword::Asc) => {
                    self.advance();
                    Expression::UnaryOperation {
                        operand: Box::new(expr),
                        operator: UnaryOperator::Asc,
                    }
                },
                Token::Keyword(Keyword::Desc) => {
                    self.advance();
                    Expression::UnaryOperation {
                        operand: Box::new(expr),
                        operator: UnaryOperator::Desc,
                    }
                },
                _ => expr,
            };
            
            orderby.push(expr);
            
            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(orderby)
    }

    fn parse_column_type(&mut self) -> Result<DBType, String> {
        match self.current_token {
            Token::Keyword(Keyword::Int) => {
                self.advance();
                Ok(DBType::Int)
            },
            Token::Keyword(Keyword::Bool) => {
                self.advance();
                Ok(DBType::Bool)
            },
            Token::Keyword(Keyword::Varchar) => {
                self.advance();
                self.expect_token(Token::LeftParentheses)?;
                
                if let Token::Number(size) = self.current_token {
                    self.advance();
                    self.expect_token(Token::RightParentheses)?;
                    Ok(DBType::Varchar(size as usize))
                } else {
                    Err("Expected number for VARCHAR size".to_string())
                }
            },
            _ => Err("Expected a valid data type".to_string()),
        }
    }

    fn parse_column_constraints(&mut self) -> Result<Vec<Constraint>, String> {
        let mut constraints = Vec::new();
        
        while let Some(constraint) = match self.current_token {
            Token::Keyword(Keyword::Primary) => {
                self.advance();
                if let Token::Keyword(Keyword::Key) = self.current_token {
                    self.advance();
                    Some(Constraint::PrimaryKey)
                } else {
                    return Err("Expected KEY after PRIMARY".to_string());
                }
            },
            Token::Keyword(Keyword::Not) => {
                self.advance();
                if let Token::Keyword(Keyword::Null) = self.current_token {
                    self.advance();
                    Some(Constraint::NotNull)
                } else {
                    return Err("Expected NULL after NOT".to_string());
                }
            },
            Token::Keyword(Keyword::Check) => {
                self.advance();
                self.expect_token(Token::LeftParentheses)?;
                let expr = self.parse_expression(0)?;
                self.expect_token(Token::RightParentheses)?;
                Some(Constraint::Check(expr))
            },
            _ => None,
        } {
            constraints.push(constraint);
        }
        
        Ok(constraints)
    }

    fn parse_column_definition(&mut self) -> Result<TableColumn, String> {
        if let Token::Identifier(name) = self.current_token.clone() {
            self.advance();
            let column_type = self.parse_column_type()?;
            let constraints = self.parse_column_constraints()?;
            
            Ok(TableColumn {
                column_name: name,
                column_type,
                constraints,
            })
        } else {
            Err("Expected column name".to_string())
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.advance(); // Skip TABLE keyword
        
        let table_name = if let Token::Identifier(name) = self.current_token.clone() {
            self.advance();
            name
        } else {
            return Err("Expected table name".to_string());
        };
        
        self.expect_token(Token::LeftParentheses)?;
        
        let mut column_list = Vec::new();
        
        loop {
            column_list.push(self.parse_column_definition()?);
            
            match self.current_token {
                Token::Comma => {
                    self.advance();
                    continue;
                },
                Token::RightParentheses => {
                    self.advance();
                    break;
                },
                _ => return Err("Expected ',' or ')'".to_string()),
            }
        }
        
        self.expect_token(Token::Semicolon)?;
        
        Ok(Statement::CreateTable {
            table_name,
            column_list,
        })
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        let columns = self.parse_select_columns()?;
        
        if self.current_token != Token::Keyword(Keyword::From) {
            return Err("Expected FROM clause".to_string());
        }
        self.advance();
        
        let from = if let Token::Identifier(table_name) = self.current_token.clone() {
            self.advance();
            table_name
        } else {
            return Err("Expected table name".to_string());
        };
        
        let mut r#where = None;
        let mut orderby = Vec::new();
        
        if self.current_token == Token::Keyword(Keyword::Where) {
            self.advance();
            r#where = Some(self.parse_expression(0)?);
        }
        
        if self.current_token == Token::Keyword(Keyword::Order) {
            self.advance();
            if self.current_token != Token::Keyword(Keyword::By) {
                return Err("Expected BY after ORDER".to_string());
            }
            self.advance();
            orderby = self.parse_orderby()?;
        }
        
        self.expect_token(Token::Semicolon)?;
        
        Ok(Statement::Select {
            columns,
            from,
            r#where,
            orderby,
        })
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token {
            Token::Keyword(Keyword::Select) => {
                self.advance();
                self.parse_select()
            },
            Token::Keyword(Keyword::Create) => {
                self.advance();
                if self.current_token != Token::Keyword(Keyword::Table) {
                    return Err("Expected TABLE after CREATE".to_string());
                }
                self.parse_create_table()
            },
            _ => Err("Expected SELECT or CREATE".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let mut parser = Parser::new("SELECT name, age FROM users;");
        let stmt = parser.parse_statement().unwrap();
        
        assert_eq!(stmt, Statement::Select {
            columns: vec![
                Expression::Identifier("name".to_string()),
                Expression::Identifier("age".to_string()),
            ],
            from: "users".to_string(),
            r#where: None,
            orderby: vec![],
        });
    }

    #[test]
    fn test_select_with_where() {
        let mut parser = Parser::new("SELECT id FROM users WHERE age >= 18;");
        let stmt = parser.parse_statement().unwrap();
        
        assert_eq!(stmt, Statement::Select {
            columns: vec![Expression::Identifier("id".to_string())],
            from: "users".to_string(),
            r#where: Some(Expression::BinaryOperation {
                left_operand: Box::new(Expression::Identifier("age".to_string())),
                operator: BinaryOperator::GreaterThanOrEqual,
                right_operand: Box::new(Expression::Number(18)),
            }),
            orderby: vec![],
        });
    }

    #[test]
    fn test_create_table() {
        let mut parser = Parser::new("CREATE TABLE users(id INT PRIMARY KEY, name VARCHAR(255) NOT NULL);");
        let stmt = parser.parse_statement().unwrap();
        
        assert_eq!(stmt, Statement::CreateTable {
            table_name: "users".to_string(),
            column_list: vec![
                TableColumn {
                    column_name: "id".to_string(),
                    column_type: DBType::Int,
                    constraints: vec![Constraint::PrimaryKey],
                },
                TableColumn {
                    column_name: "name".to_string(),
                    column_type: DBType::Varchar(255),
                    constraints: vec![Constraint::NotNull],
                },
            ],
        });
    }
}
