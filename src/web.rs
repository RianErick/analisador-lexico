use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::lexer::Lexer;
use crate::token::Token;

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Deserialize)]
struct AnalyzeRequest {
    source: String,
}

#[derive(Serialize)]
struct Stats {
    tokens: usize,
    identificadores: usize,
    palavras_chave: usize,
    literais: usize,
    operadores: usize,
    erros: usize,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    tokens: Vec<Token>,
    simbolos: Vec<crate::symbol_table::SymbolEntry>,
    erros: Vec<Token>,
    buffer_eventos: Vec<crate::buffer::BufferEvent>,
    buffer_size: usize,
    estatisticas: Stats,
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn analyze(Json(req): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let result = Lexer::analyze(req.source);
    let estatisticas = Stats {
        tokens: result.tokens.len(),
        identificadores: result
            .tokens
            .iter()
            .filter(|t| t.tipo == "IDENTIFIER")
            .count(),
        palavras_chave: result.tokens.iter().filter(|t| t.tipo == "KEYWORD").count(),
        literais: result
            .tokens
            .iter()
            .filter(|t| matches!(t.tipo.as_str(), "INTEGER" | "FLOAT" | "CHAR" | "STRING"))
            .count(),
        operadores: result
            .tokens
            .iter()
            .filter(|t| matches!(t.tipo.as_str(), "OPERATOR" | "DELIMITER"))
            .count(),
        erros: result.errors.len(),
    };

    Json(AnalyzeResponse {
        tokens: result.tokens,
        simbolos: result.symbols.entries(),
        erros: result.errors,
        buffer_eventos: result.buffer_events,
        buffer_size: result.buffer_size,
        estatisticas,
    })
}

pub async fn serve(addr: &str) {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/analyze", post(analyze))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("não foi possível abrir {addr}: {err}"));

    println!("Interface disponível em http://{addr}");
    axum::serve(listener, app)
        .await
        .expect("falha ao iniciar o servidor");
}
