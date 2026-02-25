use std::path::PathBuf;

use colored::Colorize;
use getopts::Options;
use itertools::Itertools;
use reqtool::{
    Analysis, diagnostic,
    renderer::{Render, debug},
    syntax,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ProgramDisplayMode::*;

    let start = std::time::Instant::now();

    let args = std::env::args().collect::<Vec<String>>();
    let config = ProgramConfig::try_from(args)?;

    let mut parser = syntax::NodeParser::default();
    let root = parser.parse(&config.input[..])?;
    let context = parser.context;

    handle_syntax_errors(&parser.errors);

    let mut analysis = Analysis::from(&root);
    analysis.diagnostics = match config.propagate {
        false => analysis
            .diagnostics
            .into_iter()
            .unique_by(|e| e.did)
            .collect(),
        true => analysis.diagnostics.into_iter().collect(),
    };

    let result = match config.display {
        Normal => handle_verify_errors(&context, &analysis.diagnostics),
        Verbose | Tree => {
            let rendered = debug::Renderer::new(&analysis).render(&root);
            print!("{}", rendered);
            (config.display == Verbose)
                .then(|| handle_verify_errors(&context, &analysis.diagnostics))
                .unwrap_or(Ok(()))
        }
    };

    println!("All done in {:.2?}", start.elapsed());
    result
}

#[derive(Debug, PartialEq, Eq)]
enum ProgramDisplayMode {
    Normal,
    Verbose,
    Tree,
}

#[derive(Debug)]
struct ProgramConfig {
    propagate: bool,
    display: ProgramDisplayMode,
    input: String,
}

impl TryFrom<Vec<String>> for ProgramConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        use ProgramDisplayMode::*;

        let mut opts = Options::new();
        opts.optflag("t", "tree", "show source tree");
        opts.optflag("p", "propagate", "propagate errors");
        opts.optflag("v", "verbose", "enable verbose mode");
        opts.reqopt("i", "input", "input file", "./somefile");

        let program = args[0].clone();
        let parsed = opts.parse(args.iter().skip(1)).map_err(|err| {
            println!("{}", opts.usage(&format!("Usage: {} [options]", program)));
            Box::new(err)
        })?;

        let propagate = parsed.opt_present("p");
        let display = if parsed.opt_present("v") {
            Verbose
        } else if parsed.opt_present("t") {
            Tree
        } else {
            Normal
        };

        let input = parsed.opt_str("i").unwrap();
        let path = PathBuf::from(input);
        let input = std::fs::read_to_string(&path)?;

        Ok(Self {
            propagate,
            display,
            input,
        })
    }
}

fn handle_verify_errors(
    context: &syntax::ContextMap,
    errors: &Vec<diagnostic::Diagnostic>,
) -> Result<(), Box<dyn std::error::Error>> {
    use diagnostic::DiagnosticSeverity::*;

    match &errors[..] {
        [] => Ok(()),
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
                0 => "No errors".to_string().green().bold(),
                1 => "1 error".to_string().white().bold(),
                n => format!("{} errors", n).white().bold(),
            };

            eprintln!("{}", message);
            Err(format!("Verification failed").into())
        }
    }
}

fn handle_syntax_errors(errors: &Vec<syntax::error::Error>) {
    for error in errors {
        let span = format!("{}", error.span.start).bold().white();
        let err = format!("{}", error.kind).red().bold();
        eprintln!("{:4}{:12}", span, err);

        let message = match errors.len() {
            0 => "No errors".to_string().green().bold(),
            1 => "1 error".to_string().red().bold(),
            n => format!("{} errors", n).red().bold(),
        };

        eprintln!("{}", message);
    }
}
