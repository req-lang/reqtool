use std::path::PathBuf;

use getopts::Options;
use reqtool::{
    renderer::{linter, Render},
    syntax,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let config = ProgramConfig::try_from(args)?;

    let mut parser = syntax::NodeParser::default();
    let root = parser.parse(&config.input[..])?;

    let renderer = linter::Renderer::new();
    let result = renderer.render(&root);

    match config.output {
        Some(p) => std::fs::write(p, result)?,
        None => println!("{}", result),
    }

    Ok(())
}

#[derive(Debug)]
struct ProgramConfig {
    input: String,
    output: Option<PathBuf>,
}

impl TryFrom<Vec<String>> for ProgramConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        let mut opts = Options::new();
        opts.reqopt("i", "input", "input file", "./somefile");
        opts.optopt(
            "o",
            "output",
            "output file (stdout by default)",
            "./somefile",
        );

        let program = args[0].clone();
        let parsed = opts.parse(args.iter().skip(1)).map_err(|err| {
            println!("{}", opts.usage(&format!("Usage: {} [options]", program)));
            Box::new(err)
        })?;

        let input = parsed.opt_str("i").unwrap();
        let path = PathBuf::from(input);
        let input = std::fs::read_to_string(&path)?;

        let output = parsed.opt_str("o").map(|p| p.into());

        Ok(Self { output, input })
    }
}
