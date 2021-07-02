use std::{collections::HashMap, fmt::Display};

use crate::{
    tokenize::{Token, Tokenizer},
    value::Value,
};

#[derive(Debug)]
pub enum ParseError {
    EmptyInput,
    UnexpectedEof,
    UnterminatedString,
    EmptyKeyword,
    InvalidNumber,
    UnexpectedToken,
    InvalidStringEscape(char),
    InvalidMapKey(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: better errors!!
        match self {
            ParseError::UnexpectedEof | ParseError::UnterminatedString => {
                write!(f, "unexpected end of input")
            }
            ParseError::InvalidNumber => write!(f, "invalid number"),
            ParseError::EmptyKeyword => write!(f, "empty keyword"),
            ParseError::UnexpectedToken => write!(f, "unexpected token"),
            ParseError::InvalidStringEscape(c) => write!(f, "invalid string escape: \\{}", c),
            ParseError::InvalidMapKey(k) => write!(f, "invalid map key '{}'", k),
            ParseError::EmptyInput => unreachable!(),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

struct Reader<'a> {
    tokenizer: Tokenizer<'a>,
    next_token: Option<Token>,
}

impl<'a> Reader<'a> {
    fn new(input: &'a str) -> ParseResult<Self> {
        let mut tokenizer = Tokenizer::new(input);
        Ok(Self {
            next_token: Some(tokenizer.next_token().map_err(|e| match e {
                ParseError::UnexpectedEof => ParseError::EmptyInput,
                e => e,
            })?),
            tokenizer,
        })
    }
    fn next(&mut self) -> ParseResult<Token> {
        if let Some(token) = self.next_token.take() {
            Ok(token)
        } else {
            self.tokenizer.next_token()
        }
    }

    fn peek(&mut self) -> ParseResult<&Token> {
        if self.next_token.is_none() {
            self.next_token = Some(self.tokenizer.next_token()?);
        }
        Ok(self.next_token.as_ref().unwrap())
    }

    fn read_form(&mut self) -> ParseResult<Value> {
        let token = self.peek()?;
        match token {
            Token::LeftParen => Ok(Value::List(self.read_list(Token::RightParen)?)),
            Token::LeftBracket => Ok(Value::Vec(self.read_list(Token::RightBracket)?)),
            Token::LeftBrace => Ok(Value::Map(self.read_map()?)),
            Token::SingleQuote => {
                self.next().unwrap();
                self.read_reader_macro("quote")
            }
            Token::Backtick => {
                self.next().unwrap();
                self.read_reader_macro("quasiquote")
            }
            Token::Tilde => {
                self.next().unwrap();
                if matches!(self.peek(), Ok(&Token::At)) {
                    self.next().unwrap();
                    self.read_reader_macro("splice-unquote")
                } else {
                    self.read_reader_macro("unquote")
                }
            }
            Token::At => {
                self.next().unwrap();
                self.read_reader_macro("deref")
            }
            Token::Hat => {
                self.next().unwrap();
                let second = self.read_form()?;
                let first = self.read_form()?;
                Ok(Value::List(vec![
                    Value::Symbol("with-meta".into()),
                    first,
                    second,
                ]))
            }
            _ => self.read_atom(),
        }
    }

    fn read_reader_macro(&mut self, name: impl Into<String>) -> ParseResult<Value> {
        let content = self.read_form()?;
        Ok(Value::List(vec![Value::Symbol(name.into()), content]))
    }

    fn read_list(&mut self, terminator: Token) -> ParseResult<Vec<Value>> {
        self.next()?;
        let mut values = vec![];
        while self.peek()? != &terminator {
            values.push(self.read_form()?)
        }
        self.next()?;
        Ok(values)
    }

    fn read_map(&mut self) -> ParseResult<HashMap<String, Value>> {
        self.next()?;
        let mut map = HashMap::new();
        while self.peek()? != &Token::RightBrace {
            map.insert(
                self.read_form()?
                    .into_hash_map_key()
                    .map_err(|e| ParseError::InvalidMapKey(e.to_string()))?,
                self.read_form()?,
            );
        }
        self.next()?;
        Ok(map)
    }

    fn read_atom(&mut self) -> ParseResult<Value> {
        let value = self.next()?;
        Ok(match value {
            Token::Number(n) => Value::Number(n.parse().map_err(|_| ParseError::InvalidNumber)?),
            Token::Ident(value) if value == "true" => Value::Bool(true),
            Token::Ident(value) if value == "false" => Value::Bool(false),
            Token::Ident(value) if value == "nil" => Value::Nil,
            Token::Ident(value) => Value::Symbol(value),
            Token::Keyword(value) => Value::Keyword(value),
            Token::String(value) => Value::String(value),
            _ => return Err(ParseError::UnexpectedToken),
        })
    }
}

pub fn read_str(input: &str) -> ParseResult<Value> {
    Reader::new(input)?.read_form()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wat() {
        read_str("~@(1 2 3)").unwrap();
    }
}
