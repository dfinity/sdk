use std::fmt;
use std::iter::Peekable;
use std::str::CharIndices;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
    UnknownEscape(char),
    NonTerminatedString(usize),
    ExpectedDigit,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::ExpectedDigit => write!(fmt, "Expected a digit"),
            LexicalError::UnknownEscape(c) => write!(fmt, "Unknown escape \\{}", c),
            LexicalError::NonTerminatedString(pos) => {
                write!(fmt, "Unclosed string literal starting at {}", pos)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Equals,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Comma,
    Null,
    Vec,
    Record,
    Variant,
    None,
    Opt,
    Label(String),
    TextLiteral(String),
    IntLiteral(i64),
    NatLiteral(u64),
    BooleanLiteral(bool),
}

impl fmt::Display for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
pub struct Lexer<'input> {
    input: Peekable<CharIndices<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Lexer<'input> {
        let mut lexer = Lexer {
            input: input.char_indices().peekable(),
        };
        lexer.consume_whitespace();
        lexer
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.input.next()
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.input.peek().cloned()
    }

    fn consume_whitespace(&mut self) {
        while let Some((_, c)) = self.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn read_digits(&mut self, buffer: &mut String) -> Result<usize, LexicalError> {
        let mut len = 0;
        while let Some((_, c)) = self.peek() {
            if c.is_digit(10) {
                len += 1;
                buffer.push(self.next_char().unwrap().1)
            } else {
                break;
            }
        }

        if len == 0 {
            // Not a single digit was read, this is an error
            Err(LexicalError::ExpectedDigit)
        } else {
            Ok(len)
        }
    }

    fn read_string_literal(
        &mut self,
        start_position: usize,
    ) -> Spanned<Token, usize, LexicalError> {
        let mut result = String::new();
        let end_position: usize;
        loop {
            match self.next_char() {
                Some((end, '"')) => {
                    end_position = end + 1;
                    break;
                }
                Some((_, '\\')) => match self.next_char() {
                    Some((_, 'n')) => result.push('\n'),
                    Some((_, 't')) => result.push('\t'),
                    Some((_, '\\')) => result.push('\\'),
                    Some((_, '"')) => result.push('"'),
                    Some((_, c)) => return Err(LexicalError::UnknownEscape(c)),
                    None => return Err(LexicalError::NonTerminatedString(start_position)),
                },
                Some((_, c)) => result.push(c),
                None => return Err(LexicalError::NonTerminatedString(start_position)),
            }
        }
        Ok((start_position, Token::TextLiteral(result), end_position))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match self.next_char() {
            Some((i, '(')) => Some(Ok((i, Token::LParen, i + 1))),
            Some((i, ')')) => Some(Ok((i, Token::RParen, i + 1))),
            Some((i, '{')) => Some(Ok((i, Token::LBrace, i + 1))),
            Some((i, '}')) => Some(Ok((i, Token::RBrace, i + 1))),
            Some((i, ';')) => Some(Ok((i, Token::Semi, i + 1))),
            Some((i, ',')) => Some(Ok((i, Token::Comma, i + 1))),
            Some((i, '=')) => Some(Ok((i, Token::Equals, i + 1))),
            Some((i, '"')) => Some(self.read_string_literal(i)),
            Some((i, c @ '+')) | Some((i, c @ '-')) => {
                let mut res = c.to_string();
                let digit_len = self.read_digits(&mut res);
                Some(digit_len.map(|len| {
                    (
                        i,
                        Token::IntLiteral(res.parse::<i64>().unwrap()),
                        i + 1 + len,
                    )
                }))
            }
            Some((i, c)) if c.is_digit(10) => {
                let mut res = c.to_string();
                let len = self.read_digits(&mut res).unwrap_or(0) + 1;
                Some(Ok((
                    i,
                    Token::NatLiteral(res.parse::<u64>().unwrap()),
                    i + len,
                )))
            }
            Some((i, c)) if c.is_alphabetic() => {
                let mut res = c.to_string();
                while let Some((_, c)) = self.peek() {
                    if c.is_alphabetic() || c == '_' {
                        res.push(self.next_char().unwrap().1)
                    } else {
                        break;
                    }
                }
                let tok = match res.as_str() {
                    "true" => Ok((Token::BooleanLiteral(true), 4)),
                    "false" => Ok((Token::BooleanLiteral(false), 5)),
                    "none" => Ok((Token::None, 4)),
                    "null" => Ok((Token::Null, 4)),
                    "opt" => Ok((Token::Opt, 3)),
                    "vec" => Ok((Token::Vec, 3)),
                    "record" => Ok((Token::Record, 6)),
                    "variant" => Ok((Token::Variant, 7)),
                    label => Ok((Token::Label(label.to_string()), label.len())),
                };
                Some(tok.map(|(token, len)| (i, token, i + len)))
            }
            _ => None,
        };
        self.consume_whitespace();
        token
    }
}
