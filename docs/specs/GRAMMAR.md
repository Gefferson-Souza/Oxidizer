# Tyrus Grammar Specification (Oxidizable Subset)

This document formally defines the subset of TypeScript supported by Tyrus using EBNF notation.

## 1. Fundamentals

```ebnf
Program ::= Statement*

Statement ::=
    | InterfaceDecl
    | ClassDecl
    | FunctionDecl
    | VariableDecl
    | ExpressionStmt
    | ReturnStmt
    | IfStmt
    | WhileStmt
    | ForStmt
    | ForOfStmt
    | DoWhileStmt
    | SwitchStmt
    | TryCatchStmt
    | ThrowStmt
    | TypeAliasDecl
    | EnumDecl
```

## 2. Declarations

```ebnf
InterfaceDecl ::= 'interface' Identifier '{' InterfaceMember* '}'
InterfaceMember ::= Identifier '?'? ':' Type ';'

ClassDecl ::= Decorator* 'class' Identifier ('extends' Identifier)? '{' ClassMember* '}'
ClassMember ::= PropertyDecl | MethodDecl | GetterDecl | SetterDecl | Constructor
GetterDecl ::= 'get' Identifier '(' ')' ':' Type '{' Block '}'
SetterDecl ::= 'set' Identifier '(' Param ')' '{' Block '}'
Constructor ::= 'constructor' '(' ParamList ')' '{' Block '}'

TypeAliasDecl ::= 'type' Identifier '=' Type
EnumDecl ::= 'enum' Identifier '{' EnumMember (',' EnumMember)* '}'
EnumMember ::= Identifier ('=' (NumberLiteral | StringLiteral))?

FunctionDecl ::= 'async'? 'function' Identifier '(' ParamList ')' ':' Type '{' Block '}'
```

## 3. Expressions

```ebnf
Expression ::=
    | BinaryExpr
    | UnaryExpr
    | CallExpr
    | MemberExpr
    | ArrowExpr
    | AssignExpr
    | UpdateExpr
    | OptionalChainExpr
    | TypeAssertionExpr
    | TernaryExpr
    | SpreadExpr
    | NewExpr
    | Literal

TypeAssertionExpr ::= Expression 'as' Type       (* → no-op, compile-time only *)
TernaryExpr       ::= Expression '?' Expression ':' Expression
SpreadExpr        ::= '...' Expression
NewExpr           ::= 'new' Identifier '(' ArgumentList? ')'

UnaryExpr         ::= ('!' | '-' | '+') Expression
ArrowExpr         ::= '(' ParamList ')' '=>' (Expression | Block)
AssignExpr        ::= Expression AssignOp Expression
AssignOp          ::= '=' | '-=' | '*=' | '/=' | '%='
                    | '&=' | '|=' | '^=' | '<<=' | '>>='
UpdateExpr        ::= Expression ('++' | '--') | ('++' | '--') Expression
OptionalChainExpr ::= Expression '?.' (Identifier | CallExpr | MemberExpr)
```

## 4. Types

```ebnf
Type ::=
    | 'string'
    | 'number'
    | 'boolean'
    | 'void'
    | 'null'
    | Identifier                           (* User-defined structs *)
    | Type '[]'                            (* Array → Vec<T> *)
    | 'Array' '<' Type '>'                 (* Array → Vec<T> *)
    | 'Promise' '<' Type '>'               (* → Result<T, AppError> *)
    | 'Record' '<' Type ',' Type '>'       (* → HashMap<K, V> *)
    | 'Map' '<' Type ',' Type '>'          (* → HashMap<K, V> *)
    | 'Set' '<' Type '>'                   (* → HashSet<T> *)
    | 'Date'                                (* → String today; chrono::DateTime planned *)
    | Type '|' 'undefined'                 (* → Option<T> *)
    | StringUnionType                      (* → Rust enum *)

StringUnionType ::= StringLiteral ('|' StringLiteral)+
```

### Built-in stdlib calls recognized

| TypeScript                | Rust                                              |
|---------------------------|---------------------------------------------------|
| `new Map()`               | `HashMap::new()`                                  |
| `new Set()`               | `HashSet::new()`                                  |
| `Date.now()`              | `chrono::Utc::now().timestamp_millis()`           |
| `Math.*` / `JSON.*` / `console.*` / `Object.*` | mapped per `crates/tyrus_codegen/src/stdlib/`     |

## 5. Control Flow (Supported)

```ebnf
WhileStmt   ::= 'while' '(' Expression ')' Block
IfStmt      ::= 'if' '(' Expression ')' Block ('else' (IfStmt | Block))?
ForOfStmt   ::= 'for' '(' ('let' | 'const') Identifier 'of' Expression ')' Block
ForStmt     ::= 'for' '(' VariableDecl? ';' Expression? ';' Expression? ')' Block
DoWhileStmt ::= 'do' Block 'while' '(' Expression ')'
SwitchStmt  ::= 'switch' '(' Expression ')' '{' CaseClause* DefaultClause? '}'
CaseClause  ::= 'case' Expression ':' Statement*
DefaultClause ::= 'default' ':' Statement*
TryCatchStmt ::= 'try' Block 'catch' '(' Identifier ')' Block ('finally' Block)?
ThrowStmt   ::= 'throw' Expression
```

## 6. Literals (Supported)

```ebnf
Literal ::=
    | NumberLiteral
    | StringLiteral
    | BoolLiteral
    | 'null'
    | ArrayLiteral
    | ObjectLiteral
    | TemplateLiteral

ArrayLiteral    ::= '[' (ArrayElement (',' ArrayElement)*)? ']'
ArrayElement    ::= Expression | SpreadExpr

ObjectLiteral   ::= '{' (ObjectMember (',' ObjectMember)*)? '}'
ObjectMember    ::= Identifier ':' Expression          (* explicit         *)
                  | Identifier                          (* shorthand: {x}    *)
                  | SpreadExpr                          (* spread: {...base} *)

TemplateLiteral ::= '`' (StringPart | '${' Expression '}')* '`'
```

`{...base, field: v}` lowers to Rust struct update syntax (`Foo { field: v, ..base }`); `[...a, b]` lowers to iterator chain (`a.iter().cloned().chain(...).collect()`).

## 7. Decorators (NestJS Subset — Supported)

Decorator dispatch is registry-based ([ADR 0007](../architecture/decisions/0007-decorator-registry.md)). The set of recognized names lives in `tyrus_decorator_kinds::DecoratorKind`. Adding a new decorator is a one-variant edit there plus one handler registration in `tyrus_codegen::decorators::default_registry()`; nothing in the legacy match-on-name dispatch survives.

```ebnf
Decorator ::= '@' Identifier ('(' ArgumentList? ')')?

SupportedDecorators ::=
    (* Class-level *)
    | '@Module' '(' ModuleOptions ')'                                  (* analyzer DI graph *)
    | '@Injectable' '(' ')'                                            (* analyzer DI graph *)
    | '@Controller' '(' StringLiteral? ')'                             (* → Axum router scaffolding *)
    | '@UseGuards' '(' Identifier (',' Identifier)* ')'                (* → middleware layers *)

    (* Method-level *)
    | '@Get'    '(' StringLiteral? ')'                                 (* → axum::routing::get *)
    | '@Post'   '(' StringLiteral? ')'                                 (* → axum::routing::post *)
    | '@Put'    '(' StringLiteral? ')'                                 (* → axum::routing::put *)
    | '@Delete' '(' StringLiteral? ')'                                 (* → axum::routing::delete *)
    | '@Patch'  '(' StringLiteral? ')'                                 (* → axum::routing::patch *)
    | '@HttpCode' '(' NumberLiteral ')'                                (* → Result<(StatusCode, Json<T>), AppError>, codes resolved via STATUS_CODES static; unknown codes emit compile_error! *)

    (* Param-level *)
    | '@Body'  '(' ')'                                                 (* → axum::Json<T> *)
    | '@Param' '(' StringLiteral? ')'                                  (* → axum::extract::Path<T> *)
    | '@Query' '(' StringLiteral? ')'                                  (* → axum::extract::Query<T> *)
```

Unknown decorators are *not* an error: they are skipped silently by the registry. Generic-translation behavior for arbitrary decorators is handled outside the NestJS-aware path.

## 8. Unsafe (Forbidden)

The following constructs are explicitly rejected by the `tyrus_analyzer` (`LintVisitor`, 7 rules):

- `any` type usage.
- `eval()` calls.
- `var` declarations (use `let`/`const`).
- `for-in` loops (use `for-of`).
- `delete` operator.
- `with` statements.
- Labeled statements.
