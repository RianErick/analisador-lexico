use std::collections::{BTreeMap, BTreeSet};

use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::grammar::Grammar;
use crate::lexer::Lexer;
use crate::parser::{Parser, SyntaxError, TraceStep};
use crate::token::Token;

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Deserialize)]
struct AnalyzeRequest {
    source: String,
}

#[derive(Serialize)]
struct TokenView {
    kind: String,
    lexeme: String,
    terminal: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct ProductionView {
    number: usize,
    left: String,
    right: String,
}

#[derive(Serialize)]
struct TableCellView {
    nonterminal: String,
    terminal: String,
    production: usize,
    rule: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    accepted: bool,
    tokens: Vec<TokenView>,
    errors: Vec<SyntaxError>,
    trace: Vec<TraceStep>,
    tree: String,
    productions: Vec<ProductionView>,
    first: BTreeMap<String, BTreeSet<String>>,
    follow: BTreeMap<String, BTreeSet<String>>,
    table: Vec<TableCellView>,
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn analyze(Json(request): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let grammar = Grammar::java_subset().expect("a gramatica interna deve ser LL(1)");
    let tokens = Lexer::analyze(&request.source);
    let result = Parser::new(&grammar).parse(&tokens);
    let tree = result.preorder_tree();
    let token_views = tokens.iter().map(token_view).collect();
    let productions = grammar
        .productions
        .iter()
        .enumerate()
        .map(|(index, production)| ProductionView {
            number: index + 1,
            left: production.left.clone(),
            right: production.right.join(" "),
        })
        .collect();
    let table = grammar
        .table
        .iter()
        .map(|((nonterminal, terminal), &index)| {
            let production = &grammar.productions[index];
            TableCellView {
                nonterminal: nonterminal.clone(),
                terminal: terminal.clone(),
                production: index + 1,
                rule: format!("{} -> {}", production.left, production.right.join(" ")),
            }
        })
        .collect();

    Json(AnalyzeResponse {
        accepted: result.accepted,
        tokens: token_views,
        errors: result.errors,
        trace: result.trace,
        tree,
        productions,
        first: grammar.first,
        follow: grammar.follow,
        table,
    })
}

fn token_view(token: &Token) -> TokenView {
    TokenView {
        kind: format!("{:?}", token.kind),
        lexeme: token.lexeme.clone(),
        terminal: token.terminal(),
        line: token.line,
        column: token.column,
    }
}

pub async fn serve(addr: &str) {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/analyze", post(analyze))
        .layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|error| panic!("nao foi possivel abrir {addr}: {error}"));
    println!("Interface disponivel em http://{addr}");
    axum::serve(listener, app)
        .await
        .expect("falha ao iniciar o servidor");
}
