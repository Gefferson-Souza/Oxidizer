// Fixture for tyrus check exit-code tests. The `var` keyword is rejected
// by the analyzer (LintVisitor::visit_var_decl → TyrusError::UseOfVar),
// so any path running this through `tyrus check` must exit non-zero.
var legacy: number = 42;
console.log(legacy);
