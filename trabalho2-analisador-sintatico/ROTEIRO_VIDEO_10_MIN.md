# Roteiro de apresentação - aproximadamente 10 minutos

Antes de gravar: abra o projeto no editor, deixe um terminal na raiz e aumente a fonte. Substitua `[SEU NOME]` e, se houver, `[NOME DA DUPLA]`. O texto abaixo foi escrito para ser falado naturalmente; não é necessário lê-lo de modo mecânico.

## 0:00-0:35 - Identificação e objetivo

**Tela:** README ou título do projeto.

**Fala:**

"Olá, meu nome é [SEU NOME] [e este trabalho foi desenvolvido com NOME DA DUPLA]. Este é o Trabalho 2 da disciplina: uma implementação manual, em Rust, de um analisador sintático descendente preditivo dirigido por tabela LL(1). Ele recebe um arquivo Java, usa o analisador léxico para gerar tokens e depois verifica se a sequência pertence à gramática simplificada fornecida. Não utilizei nenhum gerador automático de parser."

## 0:35-1:30 - Fundamento LL(1) e gramática

**Tela:** `src/grammar.rs`, função `java_subset`.

**Fala:**

"LL(1) significa que a entrada é lida da esquerda para a direita, produzindo uma derivação mais à esquerda, com apenas um token de antecipação. A gramática está representada aqui como uma lista de produções. O símbolo inicial é PROG. CMDS pode produzir um comando seguido de outros comandos ou épsilon. As expressões foram fatoradas em E, E linha, T, T linha e F. Essa forma representa precedência: multiplicação e divisão ficam em T, enquanto soma e subtração ficam em E. A recursão está à direita, o que torna a gramática apropriada para o método preditivo."

## 1:30-2:40 - FIRST, FOLLOW e Tabela M

**Tela/ação:** execute:

```bash
cargo run -- exemplos/Exemplo1_Sucesso.java --grammar --no-trace
```

Mostre rapidamente FIRST, FOLLOW e algumas células da tabela.

**Fala:**

"O conjunto FIRST de um não-terminal contém os terminais que podem iniciar alguma cadeia derivada dele. Por exemplo, FIRST de F é abre parêntese, id ou num. FIRST de CMDS também possui épsilon, porque um bloco pode não conter comandos. FOLLOW contém o que pode aparecer logo depois do não-terminal. Por exemplo, no FOLLOW de E aparecem ponto e vírgula, fecha parêntese e operadores relacionais, conforme o contexto da expressão.

Neste projeto, FIRST e FOLLOW não foram digitados prontos: os algoritmos iteram até não surgir nenhum novo elemento. Depois, a Tabela M também é construída dinamicamente. Para cada produção A produz alfa, os terminais de FIRST de alfa recebem essa produção. Se alfa deriva épsilon, também são usadas as colunas de FOLLOW de A. O programa ainda acusa um conflito caso duas produções tentem ocupar a mesma célula; como esta gramática é LL(1), isso não acontece. Para facilitar a leitura, são impressas apenas as células preenchidas."

## 2:40-3:35 - Integração com o léxico

**Tela:** `src/token.rs` e depois `src/lexer.rs`.

**Fala:**

"A análise começa no léxico. O arquivo token define a categoria, o lexema, a linha e a coluna. A função terminal faz a adaptação pedida no enunciado: qualquer identificador, como o nome da classe, main ou uma variável, vira o terminal id; números inteiros e reais viram num; e o fim do arquivo vira cifrão. Os demais símbolos usam o próprio lexema.

No arquivo lexer, o código percorre os caracteres, ignora espaços e comentários e reconhece palavras reservadas, identificadores, números, operadores e delimitadores. Assim, o parser trabalha com tipos de token e não com nomes específicos. Main não foi colocado entre as palavras reservadas, então ele chega corretamente como id, conforme a produção inicial da gramática."

## 3:35-4:50 - Parser, pilha, trace e árvore

**Tela:** `src/parser.rs`, dentro de `parse`.

**Fala:**

"O parser começa com cifrão e PROG na pilha. O topo está à direita. Em cada passo ele compara o topo com o lookahead, que é o token atual da entrada. Se o topo é um terminal igual à entrada, ocorre o casamento e a entrada avança. Se é um não-terminal, o programa consulta a Tabela M. A produção escolhida é colocada na pilha em ordem inversa, para que o primeiro símbolo da produção fique no topo.

Cada iteração registra uma linha do trace com a pilha, a entrada restante e a ação realizada. Durante a expansão, também são criados nós ligados ao pai. A função preorder_tree visita primeiro o nó e depois seus filhos, gerando a árvore textual em pré-ordem. Quando tanto o topo quanto a entrada são cifrão, a cadeia é aceita."

## 4:50-6:15 - Exemplo 1: sucesso

**Tela:** abra `exemplos/Exemplo1_Sucesso.java` e mostre declarações, expressão e if-else. Depois execute:

```bash
cargo run -- exemplos/Exemplo1_Sucesso.java --tokens --tree --no-trace
```

Em seguida execute o trace e mostre o começo e o final, sem precisar ler todas as linhas:

```bash
cargo run -- exemplos/Exemplo1_Sucesso.java
```

**Fala:**

"O primeiro exemplo é válido e cobre declarações int e float, atribuições, parênteses, precedência das quatro operações e if-else. Na tabela de tokens podemos ver, por exemplo, TesteSucesso convertido para id e 2.5 convertido para num. A árvore mostra a derivação a partir de PROG.

Agora, no trace, a primeira expansão é PROG para o cabeçalho completo. Os terminais public e class são casados, e as expansões continuam. Mais adiante, a multiplicação é reconhecida em T linha, comprovando a precedência. No final, a pilha e a entrada chegam a cifrão, o parser executa a ação aceitar e apresenta o veredito Aceito com Sucesso."

## 6:15-7:15 - Exemplo 2: erro na Tabela M

**Tela:** abra a linha `b = (a + );` e execute:

```bash
cargo run -- exemplos/Exemplo2_ErroTabela.java
```

**Fala:**

"No segundo exemplo existe um operador de soma sem operando à direita. Depois de consumir o mais pela produção E linha produz mais T E linha, o topo passa a ser T. Porém, o lookahead é fecha parêntese. A célula M de T com fecha parêntese está vazia, porque um termo só pode começar por abre parêntese, id ou num. A saída informa exatamente o token inesperado, os terminais esperados, o não-terminal T, a linha e a coluna. O programa é rejeitado."

## 7:15-8:10 - Exemplo 3: erro no topo da pilha

**Tela:** abra a linha `a = (10 + 5;` e execute:

```bash
cargo run -- exemplos/Exemplo3_ErroPilha.java
```

**Fala:**

"O terceiro exemplo abre um parêntese, mas não o fecha. Ao aplicar F produz abre parêntese E fecha parêntese, o fecha parêntese permanece guardado na pilha. A expressão interna termina e o topo passa a ser esse terminal, mas a entrada atual é ponto e vírgula. Aqui o erro não é uma consulta vazia da tabela: é uma diferença entre o terminal esperado no topo e o token encontrado. A mensagem diz que encontrou ponto e vírgula, mas esperava fecha parêntese, novamente com linha e coluna."

## 8:10-8:55 - Diferenciais: recuperação e múltiplos erros

**Tela:** execute:

```bash
cargo run -- exemplos/Exemplo4_MultiplosErros.java --no-trace
```

**Fala:**

"Também implementei recuperação por modo pânico. Se a célula da tabela está vazia e o lookahead pertence ao FOLLOW do não-terminal, o parser remove esse não-terminal para sincronizar. Caso contrário, descarta tokens da entrada até encontrar um ponto seguro. Numa divergência de terminal, ele registra uma inserção do terminal esperado e continua. Por isso este arquivo, que possui vários defeitos, produz múltiplos diagnósticos em uma única execução, em vez de encerrar no primeiro erro. Algumas mensagens adicionais são consequência natural da recuperação local."

## 8:55-9:35 - Estrutura de arquivos e testes automatizados

**Tela:** árvore de arquivos no editor e terminal. Execute:

```bash
cargo test
```

**Fala:**

"Resumindo os arquivos: main controla a linha de comando e a impressão; token define os tokens e a conversão para terminais; lexer faz a análise léxica; grammar contém produções e os algoritmos de FIRST, FOLLOW e Tabela M; parser contém pilha, trace, árvore e recuperação. A pasta exemplos guarda os três casos obrigatórios e um caso extra. O projeto não usa dependências externas. Aqui também estão oito testes automatizados: dois do léxico, dois da gramática e quatro do parser. Todos passam."

## 9:35-10:00 - Encerramento

**Tela:** resultado `8 passed` ou README.

**Fala:**

"Portanto, o trabalho integra léxico e sintático, calcula FIRST e FOLLOW, constrói a tabela automaticamente, mostra o trace completo e produz um veredito claro. Como diferenciais, inclui a árvore em pré-ordem, detecção de conflito LL(1), modo pânico e testes automatizados. Obrigado."

## Checklist de gravação e entrega

- trocar os nomes no início do roteiro;
- mostrar nome(s) do(s) aluno(s) no primeiro quadro;
- deixar o terminal largo para não quebrar excessivamente o trace;
- executar os três exemplos obrigatórios;
- não gastar tempo lendo o trace inteiro: mostre começo, um trecho relevante e o final;
- confirmar áudio e legibilidade antes de publicar;
- enviar o código-fonte sem a pasta `target/`;
- publicar no YouTube e entregar o link no Classroom.
