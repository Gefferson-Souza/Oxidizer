use console::style;

const TYRUS_LOGO: &str = r"
  ████████╗██╗   ██╗██████╗ ██╗   ██╗███████╗
  ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██║   ██║██╔════╝
     ██║    ╚████╔╝ ██████╔╝██║   ██║███████╗
     ██║     ╚██╔╝  ██╔══██╗██║   ██║╚════██║
     ██║      ██║   ██║  ██║╚██████╔╝███████║
     ╚═╝      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
";

pub(crate) fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("{}", style(TYRUS_LOGO).cyan().bold());
    eprintln!(
        "  {} {} — TypeScript to Rust Compiler",
        style("Tyrus").cyan().bold(),
        style(format!("v{version}")).dim(),
    );
    eprintln!(
        "  {}\n",
        style("Safe Transpilation · Zero Runtime · Semantic Preservation").dim(),
    );
}
