# Analisador Léxico

Trabalho 1 de Compiladores — analisador léxico manual para um subconjunto de Java.

Feito em Rust, sem Lex, Flex, PLY ou ANTLR.

## O que faz

Lê um arquivo `.java` e devolve:

- tabela de tokens (tipo, lexema e atributo)
- tabela de símbolos (identificadores e ocorrências)
- erros léxicos (`1nota`, `7,5`, string sem fechar)

A leitura do fonte usa dois buffers e os ponteiros `begin` e `forward`.

## Como rodar

```bash
# interface no navegador
cargo run

# terminal
cargo run -- exemplos/sem_erro.java
cargo run -- exemplos/com_erro.java
```

A interface abre em http://127.0.0.1:3000

## Estrutura

- `src/token.rs` — tipos de token e palavras reservadas
- `src/buffer.rs` — buffer duplo
- `src/lexer.rs` — reconhecimento dos lexemas
- `src/symbol_table.rs` — tabela de símbolos
- `src/main.rs` — terminal
- `src/web.rs` e `static/index.html` — interface
- `exemplos/` — programas de teste
