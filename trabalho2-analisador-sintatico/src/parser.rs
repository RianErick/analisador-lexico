use crate::grammar::{Grammar, EPSILON};
use crate::token::{Token, TokenKind};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TraceStep {
    pub stack: String,
    pub input: String,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug)]
struct TreeNode {
    symbol: String,
    lexeme: Option<String>,
    children: Vec<usize>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub accepted: bool,
    pub errors: Vec<SyntaxError>,
    pub trace: Vec<TraceStep>,
    tree: Vec<TreeNode>,
}

impl ParseResult {
    /// Arvore textual em pre-ordem: pai antes dos filhos.
    pub fn preorder_tree(&self) -> String {
        fn visit(nodes: &[TreeNode], id: usize, depth: usize, out: &mut String) {
            let node = &nodes[id];
            out.push_str(&"  ".repeat(depth));
            out.push_str(&node.symbol);
            if let Some(lexeme) = &node.lexeme {
                if lexeme != &node.symbol {
                    out.push_str(&format!(" -> {lexeme}"));
                }
            }
            out.push('\n');
            for &child in &node.children {
                visit(nodes, child, depth + 1, out);
            }
        }
        let mut out = String::new();
        if !self.tree.is_empty() {
            visit(&self.tree, 0, 0, &mut out);
        }
        out
    }
}

#[derive(Debug)]
struct StackItem {
    symbol: String,
    node: Option<usize>,
}

pub struct Parser<'a> {
    grammar: &'a Grammar,
}

impl<'a> Parser<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        Self { grammar }
    }

    pub fn parse(&self, tokens: &[Token]) -> ParseResult {
        let mut stack = vec![
            StackItem {
                symbol: "$".into(),
                node: None,
            },
            StackItem {
                symbol: self.grammar.start.clone(),
                node: Some(0),
            },
        ];
        let mut tree = vec![TreeNode {
            symbol: self.grammar.start.clone(),
            lexeme: None,
            children: vec![],
        }];
        let mut trace = Vec::new();
        let mut errors = Vec::new();
        let mut position = 0;

        while let Some(top) = stack.last() {
            let token = tokens
                .get(position)
                .unwrap_or_else(|| tokens.last().expect("o lexico sempre produz EOF"));
            let lookahead = token.terminal();
            let stack_view = stack
                .iter()
                .map(|item| item.symbol.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let input_view = tokens[position..]
                .iter()
                .map(Token::terminal)
                .take(14)
                .collect::<Vec<_>>()
                .join(" ");

            if top.symbol == "$" && lookahead == "$" {
                trace.push(TraceStep {
                    stack: stack_view,
                    input: input_view,
                    action: "aceitar: $ casa com $".into(),
                });
                stack.pop();
                break;
            }

            if !self.grammar.nonterminals.contains(&top.symbol) {
                if top.symbol == lookahead {
                    let item = stack.pop().unwrap();
                    if let Some(node) = item.node {
                        tree[node].lexeme = Some(token.lexeme.clone());
                    }
                    trace.push(TraceStep {
                        stack: stack_view,
                        input: input_view,
                        action: format!("casar terminal '{}'", lookahead),
                    });
                    position += 1;
                } else {
                    let expected = top.symbol.clone();
                    let item = stack.pop().unwrap();
                    if let Some(node) = item.node {
                        tree[node].lexeme = Some("<ausente>".into());
                    }
                    let message = format!(
                        "encontrado '{}', mas esperava-se '{}' no topo da pilha",
                        token.lexeme, expected
                    );
                    errors.push(SyntaxError {
                        line: token.line,
                        column: token.column,
                        message: message.clone(),
                    });
                    trace.push(TraceStep {
                        stack: stack_view,
                        input: input_view,
                        action: format!("ERRO: {message}; inserir '{}' e continuar", expected),
                    });
                }
                continue;
            }

            let nonterminal = top.symbol.clone();
            if let Some(&production_index) = self
                .grammar
                .table
                .get(&(nonterminal.clone(), lookahead.clone()))
            {
                let item = stack.pop().unwrap();
                let production = &self.grammar.productions[production_index];
                let right_text = production.right.join(" ");
                let mut children = Vec::new();
                if let Some(parent) = item.node {
                    for symbol in &production.right {
                        let id = tree.len();
                        tree.push(TreeNode {
                            symbol: symbol.clone(),
                            lexeme: None,
                            children: vec![],
                        });
                        children.push(id);
                    }
                    tree[parent].children = children.clone();
                }
                for (symbol, node) in production.right.iter().zip(children.iter()).rev() {
                    if symbol != EPSILON {
                        stack.push(StackItem {
                            symbol: symbol.clone(),
                            node: Some(*node),
                        });
                    }
                }
                trace.push(TraceStep {
                    stack: stack_view,
                    input: input_view,
                    action: format!("aplicar {} -> {}", production.left, right_text),
                });
            } else {
                let expected = self.grammar.expected(&nonterminal);
                let message = format!(
                    "token '{}' inesperado; esperava-se {} para derivar {}",
                    token.lexeme,
                    quoted_list(&expected),
                    nonterminal
                );
                errors.push(SyntaxError {
                    line: token.line,
                    column: token.column,
                    message: message.clone(),
                });
                if self.grammar.follow[&nonterminal].contains(&lookahead) || lookahead == "$" {
                    stack.pop();
                    trace.push(TraceStep {
                        stack: stack_view,
                        input: input_view,
                        action: format!(
                            "ERRO: {message}; modo panico: sincronizar removendo {nonterminal}"
                        ),
                    });
                } else {
                    trace.push(TraceStep {
                        stack: stack_view,
                        input: input_view,
                        action: format!(
                            "ERRO: {message}; modo panico: descartar '{}'",
                            token.lexeme
                        ),
                    });
                    position += 1;
                }
            }
        }

        // Erros lexicos tambem impedem o aceite, mesmo se o modo panico chegar ao fim.
        for token in tokens.iter().filter(|t| t.kind == TokenKind::Error) {
            if !errors
                .iter()
                .any(|e| e.line == token.line && e.column == token.column)
            {
                errors.push(SyntaxError {
                    line: token.line,
                    column: token.column,
                    message: format!("token lexico invalido '{}'", token.lexeme),
                });
            }
        }
        ParseResult {
            accepted: errors.is_empty() && stack.is_empty(),
            errors,
            trace,
            tree,
        }
    }
}

fn quoted_list(items: &[String]) -> String {
    match items {
        [] => "nenhum terminal valido".into(),
        [only] => format!("'{only}'"),
        _ => {
            let mut quoted: Vec<_> = items.iter().map(|s| format!("'{s}'")).collect();
            let last = quoted.pop().unwrap();
            format!("{} ou {}", quoted.join(", "), last)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> ParseResult {
        let grammar = Grammar::java_subset().unwrap();
        Parser::new(&grammar).parse(&Lexer::analyze(source))
    }

    #[test]
    fn accepts_valid_program() {
        let result =
            parse("public class T { public static void main() { int a; a = 2 + 3 * 4; } }");
        assert!(result.accepted, "{:?}", result.errors);
    }

    #[test]
    fn reports_empty_table_cell_after_plus() {
        let result = parse("public class T { public static void main() { int a; a = (1 + ); } }");
        assert!(!result.accepted);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("token ')' inesperado")
                    && e.message.contains("derivar T"))
        );
    }

    #[test]
    fn reports_terminal_mismatch_for_missing_parenthesis() {
        let result = parse("public class T { public static void main() { int a; a = (1 + 2; } }");
        assert!(!result.accepted);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("encontrado ';'")
                    && e.message.contains("esperava-se ')'"))
        );
    }

    #[test]
    fn panic_mode_can_find_multiple_errors() {
        let result = parse("public class T { public static void main() { int = a; a = (1 + ); } }");
        assert!(result.errors.len() >= 2, "{:?}", result.errors);
    }
}
