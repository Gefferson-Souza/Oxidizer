> 🌐 **Idioma:** [English](README.md) | Português (BR)

# Tyrus: Um Compilador TypeScript-para-Rust de Alta Fidelidade

_Projeto Acadêmico em Teoria de Compiladores e Preservação Semântica_

[![CI Status](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml/badge.svg)](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

Tyrus é um compilador source-to-source projetado para conectar a sintaxe dinâmica de alto nível (TypeScript) à segurança de memória de baixo nível (Rust). Como uma iniciativa acadêmica, concentra-se no mapeamento formal de abstrações de ordem superior para equivalentes Rust de custo zero, explorando os limites da **Preservação Semântica** entre modelos de execução distintos.

---

## 🔬 Filosofias Fundamentais

### 🛡️ Zero Magia

Tyrus não depende de um runtime oculto ou emulação de garbage collection. Cada construção TypeScript é mapeada para seu equivalente Rust mais eficiente, aproveitando o modelo de ownership e o sistema de tipos estrito do Rust em tempo de compilação.

### 📐 Preservação Semântica

O objetivo principal é a equivalência formal. Se um projeto TypeScript for "Oxidizável", o código Rust gerado é garantido para manter a integridade semântica da lógica original, ao mesmo tempo em que melhora o desempenho e a segurança.

### 🚫 O Padrão Oxidizável

Tyrus impõe um subconjunto estrito de TypeScript chamado "Padrão Oxidizável". Ele rejeita padrões não idiomáticos ou inseguros (como `any` ou `eval`) para garantir que o código Rust resultante seja seguro e performático.

### 🔐 Arquitetura de Transpilação Segura

Aderindo aos princípios estritos de "Transpilação Segura":

- **Compilação Livre de Panic**: A lógica do compilador utiliza tratamento robusto de erros em vez de entrar em panic com entrada inválida.
- **Linting Estrito**: O código-fonte é verificado com regras `clippy::pedantic` (por exemplo, sem `unwrap()`/`expect()` em caminhos de produção).
- **Mapeamento Formal de AST**: Utiliza Tipos de Dados Algébricos (ADTs) para representar a lógica, evitando vulnerabilidades de manipulação de strings.

---

## 🚀 Camadas de Funcionalidades

### Camada 1: Linguagem Base (Pronto para Produção)

- Primitivos (`string`, `number`, `boolean`)
- Fluxo de Controle (`if/else`, `while`, `for`)
- Tratamento de Erros (`Result`, `Option`)

### Camada 2: Sistema de Tipos Avançado (Pronto para Produção)

- Interfaces e Type Aliases para Structs/Enums
- Generics e Polimorfismo
- Mapeamento Abrangente de Coleções (`Array<T>` -> `Vec<T>`)

### Camada 3: Ecossistema e Assincronia (Pronto para Produção)

- `Async/Await` para concorrência baseada em Future
- Serialização/Desserialização JSON (via `serde`)
- Cliente HTTP e padrões REST (via `axum` & `reqwest`)

### 📦 Padrões Suportados (Verificados)

- **Array Literals**: `[1, 2, 3]` -> `vec![1, 2, 3]`
- **Propriedades Computadas**: `obj["key"]` -> `obj["key"]` (via serde_json)
- **Estado de Classe**: Encapsulamento automático com `Arc<Mutex<T>>` para services/controllers.
- **DTOs**: Structs puros para objetos de transferência de dados.
- **Biblioteca Padrão**: `map`, `filter`, `find`, `push` mapeados para equivalentes Rust.
- **String Replace**: `str.replace(a, b)` -> `str.replacen(a, b, 1)` (Semântica exata do JS).

---

## 🛠 Instalação e Uso

### Pré-requisitos

- Rust 1.75+ (Stable)
- Cargo

### Configuração

```bash
git clone https://github.com/Gefferson-Souza/Tyrus.git
cd Tyrus
cargo build --release
```

### Compilando um Projeto

```bash
# Analisar um arquivo TypeScript para compatibilidade
./target/release/tyrus check ./src/index.ts

# Transpilar para um projeto Rust completo
./target/release/tyrus build ./src/index.ts
```

---

## 📋 Referência de Comandos

<!-- AUTO-GENERATED from Cargo.toml and CI -->
| Comando | Descrição |
|---------|-----------|
| `cargo build --workspace` | Compilar todas as crates do workspace |
| `cargo build --release` | Build de produção com LTO |
| `cargo nextest run --workspace` | Executar todos os testes (paralelo, recomendado) |
| `cargo test --workspace` | Executar todos os testes (runner legado) |
| `cargo test -p integration_tests` | Apenas testes de integração |
| `cargo clippy --workspace` | Lint com regras estritas (`-Dwarnings` aplicado) |
| `cargo fmt -- --check` | Verificar formatação |
| `cargo insta review` | Revisar mudanças em snapshots |
| `cargo run --bin tyrus -- check <file.ts>` | Analisar um arquivo TypeScript para compatibilidade |
| `cargo run --bin tyrus -- build <dir>/src --output <dir>/output` | Transpilar para um projeto Rust completo |
<!-- /AUTO-GENERATED -->

---

## 🧪 Suite de Testes

86 testes distribuídos em 3 tipos de teste e 4 camadas de funcionalidades:

| Camada | Escopo | Testes |
|--------|--------|--------|
| **Camada 1** | Variáveis, matemática, strings, funções, fluxo de controle, console | 34 |
| **Camada 2** | Interfaces, type aliases, arrays, classes, async/await | 12 |
| **Camada 3** | Generics, optional chaining, destructuring, métodos avançados | 18 |
| **Camada 4** | NestJS `@Injectable`, `@Controller`, roteamento Axum, JSON | 7 |

Tipos de teste: **Unitário** (rápido, funções isoladas) · **Snapshot** (insta, saída do codegen) · **Compilação** (Rust gerado passa no `cargo check`)

---

## 📖 Tese e Arquitetura

Para um aprofundamento nos internos do compilador, consulte [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 🏗 Estrutura dos Módulos de Geração de Código

A crate `tyrus_codegen` é organizada em módulos focados e com responsabilidade única em `crates/tyrus_codegen/src/convert/`:

```
convert/
├── mod.rs          — declarações de módulo e re-exports
├── interface.rs    — definição da struct RustGenerator + impl Visit (ponto de entrada)
├── helpers.rs      — utilitários compartilhados: to_snake_case, to_pascal_case, is_string_expr
├── stmt.rs         — conversão de statements (convert_stmt, convert_stmt_recursive)
├── fn_decl.rs      — processamento de declaração de funções (process_fn_decl)
├── module.rs       — manipulação de módulos/imports
├── type_mapper.rs  — mapeamento de tipos TypeScript → Rust (map_type_core deduplicado)
├── class/          — class → struct+impl (separado do monolítico class.rs)
│   ├── mod.rs          — dispatcher + conversão de propriedades
│   ├── constructor.rs  — transpilação de construtores + DI
│   ├── method.rs       — transpilação de métodos + decorators
│   ├── routing.rs      — geração de router Axum + FromRequestParts
│   └── mutation.rs     — detecção de self-mutation
└── expr/
    ├── mod.rs      — dispatcher de expressões (convert_expr)
    ├── binary.rs   — operadores binários (convert_bin_expr)
    ├── call.rs     — chamadas de função/método, métodos axios/fetch/array
    ├── member.rs   — acesso a propriedades, estado mutex (convert_member_expr)
    ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
    ├── literal.rs  — literais, expressões de objeto/array/template
    └── misc.rs     — atribuições, atualizações, optional chaining
```

Todo código Rust é gerado utilizando macros `quote!` produzindo `proc_macro2::TokenStream` — nunca concatenação de strings.

## 📄 Licença

Licença MIT. Consulte [LICENSE](LICENSE) para detalhes.
