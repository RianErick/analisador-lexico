# Trabalho 2 - Analisador Sintático LL(1) em Rust

Implementação manual de um analisador sintático descendente preditivo dirigido por tabela para o subconjunto de Java definido no enunciado. Não usa geradores como Yacc, Bison, PLY ou ANTLR.

## Recursos implementados

- analisador léxico integrado, que converte identificadores em `id` e inteiros/reais em `num`;
- cálculo automático de FIRST e FOLLOW a partir das produções;
- construção dinâmica da Tabela Preditiva M, com detecção de conflitos LL(1);
- algoritmo de reconhecimento por pilha;
- trace com `Pilha`, `Entrada` e `Ação` em todos os passos;
- mensagens com linha, coluna, token encontrado e símbolos esperados;
- árvore de derivação textual em pré-ordem;
- recuperação de erros em modo pânico, permitindo relatar mais de um erro;
- oito testes automatizados e os três exemplos obrigatórios do enunciado.

## Pré-requisito

Instale o Rust pelo [rustup](https://rustup.rs/). Confira com:

```bash
rustc --version
cargo --version
```

## Como executar

Na raiz deste projeto:

```bash
cargo run -- exemplos/Exemplo1_Sucesso.java
```

Opções disponíveis:

```text
--tokens     exibe os tokens e sua conversão para terminais da gramática
--grammar    exibe produções, FIRST, FOLLOW e células preenchidas da Tabela M
--tree       exibe a árvore de derivação em pré-ordem
--no-trace   omite o trace da pilha
```

Comando completo para a demonstração teórica e o caso aceito:

```bash
cargo run -- exemplos/Exemplo1_Sucesso.java --tokens --grammar --tree
```

Como a saída é extensa, durante o vídeo pode-se usar `--no-trace` ao apresentar FIRST/FOLLOW e depois executar novamente sem `--grammar` para destacar o trace.

## Fluxo dos testes do vídeo

1. Verificar o projeto:

   ```bash
   cargo test
   ```

2. Caso aceito, exibindo integração léxica, teoria e árvore:

   ```bash
   cargo run -- exemplos/Exemplo1_Sucesso.java --tokens --grammar --tree --no-trace
   ```

3. Caso aceito, mostrando o começo e o final do trace:

   ```bash
   cargo run -- exemplos/Exemplo1_Sucesso.java
   ```

4. Erro por célula vazia `M[T, )]`:

   ```bash
   cargo run -- exemplos/Exemplo2_ErroTabela.java
   ```

5. Erro de comparação entre terminal no topo e entrada:

   ```bash
   cargo run -- exemplos/Exemplo3_ErroPilha.java
   ```

6. Diferencial de múltiplos erros com modo pânico:

   ```bash
   cargo run -- exemplos/Exemplo4_MultiplosErros.java --no-trace
   ```

Os comandos com exemplos inválidos terminam com código de saída `1`, propositalmente, porque o programa foi rejeitado.

## O que cada arquivo faz

| Arquivo | Responsabilidade |
|---|---|
| `Cargo.toml` | Metadados do pacote Rust. O projeto não depende de bibliotecas externas. |
| `Cargo.lock` | Registra a resolução reproduzível do pacote pelo Cargo. |
| `src/main.rs` | Interface de terminal: lê argumentos e arquivo `.java`, chama léxico e parser e imprime tokens, gramática, trace, árvore e veredito. |
| `src/token.rs` | Define `TokenKind` e `Token`; faz a ponte entre tokens concretos e terminais abstratos `id`, `num` e `$`. |
| `src/lexer.rs` | Analisador léxico manual: reconhece palavras-chave, identificadores, números, operadores e delimitadores; ignora espaços e comentários. |
| `src/grammar.rs` | Declara as produções e calcula automaticamente não-terminais, terminais, FIRST, FOLLOW e a Tabela M. |
| `src/parser.rs` | Executa o algoritmo LL(1) com pilha, produz trace, árvore, mensagens de erro e recuperação em modo pânico. |
| `exemplos/Exemplo1_Sucesso.java` | Programa válido exigido pelo enunciado. |
| `exemplos/Exemplo2_ErroTabela.java` | Operador `+` sem operando: consulta vazia em `M[T, )]`. |
| `exemplos/Exemplo3_ErroPilha.java` | Parêntese não fechado: topo espera `)`, mas a entrada contém `;`. |
| `exemplos/Exemplo4_MultiplosErros.java` | Caso extra para demonstrar recuperação e múltiplos diagnósticos. |
| `ROTEIRO_VIDEO_10_MIN.md` | Texto cronometrado, comandos e fluxo de telas para a gravação. |

`target/` é criado automaticamente pelo Cargo e não precisa ser enviado. Envie o código-fonte, `Cargo.toml`, `Cargo.lock`, exemplos, README e roteiro.

## Resumo do algoritmo

A pilha começa com `$ PROG`, com `PROG` no topo. Em cada iteração:

1. se o topo é terminal e coincide com a entrada, ambos avançam;
2. se o topo é não-terminal, consulta-se `M[não-terminal, lookahead]`;
3. a produção encontrada substitui o não-terminal, empilhada da direita para a esquerda;
4. uma produção `ε` apenas remove o não-terminal;
5. `$` no topo e na entrada significa aceite;
6. numa célula vazia, o modo pânico descarta a entrada ou sincroniza pelo FOLLOW;
7. numa divergência de terminal, registra-se o terminal ausente e a análise continua.

## Validação

```bash
cargo fmt --check
cargo test
```

Resultado esperado: `8 passed; 0 failed`.
