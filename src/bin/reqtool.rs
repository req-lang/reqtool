// reqtool - Compiler and tooling for the req language
// Copyright (C) 2021-2026  Sami Dahoux
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// For commercial licensing, see COMMERCIAL.md

use std::path::PathBuf;

use clap::Parser as _;
use colored::Colorize;
use reqtool::{
    Analysis,
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

            for error in &parser.errors {
                let span = format!("{}", error.span.start).bold().white();
                let err = format!("{}", error.kind).red().bold();
                eprintln!("{:4}{:12}", span, err);
            }

            if !parser.errors.is_empty() {
                eprintln!("{} syntax error(s)", parser.errors.len())
            }

            let mut analysis = Analysis::from(&root);
            if propagate {
                analysis.diagnostics.dedup_by_key(|e| e.did);
            }

            if tree || verbose {
                let rendered = debug::Renderer::new(&analysis).render(&root);
                print!("{}", rendered);
            }

            if !tree {
                for error in &analysis.diagnostics {
                    let context = context.get(&error.id).unwrap();
                    let span = format!("{}", context.span.start).bold().white();
                    let id = format!("{}", error.id).purple();

                    let err = format!("{}", error.kind).bold();
                    let colored = error.severity.colored(&err);
                    eprintln!("{} {}", colored, id);
                    eprintln!("{:4}{:12}", "", span);
                }

                if !analysis.diagnostics.is_empty() {
                    eprintln!("{} diagnostic(s)", analysis.diagnostics.len())
                }
            }
        }
    }

    println!("All done in {:.2?}", start.elapsed());
}
