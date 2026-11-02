#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Identifier,
    Integer,
    Float,
    Operator,
    Delimiter,
    Error,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            column,
        }
    }

    /// Converte os tokens concretos do lexico nos terminais abstratos da gramatica.
    pub fn terminal(&self) -> String {
        match self.kind {
            TokenKind::Identifier => "id".into(),
            TokenKind::Integer | TokenKind::Float => "num".into(),
            TokenKind::Eof => "$".into(),
            _ => self.lexeme.clone(),
        }
    }
}
