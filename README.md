# SQL Parser in Rust

A SQL parser that implements SELECT and CREATE TABLE statements using Pratt parsing.

## Features

### SQL Statement Support
- **SELECT Statements**
  - Basic column selection
  - Table selection (FROM clause)
  - Filtering (WHERE clause)
  - Sorting (ORDER BY clause with ASC/DESC)
  - Support for expressions in column selection
  - Wildcard (*) support

- **CREATE TABLE Statements**
  - Column definitions with types (INT, VARCHAR, BOOL)
  - Constraints (PRIMARY KEY, NOT NULL)
  - CHECK constraints with expressions
  - Support for complex table schemas

### Expression Support
- Binary Operations (+, -, *, /, =, !=, >, >=, <, <=)
- Logical Operations (AND, OR)
- Unary Operations (NOT, +, -)
- Parenthesized expressions
- String literals (both single and double quotes)
- Numeric literals
- Boolean literals (TRUE, FALSE)
- Column references

### Data Types
- INT
- VARCHAR(n)
- BOOL

## Example Queries

```sql
-- Simple SELECT
SELECT name, surname FROM users;

-- SELECT with WHERE clause
SELECT one, two FROM users WHERE one > 1 AND two < 1;

-- SELECT with complex expressions
SELECT age * 5, 'this is a string' FROM users;

-- SELECT with ORDER BY
SELECT id, salary FROM users ORDER BY salary - 2 * 10 ASC, id DESC;

-- CREATE TABLE with constraints
CREATE TABLE complex_table(
    id INT PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    is_junior BOOL,
    age INT CHECK(age >= 18) CHECK(age <= 65)
);
```

## Project Structure

- `src/tokenizer.rs`: Implements the SQL lexer/tokenizer
- `src/parser.rs`: Implements the Pratt parser and SQL statement parser
- `src/token.rs`: Defines the token types
- `src/statement.rs`: Defines the AST structures
- `src/main.rs`: CLI interface for testing the parser

## Usage

1. Clone the repository
2. Run with Cargo:
   ```bash
   cargo run
   ```
3. Enter SQL queries at the prompt
4. Press Ctrl+C to exit

## Implementation Details

The parser uses the Pratt parsing technique for handling operator precedence in expressions. This makes it particularly good at parsing complex mathematical and logical expressions while maintaining proper precedence rules.

The implementation follows a modular design with clear separation between:
- Tokenization (lexical analysis)
- Expression parsing (using Pratt parsing)
- Statement parsing (high-level SQL syntax)

## Error Handling

The parser provides detailed error messages for:
- Syntax errors
- Invalid tokens
- Missing required clauses
- Mismatched parentheses
- Unclosed string literals
- Invalid constraint definitions

## Author
Angunna Gamage Thilina Ranmal  
Contact: ranmal.gamage@sa.stud.vu.lt

## License
This project is part of a programming languages course assignment.