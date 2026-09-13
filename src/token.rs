use serde::Serialize;

/// Palavras reservadas de Java reconhecidas pelo analisador (mais de 20).
pub const KEYWORDS: &[&str] = &[
    "abstract", "boolean", "break", "byte", "case", "catch", "char", "class",
    "continue", "default", "do", "double", "else", "extends", "final", "finally",
    "float", "for", "if", "implements", "import", "int", "interface", "long",
    "new", "package", "private", "protected", "public", "return", "short",
    "static", "super", "switch", "this", "throw", "throws", "try", "void",
    "while",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenKind {
    Keyword,
    Identifier,
    Integer,
    Float,
    Char,
    String,
    Operator,
    Delimiter,
    Error,
    Eof,
}

impl TokenKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "KEYWORD",
            TokenKind::Identifier => "IDENTIFIER",
            TokenKind::Integer => "INTEGER",
            TokenKind::Float => "FLOAT",
            TokenKind::Char => "CHAR",
            TokenKind::String => "STRING",
            TokenKind::Operator => "OPERATOR",
            TokenKind::Delimiter => "DELIMITER",
            TokenKind::Error => "ERROR",
            TokenKind::Eof => "EOF",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub tipo: String,
    pub lexema: String,
    pub atributo: Option<String>,
    pub linha: usize,
    pub coluna: usize,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexema: impl Into<String>,
        atributo: Option<String>,
        linha: usize,
        coluna: usize,
    ) -> Self {
        Self {
            tipo: kind.as_str().to_string(),
            lexema: lexema.into(),
            atributo,
            linha,
            coluna,
        }
    }

    pub fn is_error(&self) -> bool {
        self.tipo == TokenKind::Error.as_str()
    }
}

pub fn is_keyword(lexeme: &str) -> bool {
    KEYWORDS.binary_search(&lexeme).is_ok()
}

pub fn is_delimiter(lexeme: &str) -> bool {
    matches!(
        lexeme,
        ";" | "," | "." | "(" | ")" | "{" | "}" | "[" | "]"
    )
}
