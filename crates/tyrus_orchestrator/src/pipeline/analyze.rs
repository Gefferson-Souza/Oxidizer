use std::fs;

use tyrus_di::graph::DiGraph;
use tyrus_diagnostics::TyrusError;

use super::ParsedProject;

pub(super) fn analyze_di_graph(
    parsed: &ParsedProject,
) -> Result<(DiGraph, Vec<String>), TyrusError> {
    let mut graph = DiGraph::new();
    for (i, program) in parsed.programs.iter().enumerate() {
        let path = &parsed.file_paths[i];
        let source_code = fs::read_to_string(path).map_err(TyrusError::IoError)?;
        let file_name = path.to_string_lossy().to_string();

        let analysis_result = tyrus_analyzer::Analyzer::analyze(program, source_code, file_name);
        for error in analysis_result.errors {
            println!("Warning: {:?}", miette::Report::new(error));
        }
        if !analysis_result.diagnostics.is_empty() {
            eprintln!(
                "{}",
                tyrus_analyzer::report::format_pretty(&analysis_result.diagnostics)
            );
        }
        graph.merge(analysis_result.graph);
    }

    let init_order = graph
        .resolve()
        .map_err(|e| TyrusError::Validation(e.to_string()))?;
    Ok((graph, init_order))
}
