use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub(crate) struct Pipeline {
    steps: Vec<String>,
    current: usize,
}

impl Pipeline {
    pub(crate) fn new(steps: Vec<&str>) -> Self {
        Self {
            steps: steps.into_iter().map(String::from).collect(),
            current: 0,
        }
    }

    pub(crate) fn start_step(&mut self, detail: &str) {
        if let Some(label) = self.steps.get(self.current) {
            eprintln!(
                "  {} {} {}",
                style(format!("[{}/{}]", self.current + 1, self.steps.len())).dim(),
                style(label).cyan().bold(),
                style(detail).dim(),
            );
        }
        self.current += 1;
    }

    pub(crate) fn finish_success(msg: &str) {
        eprintln!(
            "\n  {} {}\n",
            style("✓").green().bold(),
            style(msg).green().bold()
        );
    }

    pub(crate) fn finish_error(msg: &str) {
        eprintln!(
            "\n  {} {}\n",
            style("✗").red().bold(),
            style(msg).red().bold()
        );
    }
}

// Why allowed: spinner helper exported for the upcoming parallel-task
// progress surface. Sibling of progress() / success() / failure(); removing
// would force re-introducing it on first user of indeterminate progress.
#[allow(dead_code)]
pub(crate) fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(&format!("  {{spinner}} {msg}"))
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
