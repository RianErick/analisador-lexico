use crate::token::{Token, TokenKind};

/// Analisador lexico manual integrado ao sintatico.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn analyze(source: &str) -> Vec<Token> {
        let mut lexer = Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        };
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if eof {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_ignored();
        let line = self.line;
        let column = self.column;
        let Some(ch) = self.peek() else {
            return Token::new(TokenKind::Eof, "$", line, column);
        };

        if ch.is_ascii_alphabetic() || ch == '_' {
            let text = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
            let kind = if matches!(
                text.as_str(),
                "public" | "class" | "static" | "void" | "int" | "float" | "char" | "if" | "else"
            ) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            };
            return Token::new(kind, text, line, column);
        }

        if ch.is_ascii_digit() {
            let mut text = self.take_while(|c| c.is_ascii_digit());
            let mut kind = TokenKind::Integer;
            if self.peek() == Some('.') && self.peek_n(1).is_some_and(|c| c.is_ascii_digit()) {
                kind = TokenKind::Float;
                text.push(self.advance().unwrap());
                text.push_str(&self.take_while(|c| c.is_ascii_digit()));
            }
            return Token::new(kind, text, line, column);
        }

        let first = self.advance().unwrap();
        let mut text = first.to_string();
        if matches!(first, '=' | '!') && self.peek() == Some('=') {
            text.push(self.advance().unwrap());
        }
        let kind = if matches!(
            text.as_str(),
            "+" | "-" | "*" | "/" | "=" | "==" | "!=" | "<" | ">"
        ) {
            TokenKind::Operator
        } else if matches!(text.as_str(), "(" | ")" | "{" | "}" | ";") {
            TokenKind::Delimiter
        } else {
            TokenKind::Error
        };
        Token::new(kind, text, line, column)
    }

    fn skip_ignored(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.peek() == Some('/') && self.peek_n(1) == Some('/') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }
            } else if self.peek() == Some('/') && self.peek_n(1) == Some('*') {
                self.advance();
                self.advance();
                while self.peek().is_some()
                    && !(self.peek() == Some('*') && self.peek_n(1) == Some('/'))
                {
                    self.advance();
                }
                if self.peek().is_some() {
                    self.advance();
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while self.peek().is_some_and(&predicate) {
            result.push(self.advance().unwrap());
        }
        result
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_identifiers_and_numbers_to_grammar_terminals() {
        let tokens = Lexer::analyze("int valor = 2.5;");
        let terminals: Vec<_> = tokens.iter().map(Token::terminal).collect();
        assert_eq!(terminals, ["int", "id", "=", "num", ";", "$"]);
    }

    #[test]
    fn ignores_both_comment_styles() {
        let tokens = Lexer::analyze("// linha\nint /* bloco */ a;");
        assert_eq!(
            tokens.iter().map(Token::terminal).collect::<Vec<_>>(),
            ["int", "id", ";", "$"]
        );
    }
}
