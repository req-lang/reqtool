use std::path::PathBuf;

use clap::Parser as _;
use colored::Colorize;
use reqtool::{
    Analysis, diagnostic,
    renderer::{Render, debug, formatter},
    syntax,
};

#[derive(clap::Parser)]
#[command(name = "reqtool", about = "Tooling for the req language")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Format a req source file
    Fmt {
        /// Input file
        input: PathBuf,
        /// Output file (stdout by default)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check a req source file for errors
    Check {
        /// Input file
        input: PathBuf,
        /// Show source tree
        #[arg(short, long)]
        tree: bool,
        /// Enable verbose mode (show tree and errors)
        #[arg(short, long)]
        verbose: bool,
        /// Propagate errors
        #[arg(short, long)]
        propagate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let start = std::time::Instant::now();

    match cli.command {
        Command::Fmt { input, output } => {
            let source = std::fs::read_to_string(&input).expect("Failed to read input");

            let mut parser = syntax::NodeParser::default();
            let root = parser.parse(&source[..]).expect("Failed to parse input");

            let renderer = formatter::Renderer::new();
            let result = renderer.render(&root);

            match output {
                Some(p) => std::fs::write(p, result).expect("Failed to write input"),
                None => print!("{}", result),
            }
        }
        Command::Check {
            input,
            tree,
            verbose,
            propagate,
        } => {
            let source = std::fs::read_to_string(&input).expect("Failed to read input");

            let mut parser = syntax::NodeParser::default();
            let root = parser.parse(&source[..]).expect("Failed to parse input");
            let context = parser.context;

            print_syntax_errors(&parser.errors);

            let mut analysis = Analysis::from(&root);
            if propagate {
                analysis.diagnostics.dedup_by_key(|e| e.did);
            }

            if tree || verbose {
                let rendered = debug::Renderer::new(&analysis).render(&root);
                print!("{}", rendered);
            }

            if !tree {
                print_diagnostics(&context, &analysis.diagnostics)
            }
        }
    }

    println!("All done in {:.2?}", start.elapsed());
}

fn print_diagnostics(context: &syntax::ContextMap, errors: &Vec<diagnostic::Diagnostic>) {
    use diagnostic::DiagnosticSeverity::*;

    match &errors[..] {
        [] => {}
        errors => {
            for error in errors {
                let context = context.get(&error.id).unwrap();
                let span = format!("{}", context.span.start).bold().white();
                let id = format!("{}", error.id).purple();

                let err = format!("{}", error.kind).bold();
                let colored = match error.severity {
                    Critical => err.white().on_red(),
                    Severe => err.red(),
                    Moderate => err.yellow(),
                    Light => err.white(),
                };
                eprintln!("{} {}", colored, id);
                eprintln!("{:4}{:12}", "", span);
            }
            let message = match errors.len() {
                1 => "1 error".to_string().white().bold(),
                n => format!("{} errors", n).white().bold(),
            };

            eprintln!("{}", message);
        }
    }
}

fn print_syntax_errors(errors: &Vec<syntax::error::Error>) {
    for error in errors {
        let span = format!("{}", error.span.start).bold().white();
        let err = format!("{}", error.kind).red().bold();
        eprintln!("{:4}{:12}", span, err);
    }
    if !errors.is_empty() {
        let message = match errors.len() {
            1 => "1 error".to_string().red().bold(),
            n => format!("{} errors", n).red().bold(),
        };
        eprintln!("{}", message);
    }
}
