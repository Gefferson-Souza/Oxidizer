// Fixture for tyrus check --strict behavior. setTimeout goes to
// `result.diagnostics` (Severity::Error Diagnostic from
// UnsupportedApiVisitor), not `result.errors` (TyrusError from
// LintVisitor). Per plan D3, this passes default `tyrus check`
// and fails only under `--strict`.
function delayed(): void {
    setTimeout(() => {
        console.log("delayed");
    }, 100);
}

delayed();
