use std::fs;

use tyrus_di::graph::DiGraph;
use tyrus_diagnostics::TyrusError;

use super::ParsedProject;

pub(super) fn analyze_di_graph(
    parsed: &ParsedProject,
) -> Result<(DiGraph, Vec<String>), TyrusError> {
    let mut graph = DiGraph::new();
    let mut lint_error_count = 0;
    for (program, path) in parsed.programs.iter().zip(&parsed.file_paths) {
        let source_code = fs::read_to_string(path).map_err(TyrusError::IoError)?;
        let file_name = path.to_string_lossy().to_string();

        let analysis_result = tyrus_analyzer::Analyzer::analyze(program, source_code, file_name);
        lint_error_count +=
            crate::gate::render_findings(analysis_result.errors, &analysis_result.diagnostics);
        graph.merge(analysis_result.graph);
    }
    crate::gate::refuse_on_lint_errors(lint_error_count)?;

    let init_order = graph
        .resolve()
        .map_err(|e| TyrusError::Validation(e.to_string()))?;
    Ok((graph, init_order))
}
