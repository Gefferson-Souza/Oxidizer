use console::style;

pub(crate) fn success(msg: &str) -> String {
    format!("{} {}", style("✓").green().bold(), style(msg).green())
}

pub(crate) fn error(msg: &str) -> String {
    format!("{} {}", style("✗").red().bold(), style(msg).red())
}

pub(crate) fn info(msg: &str) -> String {
    format!("{} {}", style("ℹ").blue().bold(), msg)
}

pub(crate) fn warning(msg: &str) -> String {
    format!("{} {}", style("⚠").yellow().bold(), style(msg).yellow())
}

pub(crate) fn step(label: &str, detail: &str) -> String {
    format!("  {} {}", style(label).cyan().bold(), detail)
}

pub(crate) fn file_path(path: &str) -> String {
    style(path).underlined().to_string()
}
