mod grammar;
mod lexer;
mod parser;
mod token;
mod web;

use grammar::{Grammar, EPSILON};
use lexer::Lexer;
use parser::Parser;
use std::{env, fs, process};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--ui" || a == "--web" || a.starts_with("--addr="))
    {
        let addr = args
            .iter()
            .find_map(|a| a.strip_prefix("--addr="))
            .unwrap_or("127.0.0.1:3000");
        web::serve(addr).await;
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        help();
        return;
    }
    let path = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("erro: informe um arquivo .java");
            process::exit(2)
        });
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("erro ao ler {path}: {e}");
        process::exit(2)
    });
    let grammar = Grammar::java_subset().unwrap_or_else(|e| {
        eprintln!("gramatica invalida: {e}");
        process::exit(2)
    });
    let tokens = Lexer::analyze(&source);

    println!("=== ANALISADOR SINTATICO LL(1) ===\nArquivo: {path}\n");
    if args.iter().any(|a| a == "--tokens") {
        print_tokens(&tokens);
    }
    if args.iter().any(|a| a == "--grammar") {
        print_grammar(&grammar);
    }

    let result = Parser::new(&grammar).parse(&tokens);
    if !args.iter().any(|a| a == "--no-trace") {
        println!("=== TRACE DA PILHA ===");
        println!(
            "{:<5} | {:<42} | {:<52} | ACAO",
            "PASSO", "PILHA (topo a direita)", "ENTRADA"
        );
        println!("{}", "-".repeat(150));
        for (i, step) in result.trace.iter().enumerate() {
            println!(
                "{:<5} | {:<42} | {:<52} | {}",
                i + 1,
                truncate(&step.stack, 42),
                truncate(&step.input, 52),
                step.action
            );
        }
        println!();
    }
    if args.iter().any(|a| a == "--tree") {
        println!(
            "=== ARVORE DE DERIVACAO (PRE-ORDEM) ===\n{}",
            result.preorder_tree()
        );
    }

    if result.accepted {
        println!("VEREDITO: ACEITO COM SUCESSO - programa sintaticamente valido.");
    } else {
        println!(
            "VEREDITO: REJEITADO - {} erro(s) encontrado(s):",
            result.errors.len()
        );
        for error in &result.errors {
            println!(
                "  linha {}, coluna {}: Erro Sintatico: {}",
                error.line, error.column, error.message
            );
        }
        process::exit(1);
    }
}

fn print_tokens(tokens: &[token::Token]) {
    println!("=== TOKENS PRODUZIDOS PELO LEXICO ===");
    println!(
        "{:<5} {:<16} {:<18} {:<10} POSICAO",
        "#", "TIPO", "LEXEMA", "TERMINAL"
    );
    for (i, token) in tokens.iter().enumerate() {
        println!(
            "{:<5} {:<16} {:<18} {:<10} {}:{}",
            i + 1,
            format!("{:?}", token.kind),
            token.lexeme,
            token.terminal(),
            token.line,
            token.column
        );
    }
    println!();
}

fn print_grammar(g: &Grammar) {
    println!("=== PRODUCOES ===");
    for (i, p) in g.productions.iter().enumerate() {
        println!("P{:02}: {} -> {}", i + 1, p.left, p.right.join(" "));
    }
    println!("\n=== CONJUNTOS FIRST ===");
    for nt in &g.nonterminals {
        println!(
            "FIRST({:<8}) = {{ {} }}",
            nt,
            g.first[nt].iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    println!("\n=== CONJUNTOS FOLLOW ===");
    for nt in &g.nonterminals {
        println!(
            "FOLLOW({:<8}) = {{ {} }}",
            nt,
            g.follow[nt].iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    println!("\n=== TABELA PREDITIVA M (somente celulas preenchidas) ===");
    for nt in &g.nonterminals {
        for terminal in &g.terminals {
            if let Some(&index) = g.table.get(&(nt.clone(), terminal.clone())) {
                let p = &g.productions[index];
                let right = if p.right == [EPSILON] {
                    EPSILON.into()
                } else {
                    p.right.join(" ")
                };
                println!(
                    "M[{:<8}, {:<6}] = P{:02}: {} -> {}",
                    nt,
                    terminal,
                    index + 1,
                    p.left,
                    right
                );
            }
        }
    }
    println!();
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.into()
    } else {
        format!("{}...", text.chars().take(max - 3).collect::<String>())
    }
}

fn help() {
    println!("Analisador sintatico descendente preditivo LL(1) manual\n\nUso:\n  cargo run                                  abre a interface web\n  cargo run -- --ui                         abre a interface web\n  cargo run -- --addr=127.0.0.1:8080        escolhe o endereco\n  cargo run -- <arquivo.java> [opcoes]       analisa pelo terminal\n\nOpcoes da CLI:\n  --tokens     mostra a integracao com o analisador lexico\n  --grammar    mostra producoes, FIRST, FOLLOW e Tabela M\n  --tree       mostra a arvore de derivacao em pre-ordem\n  --no-trace   omite o trace passo a passo\n  -h, --help   mostra esta ajuda");
}
