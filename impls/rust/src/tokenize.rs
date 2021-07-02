use crate::reader::{ParseError, ParseResult};

#[derive(PartialEq, Eq)]
pub enum Token {
    String(String),
    Ident(String),
    Keyword(String),
    Number(String),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    SingleQuote,
    Backtick,
    Tilde,
    At,
    Hat,
}

pub struct Tokenizer<'a> {
    input: &'a str,
    current: usize,
    start: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            current: 0,
            start: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.current..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let mut char_indices = self.input[self.current..].char_indices();
        let char = char_indices.next().map(|(_, char)| char);
        self.current += char_indices
            .next()
            .map(|(idx, _)| idx)
            .unwrap_or(self.input.len() - self.current);
        char
    }

    fn read_string(&mut self) -> ParseResult<String> {
        let mut string = String::new();
        while let Some(c) = self.advance() {
            if c == '\\' {
                match self.advance() {
                    None => return Err(ParseError::UnterminatedString),
                    Some('n') => string.push('\n'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some(c) => return Err(ParseError::InvalidStringEscape(c)),
                }
                continue;
            }
            if c == '"' {
                return Ok(string);
            }
            string.push(c);
        }
        Err(ParseError::UnterminatedString)
    }

    fn read_keyword(&mut self) -> ParseResult<String> {
        self.advance();
        let mut ident = self.read_ident();
        if ident.is_empty() {
            Err(ParseError::EmptyKeyword)
        } else {
            ident.insert(0, char::from_u32(0x29E).unwrap());
            Ok(ident)
        }
    }

    fn is_ident(char: char) -> bool {
        !Self::is_punct(char) && !Self::is_whitespace(char)
    }

    fn is_punct(char: char) -> bool {
        matches!(
            char,
            '[' | ']' | '{' | '}' | '(' | '\'' | '"' | '`' | ',' | ';' | ')'
        )
    }

    fn is_whitespace(char: char) -> bool {
        char.is_whitespace() || char == ','
    }

    fn read_ident(&mut self) -> String {
        while let Some(char) = self.peek() {
            if Self::is_ident(char) {
                self.advance();
            } else {
                break;
            }
        }
        self.input[self.start..self.current].to_string()
    }

    fn read_num(&mut self) -> ParseResult<String> {
        while let Some(char) = self.peek() {
            if Self::is_punct(char) || Self::is_whitespace(char) {
                break;
            }
            if !matches!(char, '0'..='9') {
                return Err(ParseError::InvalidNumber);
            }
            self.advance();
        }
        Ok(self.input[self.start..self.current].to_string())
    }

    fn skip_whitespace(&mut self) {
        loop {
            let next = self.peek();
            if matches!(next, Some(c) if Self::is_whitespace(c)) {
                self.advance();
                continue;
            }
            if matches!(next, Some(';')) {
                while !matches!(self.advance(), Some('\n') | None) {}
                continue;
            }
            break;
        }
    }

    pub fn next_token(&mut self) -> ParseResult<Token> {
        self.skip_whitespace();
        self.start = self.current;
        let token = match self.advance() {
            Some(char) => match char {
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '\'' => Token::SingleQuote,
                '~' => Token::Tilde,
                '@' => Token::At,
                '`' => Token::Backtick,
                '^' => Token::Hat,
                '"' => Token::String(self.read_string()?),
                ':' => Token::Keyword(self.read_keyword()?),
                '0'..='9' => Token::Number(self.read_num()?),
                c if Self::is_ident(c) => Token::Ident(self.read_ident()),
                _ => unreachable!("all characters can be idents if nothing else"),
            },
            None => return Err(ParseError::UnexpectedEof),
        };
        Ok(token)
    }
}
