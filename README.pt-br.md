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

### 📦 Padrões Suportados (Verificados com Testes de Equivalência Semântica)

- **Métodos de String** (16): `includes`, `replace`, `split`, `toUpperCase`, `toLowerCase`, `trim`, `startsWith`, `endsWith`, `toString`, `substring`, `charAt`, `indexOf`, `repeat`, `slice`, `padStart`, `padEnd`
- **Métodos de Array** (15): `map`, `filter`, `forEach`, `find`, `some`, `every`, `reduce`/`fold`, `join`, `includes`, `push`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap`
- **Funções Math** (15): `max`, `min`, `round`, `floor`, `ceil`, `abs`, `random`, `pow`, `sqrt`, `log`, `trunc`, `sign`, `sin`, `cos`, `tan`
- **Constantes Math** (2): `Math.PI`, `Math.E`
- **Console** (5): `log`, `error`, `warn`, `info`, `debug`
- **Fluxo de Controle**: `if/else`, `while`, `for`, `for-of`, `do-while`, `switch/case`, ternário, `try/catch`
- **Top-Level Statements**: `const`, `let`, expressões auto-wrapped em `fn main()`
- **Spread Operator**: `[...arr1, ...arr2]` → iterator chain
- **Herança de Classe**: `extends`, `super()`, override de métodos via field flattening
- **Operadores**: Aritméticos, comparação, lógicos, `**` (exponenciação), `%` (módulo)
- **Estado de Classe**: Encapsulamento automático com `Arc<Mutex<T>>` para services/controllers
- **Interfaces**: `interface` -> `#[derive(Serialize, Deserialize)] struct`
- **Unions de String**: `type Status = "a" | "b"` -> `enum` com `Display` e `PartialEq`

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

### Instalar Globalmente

```bash
cargo install --path crates/tyrus_cli
tyrus --version
```

### Usando o Compilador

```bash
# Analisar um arquivo TypeScript para compatibilidade
tyrus check ./src/index.ts

# Diagnósticos JSON (para integração com ferramentas)
tyrus check --json ./src/index.ts

# Transpilar para código Rust (stdout ou arquivo)
tyrus build ./src/index.ts
tyrus build ./src/index.ts -o output.rs

# Transpilar + compilar para binário nativo
tyrus compile ./src/index.ts --output ./output

# Transpilar + compilar + executar
tyrus run ./src/index.ts --output ./output

# Suprimir banner para scripting
tyrus --quiet check ./src/index.ts
```

---

## 📋 Referência de Comandos

**Comandos de Desenvolvimento:**

<!-- AUTO-GENERATED from Cargo.toml and CI -->
| Comando | Descrição |
|---------|-----------|
| `cargo build --workspace` | Compilar todas as crates do workspace |
| `cargo build --release` | Build de produção com LTO |
| `cargo install --path crates/tyrus_cli` | Instalar CLI `tyrus` globalmente |
| `cargo nextest run --workspace` | Executar todos os testes (paralelo, recomendado) |
| `cargo test --workspace` | Executar todos os testes (runner legado) |
| `cargo test -p integration_tests` | Apenas testes de integração |
| `cargo clippy --workspace` | Lint com regras estritas (`-Dwarnings` aplicado) |
| `cargo fmt -- --check` | Verificar formatação |
| `cargo insta review` | Revisar mudanças em snapshots |

**Comandos CLI do Tyrus (após instalação global):**

| Comando | Descrição |
|---------|-----------|
| `tyrus check <file.ts>` | Analisar compatibilidade Oxidizável |
| `tyrus check --json <file.ts>` | Diagnósticos em JSON |
| `tyrus build <file.ts>` | Transpilar para Rust (stdout) |
| `tyrus build <file.ts> -o output.rs` | Transpilar para Rust (arquivo) |
| `tyrus compile <file.ts> -o <dir>` | Transpilar + compilar para binário nativo |
| `tyrus run <file.ts>` | Transpilar + compilar + executar |
| `tyrus --quiet <command>` | Suprimir banner para scripting |
<!-- /AUTO-GENERATED -->

---

## 🧪 Suite de Testes

170 testes distribuídos em 7 tipos de teste e 4 camadas de funcionalidades:

| Tipo | Quantidade | Descrição |
|------|-----------|-----------|
| **Equivalência** | 67 | Prova semântica: TS e Rust produzem stdout idêntico |
| **CLI** | 7 | Testes de integração para todos os comandos e flags |
| **Unitário** | 27 | Rápido, funções isoladas de codegen |
| **Snapshot** | 6 | Saída completa de transpilação via `insta` |
| **Compilação** | 54 | Rust gerado passa no `cargo check` por camada |
| **IR** | 8 | Lowering de representação intermediária tipada |
| **Trybuild** | 1 | Verificação de compilação do Rust gerado |

Tipos de teste: **Equivalência** (TS↔Rust mesma saída) · **CLI** (integração de comandos) · **Unitário** (funções rápidas e isoladas) · **Snapshot** (insta, saída do codegen) · **Compilação** (Rust gerado passa no `cargo check`) · **IR** (type lowering) · **Trybuild** (verificação de compilação)

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
├── stmt/           — conversão de statements (separado em sub-módulos)
│   ├── mod.rs          — dispatcher + convert_stmt, convert_stmt_recursive
│   ├── var_decl.rs     — declarações de variáveis (ident, desestruturação objeto/array)
│   ├── loops.rs        — while, for-of, for-in, for, do-while
│   └── switch.rs       — switch → match
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
