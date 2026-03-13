# 1. Stack Tecnológica e Estrutura de Monorepo

Data: 2025-11-22
Status: Aceito

## Contexto
O objetivo é criar o **Tyrus**, uma ferramenta que analisa código TypeScript e o transpila para Rust idiomático.
Os requisitos principais são:
1. **Performance:** A ferramenta deve processar grandes projetos rapidamente.
2. **Confiabilidade:** O código gerado deve ser seguro e a ferramenta não deve falhar inesperadamente.
3. **Manutenibilidade:** O projeto deve ser modular para permitir testes isolados de parser, analyzer e codegen.
4. **Ecossistema:** Precisamos de ferramentas robustas para parsing de JS/TS, já que escrever um parser do zero é inviável para o escopo.

## Decisão
Decidimos utilizar a linguagem **Rust** para desenvolver o compilador, organizado em um **Cargo Workspace (Monorepo)**.

### Stack Selecionada:
- **Linguagem:** Rust (Segurança de memória, Sistema de Tipos, Performance).
- **Parsing:** `swc_ecma_parser` (Biblioteca padrão da indústria, escrita em Rust, extremamente rápida).
- **AST Traversal:** `swc_ecma_visit` (Implementação do padrão Visitor para navegação eficiente na árvore).
- **Code Generation:** `quote!` e `proc_macro2` (Geração higiênica de tokens Rust).
- **CLI:** `clap` v4 (Padrão para interfaces de linha de comando).
- **Error Reporting:** `miette` (Diagnósticos ricos com suporte visual ao código fonte).
- **Testes:** `insta` (Snapshot testing) e `trybuild` (Compilation testing).

### Estrutura de Módulos (Crates):
O projeto é dividido em crates isoladas para garantir a separação de responsabilidades (SoC):
- `tyrus_cli`: Interface de entrada (clap).
- `tyrus_parser`: Wrapper sobre o SWC (`swc_ecma_parser`).
- `tyrus_analyzer`: Validação semântica (`LintVisitor` + `DecoratorVisitor`).
- `tyrus_codegen`: Transformação de AST para Tokens Rust (`quote!`).
- `tyrus_orchestrator`: Orquestração do pipeline multi-arquivo.
- `tyrus_di`: Motor de Injeção de Dependência (petgraph).
- `tyrus_diagnostics`: Erros centralizados (`TyrusError` + miette).
- `tyrus_common`: Tipos compartilhados (`FilePath`, utilitários).
- `tyrus_ast`: Reservado para IR futuro (AST do SWC usado diretamente).
- `tyrus_test_utils`: Helpers de teste (`assert_rust_compiles()`).

## Consequências

### Positivas
- **Performance Nativa:** O uso de Rust e SWC garante velocidade superior a ferramentas escritas em JS/TS.
- **Modularidade:** O Monorepo permite compilar e testar o *Analyzer* sem precisar rodar o *Codegen*.
- **Tipagem Forte:** O sistema de tipos do Rust previne erros de lógica interna no compilador.
- **Rigor Acadêmico:** A arquitetura de pipeline (Parse -> Analyze -> Generate) é clássica em teoria de compiladores.

### Negativas
- **Curva de Aprendizado:** A API do SWC é complexa e pouco documentada.
- **Tempo de Compilação:** Rust tem tempos de compilação lentos, o que pode afetar o ciclo de feedback (mitigado pelo uso de crates separadas).