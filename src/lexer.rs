use crate::buffer::{TwinBuffer, BUFFER_SIZE};
use crate::symbol_table::SymbolTable;
use crate::token::{is_delimiter, is_keyword, Token, TokenKind};

pub struct Analysis {
    pub tokens: Vec<Token>,
    pub symbols: SymbolTable,
    pub errors: Vec<Token>,
    pub buffer_events: Vec<crate::buffer::BufferEvent>,
    pub buffer_size: usize,
}

pub struct Lexer {
    buf: TwinBuffer,
}

impl Lexer {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            buf: TwinBuffer::new(source),
        }
    }

    pub fn analyze(source: impl Into<String>) -> Analysis {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        let mut symbols = SymbolTable::default();
        let mut errors = Vec::new();

        loop {
            let token = lexer.next_token();
            if token.tipo == TokenKind::Eof.as_str() {
                break;
            }
            if token.tipo == TokenKind::Identifier.as_str() {
                symbols.insert(&token.lexema);
            }
            if token.is_error() {
                errors.push(token.clone());
            }
            tokens.push(token);
        }

        Analysis {
            tokens,
            symbols,
            errors,
            buffer_events: lexer.buf.events.clone(),
            buffer_size: BUFFER_SIZE,
        }
    }

    fn next_token(&mut self) -> Token {
        if let Some(error) = self.skip_ignored() {
            return error;
        }
        self.buf.mark_begin();

        let Some(ch) = self.buf.advance() else {
            let (linha, coluna) = self.buf.begin_position();
            return Token::new(TokenKind::Eof, "", None, linha, coluna);
        };

        match ch {
            'A'..='Z' | 'a'..='z' | '_' => self.scan_identifier(),
            '0'..='9' => self.scan_number(),
            '"' => self.scan_string(),
            '\'' => self.scan_char(),
            ch if is_operator_start(ch) => self.scan_operator_or_delimiter(ch),
            _ => {
                let (linha, coluna) = self.buf.begin_position();
                Token::new(
                    TokenKind::Error,
                    self.buf.lexeme(),
                    Some(format!("caractere inválido '{ch}'")),
                    linha,
                    coluna,
                )
            }
        }
    }

    fn skip_ignored(&mut self) -> Option<Token> {
        loop {
            let Some(ch) = self.buf.peek() else {
                return None;
            };

            if ch.is_whitespace() {
                self.buf.advance();
                continue;
            }

            if ch == '/' {
                self.buf.mark_begin();
                self.buf.advance();
                match self.buf.peek() {
                    Some('/') => {
                        self.buf.advance();
                        self.skip_line_comment();
                        continue;
                    }
                    Some('*') => {
                        self.buf.advance();
                        if !self.skip_block_comment() {
                            let (linha, coluna) = self.buf.begin_position();
                            return Some(Token::new(
                                TokenKind::Error,
                                self.buf.lexeme(),
                                Some("comentário de bloco não terminado".into()),
                                linha,
                                coluna,
                            ));
                        }
                        continue;
                    }
                    _ => {
                        self.buf.retract();
                        return None;
                    }
                }
            }

            return None;
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.buf.peek() {
            self.buf.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> bool {
        loop {
            match self.buf.advance() {
                None => return false,
                Some('*') if self.buf.peek() == Some('/') => {
                    self.buf.advance();
                    return true;
                }
                _ => {}
            }
        }
    }

    fn scan_identifier(&mut self) -> Token {
        while matches!(
            self.buf.peek(),
            Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_')
        ) {
            self.buf.advance();
        }

        let lexema = self.buf.lexeme();
        let (linha, coluna) = self.buf.begin_position();
        if is_keyword(&lexema) {
            Token::new(TokenKind::Keyword, lexema, None, linha, coluna)
        } else {
            Token::new(
                TokenKind::Identifier,
                lexema.clone(),
                Some(lexema),
                linha,
                coluna,
            )
        }
    }

    fn scan_number(&mut self) -> Token {
        while matches!(self.buf.peek(), Some('0'..='9')) {
            self.buf.advance();
        }

        // 3,14 — vírgula usada como decimal (erro léxico pedido no enunciado)
        if self.buf.peek() == Some(',') {
            if matches!(self.peek_after_comma(), Some('0'..='9')) {
                self.buf.advance(); // vírgula
                while matches!(self.buf.peek(), Some('0'..='9')) {
                    self.buf.advance();
                }
                let lexema = self.buf.lexeme();
                let (linha, coluna) = self.buf.begin_position();
                return Token::new(
                    TokenKind::Error,
                    lexema,
                    Some("vírgula não é separador decimal válido (use ponto)".into()),
                    linha,
                    coluna,
                );
            }
        }

        // 123abc — identificador começando com dígito
        if matches!(self.buf.peek(), Some('A'..='Z' | 'a'..='z' | '_')) {
            while matches!(
                self.buf.peek(),
                Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_')
            ) {
                self.buf.advance();
            }
            let lexema = self.buf.lexeme();
            let (linha, coluna) = self.buf.begin_position();
            return Token::new(
                TokenKind::Error,
                lexema,
                Some("identificador não pode começar com número".into()),
                linha,
                coluna,
            );
        }

        if self.buf.peek() == Some('.') {
            if matches!(self.peek_after_dot(), Some('0'..='9')) {
                self.buf.advance(); // ponto
                while matches!(self.buf.peek(), Some('0'..='9')) {
                    self.buf.advance();
                }
                if matches!(self.buf.peek(), Some('A'..='Z' | 'a'..='z' | '_')) {
                    while matches!(
                        self.buf.peek(),
                        Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_')
                    ) {
                        self.buf.advance();
                    }
                    let lexema = self.buf.lexeme();
                    let (linha, coluna) = self.buf.begin_position();
                    return Token::new(
                        TokenKind::Error,
                        lexema,
                        Some("literal numérico inválido".into()),
                        linha,
                        coluna,
                    );
                }
                let lexema = self.buf.lexeme();
                let (linha, coluna) = self.buf.begin_position();
                return Token::new(
                    TokenKind::Float,
                    lexema.clone(),
                    Some(lexema),
                    linha,
                    coluna,
                );
            }
        }

        let lexema = self.buf.lexeme();
        let (linha, coluna) = self.buf.begin_position();
        Token::new(TokenKind::Integer, lexema.clone(), Some(lexema), linha, coluna)
    }

    fn peek_after_comma(&mut self) -> Option<char> {
        self.buf.advance(); // consome ','
        let next = self.buf.peek();
        self.buf.retract(); // devolve ','
        next
    }

    fn peek_after_dot(&mut self) -> Option<char> {
        self.buf.advance(); // consome '.'
        let next = self.buf.peek();
        self.buf.retract(); // devolve '.'
        next
    }

    fn scan_string(&mut self) -> Token {
        let (linha, coluna) = self.buf.begin_position();
        let mut closed = false;
        let mut unescaped = String::new();
        let mut broken_by_newline = false;

        while let Some(ch) = self.buf.advance() {
            match ch {
                '"' => {
                    closed = true;
                    break;
                }
                '\n' => {
                    broken_by_newline = true;
                    break;
                }
                '\\' => match self.buf.advance() {
                    Some('n') => unescaped.push('\n'),
                    Some('t') => unescaped.push('\t'),
                    Some('r') => unescaped.push('\r'),
                    Some('\\') => unescaped.push('\\'),
                    Some('"') => unescaped.push('"'),
                    Some('\'') => unescaped.push('\''),
                    Some(other) => {
                        unescaped.push('\\');
                        unescaped.push(other);
                    }
                    None => break,
                },
                other => unescaped.push(other),
            }
        }

        let lexema = self.buf.lexeme();
        if !closed {
            let motivo = if broken_by_newline {
                "string não terminada (quebra de linha)"
            } else {
                "string não terminada"
            };
            return Token::new(
                TokenKind::Error,
                lexema,
                Some(motivo.into()),
                linha,
                coluna,
            );
        }

        Token::new(TokenKind::String, lexema, Some(unescaped), linha, coluna)
    }

    fn scan_char(&mut self) -> Token {
        let (linha, coluna) = self.buf.begin_position();
        let mut value = String::new();
        let mut closed = false;

        match self.buf.advance() {
            None | Some('\n') => {
                return Token::new(
                    TokenKind::Error,
                    self.buf.lexeme(),
                    Some("literal de caractere não terminado".into()),
                    linha,
                    coluna,
                );
            }
            Some('\\') => match self.buf.advance() {
                Some('n') => value.push('\n'),
                Some('t') => value.push('\t'),
                Some('r') => value.push('\r'),
                Some('\\') => value.push('\\'),
                Some('\'') => value.push('\''),
                Some('"') => value.push('"'),
                Some(other) => {
                    value.push('\\');
                    value.push(other);
                }
                None => {
                    return Token::new(
                        TokenKind::Error,
                        self.buf.lexeme(),
                        Some("literal de caractere não terminado".into()),
                        linha,
                        coluna,
                    );
                }
            },
            Some(ch) => value.push(ch),
        }

        if self.buf.peek() == Some('\'') {
            self.buf.advance();
            closed = true;
        }

        let lexema = self.buf.lexeme();
        if !closed {
            return Token::new(
                TokenKind::Error,
                lexema,
                Some("literal de caractere não terminado".into()),
                linha,
                coluna,
            );
        }

        Token::new(TokenKind::Char, lexema, Some(value), linha, coluna)
    }

    fn scan_operator_or_delimiter(&mut self, first: char) -> Token {
        let (linha, coluna) = self.buf.begin_position();
        let second = self.buf.peek();

        let two = match (first, second) {
            ('=', Some('='))
            | ('!', Some('='))
            | ('<', Some('='))
            | ('>', Some('='))
            | ('&', Some('&'))
            | ('|', Some('|'))
            | ('+', Some('='))
            | ('-', Some('='))
            | ('*', Some('='))
            | ('/', Some('='))
            | ('%', Some('='))
            | ('+', Some('+'))
            | ('-', Some('-')) => {
                self.buf.advance();
                self.buf.lexeme()
            }
            _ => self.buf.lexeme(),
        };

        if is_delimiter(&two) {
            Token::new(TokenKind::Delimiter, two, None, linha, coluna)
        } else if is_known_operator(&two) {
            Token::new(TokenKind::Operator, two, None, linha, coluna)
        } else {
            Token::new(
                TokenKind::Error,
                two.clone(),
                Some(format!("símbolo inválido '{two}'")),
                linha,
                coluna,
            )
        }
    }
}

fn is_operator_start(ch: char) -> bool {
    matches!(
        ch,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | ';' | ','
            | '.' | '(' | ')' | '{' | '}' | '[' | ']'
    )
}

fn is_known_operator(lexeme: &str) -> bool {
    matches!(
        lexeme,
        "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&"
            | "||" | "!" | "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "++" | "--"
            | "&" | "|"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types(source: &str) -> Vec<(String, String)> {
        Lexer::analyze(source)
            .tokens
            .into_iter()
            .map(|t| (t.tipo, t.lexema))
            .collect()
    }

    #[test]
    fn keywords_and_identifiers() {
        let tokens = types("public static int soma");
        assert_eq!(tokens[0], ("KEYWORD".into(), "public".into()));
        assert_eq!(tokens[1], ("KEYWORD".into(), "static".into()));
        assert_eq!(tokens[2], ("KEYWORD".into(), "int".into()));
        assert_eq!(tokens[3], ("IDENTIFIER".into(), "soma".into()));
    }

    #[test]
    fn float_and_comma_error() {
        let ok = types("3.14");
        assert_eq!(ok, vec![("FLOAT".into(), "3.14".into())]);

        let err = Lexer::analyze("3,14");
        assert_eq!(err.errors.len(), 1);
        assert_eq!(err.errors[0].lexema, "3,14");
    }

    #[test]
    fn number_starting_identifier_is_error() {
        let err = Lexer::analyze("int 1numero = 10;");
        assert!(err.errors.iter().any(|e| e.lexema == "1numero"));
    }

    #[test]
    fn unterminated_string_is_error() {
        let err = Lexer::analyze("String s = \"abc;");
        assert!(err.errors.iter().any(|e| e.tipo == "ERROR"));
    }

    #[test]
    fn comments_are_ignored() {
        let tokens = types("int a = 1; // fim\n/* bloco */ float b;");
        assert!(tokens.iter().all(|(_, lex)| lex != "//" && lex != "/*"));
        assert!(tokens.iter().any(|(t, l)| t == "KEYWORD" && l == "float"));
    }

    #[test]
    fn symbol_table_counts_only_identifiers() {
        let result = Lexer::analyze("int a = a + b;");
        let entries = result.symbols.entries();
        let a = entries.iter().find(|e| e.identificador == "a").unwrap();
        let b = entries.iter().find(|e| e.identificador == "b").unwrap();
        assert_eq!(a.ocorrencias, 2);
        assert_eq!(b.ocorrencias, 1);
        assert!(entries.iter().all(|e| e.identificador != "int"));
    }

    #[test]
    fn operators_and_delimiters() {
        let tokens = types("a += 1; if (x <= y && z != 0) {}");
        assert!(tokens.iter().any(|(_, l)| l == "+="));
        assert!(tokens.iter().any(|(_, l)| l == "<="));
        assert!(tokens.iter().any(|(_, l)| l == "&&"));
        assert!(tokens.iter().any(|(_, l)| l == "!="));
    }
}
