use std::ops::ControlFlow;

use colored::Colorize;

use crate::{
    Analysis,
    children::{Children, ChildrenIter},
    diagnostic,
    syntax::{self},
    verifier::{reference::is_builtin, typing::TypeKind},
    visitor::{Visitor, Walk, WalkCustom},
};

use super::Render;

pub struct Renderer<'a> {
    indentation: usize,
    analysis: &'a Analysis,
    pub result: String,
}

impl Walk<&syntax::entity::Entity> for Renderer<'_> {
    fn walk(&mut self, node: &syntax::entity::Entity) -> ControlFlow<()> {
        self.visit(node)?;
        if let Some(children) = node.children() {
            for child in children {
                self.walk(child)?;
            }
            if let None = children.iter().last().and_then(|c| c.children()) {
                self.result.push_str("\n");
            }
        }
        ControlFlow::Continue(())
    }
}

impl Visitor<&syntax::entity::Entity> for Renderer<'_> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &syntax::entity::Entity) -> ControlFlow<()> {
        use syntax::entity::EntityVariant::*;

        let path = &self.analysis.paths.get(&node.id).unwrap();
        self.indentation = path.depth() - 1;
        let kind = match node.variant {
            Part(_) => "part",
            Attribute(_) => "attribute",
            Package(_) => "package",
            Requirement(_) => "requirement",
        }
        .blue()
        .bold();
        let id = format!("{}", node.id).purple();
        let parent = path.parent().unwrap_or("");
        let label = path.last().white().bold();
        let result = format!("{} {} {} {:<12}", kind, label, parent, id);
        self.push_line(result);

        self.indentation += 1;

        let mut errors: Vec<_> = self
            .analysis
            .diagnostics
            .iter()
            .filter(|e| e.id == node.id)
            .collect();
        errors.sort_by_key(|e| &e.kind);
        if !errors.is_empty() {
            let title = "*** errors ***".white().bold();
            self.push_line(title.to_string());
            for err in errors {
                use diagnostic::DiagnosticSeverity::*;

                let line = format!("- {}", err).bold();
                let colored = match err.severity {
                    Critical => line.white().on_white(),
                    Severe => line.red(),
                    Moderate => line.yellow(),
                    Light => line.white(),
                };

                self.push_line(colored.to_string());
            }
        }

        if !node.meta.tags.is_empty() {
            let title = "*** tags ***".white().bold();
            self.push_line(title.to_string());
            for tag in &node.meta.tags {
                match &tag.value {
                    Some(v) => self.push_line(format!("# {} -> {}", tag.id, v)),
                    None => self.push_line(format!("# {}", tag.id)),
                }
            }
        }

        let references: Vec<_> = node.references().collect();
        if !references.is_empty() {
            let title = "*** references ***".white().bold();

            self.push_line(title.to_string());
            for reference in references {
                let raw = &reference.value;
                let rid = reference.rid;
                let line = match self.analysis.references.get(&rid) {
                    Some(target) => {
                        let path = self.analysis.paths.get(target).unwrap();
                        format!("- {} -> {}", raw, format!("{}", path).bright_green())
                    }
                    None => format!("- {} -> -", raw).to_string(),
                };

                self.push_line(line);
            }
        }

        for expression in node.expressions() {
            self.walk(expression)?;
        }

        self.indentation -= 1;
        ControlFlow::Continue(())
    }
}

impl Walk<&syntax::expression::Expression> for Renderer<'_> {
    fn walk(&mut self, node: &syntax::expression::Expression) -> ControlFlow<()> {
        self.visit(node)?;
        self.indentation += 1;
        for child in node.children_iter() {
            self.walk(child)?;
        }
        self.indentation -= 1;
        ControlFlow::Continue(())
    }
}

impl Visitor<&syntax::expression::Expression> for Renderer<'_> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &syntax::expression::Expression) -> ControlFlow<()> {
        use syntax::expression::ExpressionVariant::*;

        let kind = match node.variant {
            Branch(_) => "branch",
            When(_) => "when",
            Forall(_) => "forall",
            Exists(_) => "exists",
            Select(_) => "select",
            Aggregation(_) => "aggregation",
            UnaryOp(_) => "unary",
            BinOp(_) => "binary",
            Function(_) => "function",
            Identifier(_) => "identifier",
            Number(_) => "number",
            Boolean(_) => "boolean",
            Undefined => "undefined",
            Set(_) => "set",
        }
        .blue();
        let id = format!("{}", node.id).purple();

        let typ = match self.analysis.types.get(&node.id) {
            Some(t) if *t == TypeKind::Undefined => format!("undefined").yellow(),
            Some(t) => format!("{:?}", t).to_lowercase().bright_green(),
            None => format!("unknown").yellow(),
        }
        .bold();

        let target = match &node.variant {
            Identifier(ident) => {
                let reference = &ident.target;
                let rid = reference.rid;
                match self.analysis.references.get(&rid) {
                    Some(target) => match self.analysis.paths.get(target) {
                        Some(path) => format!(" -> {}", format!("{}", path).bright_green()),
                        None => format!(" -> {}", reference.value.bright_green()),
                    },

                    None if is_builtin(&reference.value) => {
                        format!(" -> {}", reference.value.cyan()).to_string()
                    }

                    None => format!(" -> {}", reference.value).to_string(),
                }
            }
            _ => "".to_string(),
        };

        self.push_line(format!("{} {} {} {}", kind, typ, id, target));

        let errors: Vec<_> = self
            .analysis
            .diagnostics
            .iter()
            .filter(|e| e.id == node.id)
            .collect();
        if errors.len() > 0 {
            let title = "*** errors ***".red().bold();
            self.push_line(title.to_string());
            for err in errors {
                self.push_line(format!("- {}", err).red().bold().to_string());
            }
        }

        ControlFlow::Continue(())
    }
}

impl Render<syntax::entity::Entity> for Renderer<'_> {
    fn render(mut self, node: &syntax::entity::Entity) -> String {
        let _ = self.walk(node);
        self.result
    }
}

impl<'a> Renderer<'a> {
    pub fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            indentation: 0,
            result: String::new(),
        }
    }

    pub fn push_line(&mut self, text: String) {
        let indentation = "    ".repeat(self.indentation);
        self.result.push_str(&format!("{}{}\n", indentation, text));
    }
}
