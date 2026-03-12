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
    | ForOfStmt
    | ForInStmt
    | DoWhileStmt
```

## 2. Declarations

```ebnf
InterfaceDecl ::= 'interface' Identifier '{' InterfaceMember* '}'
InterfaceMember ::= Identifier '?'? ':' Type ';'

ClassDecl ::= Decorator* 'class' Identifier '{' ClassMember* '}'
ClassMember ::= PropertyDecl | MethodDecl

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
    | Literal

UnaryExpr         ::= ('!' | '-' | '+') Expression
ArrowExpr         ::= '(' ParamList ')' '=>' (Expression | Block)
AssignExpr        ::= Expression '=' Expression
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
    | 'Promise' '<' Type '>'               (* → Result<T, AppError> *)
    | 'Record' '<' Type ',' Type '>'       (* → HashMap<K, V> *)
    | StringUnionType                      (* → Rust enum *)

StringUnionType ::= StringLiteral ('|' StringLiteral)+
```

## 5. Control Flow (Supported)

```ebnf
WhileStmt   ::= 'while' '(' Expression ')' Block
IfStmt      ::= 'if' '(' Expression ')' Block ('else' Block)?
ForOfStmt   ::= 'for' '(' ('let' | 'const') Identifier 'of' Expression ')' Block
ForInStmt   ::= 'for' '(' ('let' | 'const') Identifier 'in' Expression ')' Block
DoWhileStmt ::= 'do' Block 'while' '(' Expression ')'
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

ArrayLiteral    ::= '[' (Expression (',' Expression)*)? ']'
ObjectLiteral   ::= '{' (Identifier ':' Expression (',' Identifier ':' Expression)*)? '}'
TemplateLiteral ::= '`' (StringPart | '${' Expression '}')* '`'
```

## 7. Decorators (NestJS Subset — Supported)

```ebnf
Decorator ::= '@' Identifier ('(' ArgumentList? ')')?

SupportedDecorators ::=
    | '@Module' '(' ModuleOptions ')'
    | '@Injectable' '(' ')'
    | '@Controller' '(' StringLiteral? ')'
    | '@Get' '(' StringLiteral? ')'
    | '@Post' '(' StringLiteral? ')'
    | '@Put' '(' StringLiteral? ')'
    | '@Delete' '(' StringLiteral? ')'
    | '@Body' '(' ')'
    | '@Param' '(' StringLiteral? ')'
```

## 8. Unsafe (Forbidden)

The following constructs are explicitly rejected by the `tyrus_analyzer` (`LintVisitor`):

- `any` type usage.
- `eval()` calls.
- `var` declarations (use `let`/`const`).
