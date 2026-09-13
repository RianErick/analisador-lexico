mod buffer;
mod lexer;
mod symbol_table;
mod token;
mod web;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() || args.iter().any(|a| a == "--ui" || a == "--web") {
        args.retain(|a| a != "--ui" && a != "--web");
        let addr = args
            .iter()
            .find(|a| a.starts_with("--addr="))
            .map(|a| a.trim_start_matches("--addr=").to_string())
            .unwrap_or_else(|| "127.0.0.1:3000".into());
        web::serve(&addr).await;
        return;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("informe um arquivo .java ou execute sem argumentos para abrir a interface");
            process::exit(1);
        });

    let source = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("erro ao ler {path}: {err}");
        process::exit(1);
    });

    print_cli(&path, &source);
}

fn print_help() {
    println!(
        "Analisador Léxico (subconjunto de Java)\n\n\
         Uso:\n\
           cargo run                  abre a interface web\n\
           cargo run -- arquivo.java  analisa no terminal\n\
           cargo run -- --ui          abre a interface web\n\
           cargo run -- --addr=127.0.0.1:8080\n"
    );
}

fn print_cli(path: &str, source: &str) {
    let result = Lexer::analyze(source);

    println!("arquivo: {path}");
    println!("buffer duplo: N = {}\n", result.buffer_size);

    println!("=== Tabela de tokens ===");
    println!(
        "{:<4} {:<12} {:<28} {:<36} {:>5} {:>5}",
        "#", "tipo", "lexema", "atributo", "lin", "col"
    );
    for (i, token) in result.tokens.iter().enumerate() {
        println!(
            "{:<4} {:<12} {:<28} {:<36} {:>5} {:>5}",
            i + 1,
            token.tipo,
            truncate(&token.lexema, 28),
            truncate(token.atributo.as_deref().unwrap_or("—"), 36),
            token.linha,
            token.coluna
        );
    }

    println!("\n=== Tabela de símbolos ===");
    println!("{:<24} {:>12}", "identificador", "ocorrências");
    for entry in result.symbols.entries() {
        println!("{:<24} {:>12}", entry.identificador, entry.ocorrencias);
    }

    if result.errors.is_empty() {
        println!("\nnenhum erro léxico.");
    } else {
        println!("\n=== Erros léxicos ({}) ===", result.errors.len());
        for error in &result.errors {
            println!(
                "  linha {}, coluna {}: {} ({})",
                error.linha,
                error.coluna,
                error.lexema,
                error.atributo.as_deref().unwrap_or("inválido")
            );
        }
    }
}

fn truncate(text: &str, max: usize) -> String {
    let display = text.replace('\n', "\\n").replace('\t', "\\t");
    if display.chars().count() <= max {
        display
    } else {
        format!("{}…", display.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
