# Plano de Padronização e Versionamento (Academic & Engineering Standard)

Este documento define as regras para elevar o **Tyrus** (antigo Tyrus) de um projeto experimental para um _Software Acadêmico e de Engenharia de Classe Mundial_.

## 1. Estratégia de Versionamento (SemVer)

Adotaremos o **Semantic Versioning 2.0.0** (`MAJOR.MINOR.PATCH`).

- **MAJOR (0.x -> 1.x):** Breaking changes na CLI ou na API pública (o que afeta o usuário final).
- **MINOR (0.1 -> 0.2):** Novas funcionalidades compatíveis (ex: suporte a loops, novas transformações).
- **PATCH (0.1.0 -> 0.1.1):** Bug fixes e melhorias internas que não mudam a interface.

### Workflow de Release

1.  **Develop:** Todo PR mergeado na `main` é considerado "instável" (alpha/beta).
2.  **Release:** Criaremos Tags no Git (`v0.1.0`).
3.  **Cargo:** O arquivo `Cargo.toml` na raiz (workspace) é a fonte da verdade da versão.

## 2. Padrão Acadêmico de Documentação

Para um projeto ser considerado "Academic Standard", ele precisa de **Rigor** e **Reprodutibilidade**.

### 2.1 Especificação Formal

Não basta o código funcionar. Precisamos definir _o que_ é a linguagem suportada.

- **Ação:** Criar `docs/specs/GRAMMAR.md` contendo a EBNF (Extended Backus-Naur Form) do subconjunto TypeScript suportado ("Oxidizable Standard").

### 2.2 ADRs (Architecture Decision Records)

Continuar o uso massivo de ADRs. Cada decisão complexa (ex: por que `tokio`? por que `swc`?) deve ter um ADR. Isso serve como a "memória" da pesquisa.

### 2.3 Benchmarks

Trabalhos acadêmicos exigem métricas.

- **Ação:** Criar uma suite de benchmarks (`benches/`) comparando:
  - Tempo de Execução: Node.js vs Rust (Gerado).
  - Memória: Node.js vs Rust.
  - Isso validará a tese de "Performance e Segurança".

## 3. Política de Idiomas (Internationalization)

A ciência da computação e a engenharia global falam **Inglês**.

- **Código (Variáveis/Funções):** `STRICTLY ENGLISH`.
- **Comentários de Código:** `STRICTLY ENGLISH`.
- **Commits:** `STRICTLY ENGLISH` (Imperative mood: "Add feature", not "Adicionei feature").
- **Documentação Oficial (Docs/Architecture):** `ENGLISH`.

### A Exceção Brasileira (PR-BR)

Como o projeto tem raízes brasileiras, podemos ter acessibilidade, mas separada:

- `README.md` -> Inglês (Padrão).
- `README.pt-br.md` -> Português (Opcional, mantido via tradução).
- Dispersar comentários em PT-BR no código é considerado **code smell** em projetos internacionais.

## 4. Estrutura de Diretórios (Refinamento)

Manteremos a estrutura de Workspace, mas com adições acadêmicas:

```text
/
├── benches/                  # [NEW] Benchmarks científicos
├── docs/
│   ├── specs/                # [NEW] EBNF e Formalismos
│   ├── architecture/         # ADRs e Design Docs
│   └── translations/         # [NEW] Docs em PT-BR (se necessário)
├── papers/                   # [NEW] Rascunhos de artigos/tcc
└── ...
```

## ✅ Plano de Execução

- [x] **Fase 1: Definições (Meta)**
  - [x] Adotar SemVer no `Cargo.toml`.
  - [x] Criar `docs/specs/GRAMMAR.md` (Esqueleto).
- [x] **Fase 4: Padronização de Código (Qualidade de Engenharia)**
  - [x] `.cargo/config.toml` com `-Dwarnings` e clippy lint estritos (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`).
  - [x] `clippy.toml` com limiares de complexidade cognitiva (15), linhas por função (50) e parâmetros (5).
  - [x] `deny.toml` para auditoria de dependências e licenças via `cargo-deny`.
  - [x] CI atualizado: `actions/checkout@v4`, `Swatinem/rust-cache@v2`, `cargo nextest`, verificação E2E do demo.
  - [x] Toda base de código livre de `panic!()`, `.expect()`, e `todo!()`.
  - [x] `tyrus_ast` reservado para IR futuro (arquivo `lib.rs` com comentário explicativo).
  - [x] `tyrus_analyzer/graph.rs` removido (código morto).
- [x] **Fase 5: Decomposição do Módulo de Geração de Código**
  - [x] `func.rs` (1144 linhas) decomposto em 10 módulos focados:
    - `helpers.rs`, `stmt.rs`, `fn_decl.rs`
    - `expr/` com: `mod.rs`, `binary.rs`, `call.rs`, `member.rs`, `arrow.rs`, `literal.rs`, `misc.rs`
  - [x] `func.rs` deletado após migração completa.
  - [x] Infraestrutura de testes reconstruída em tiers (unit/snapshot/compilation).
- [ ] **Fase 2: Benchmarking (Evidência)**
  - [ ] Configurar `criterion` (crate de benchmark Rust).
  - [ ] Criar cenário de teste comparativo.
- [ ] **Fase 3: Tradução (Acessibilidade)**
  - [ ] Criar `README.pt-br.md`.
- [ ] **Fase 6: Decomposição de `class.rs`**
  - [ ] `class.rs` (1033 linhas) ainda precisa ser decomposto em módulos menores.
  - [ ] Alvo: módulos separados para NestJS controller mapping, service patterns e DTO handling.
