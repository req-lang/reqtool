use std::ops::ControlFlow;

use crate::{
    children::Children,
    syntax::{
        entity::{self, RequirementVariant},
        expression,
    },
    visitor::{Visitor, Walk, WalkCustom},
};

use unindent::unindent;

use super::Render;

pub struct Renderer {
    pub source: String,
    indentation: usize,
}

impl Walk<&entity::Entity> for Renderer {
    fn walk(&mut self, node: &entity::Entity) -> ControlFlow<()> {
        self.visit(node)?;

        self.indentation += 1;
        if let Some(imports) = node.imports() {
            for import in imports {
                self.push_line(&format!("import {}", import.value));
            }
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk(child)?;
            }
        }
        self.indentation -= 1;
        self.push_entity_end(node);
        ControlFlow::Continue(())
    }
}

impl Visitor<&entity::Entity> for Renderer {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &entity::Entity) -> ControlFlow<()> {
        use entity::EntityVariant::*;

        if let Some(comment) = &node.meta.comment {
            self.push_line("@@");

            let unindented = unindent(&comment.clone().to_string());
            for line in unindented.lines() {
                self.push_line(line);
            }

            self.push_line("@@");
        }

        for tag in &node.meta.tags {
            match &tag.value {
                Some(v) => self.push_line(&format!("#{} {}#", tag.id, v)),
                None => self.push_line(&format!("# {} #", tag.id)),
            }
        }

        match &node.variant {
            Package(_) => {
                self.push_line(&format!("package {}", node.meta.label));
            }

            Part(_) => {
                self.push_line(&format!("part {}", node.meta.label));
            }

            Attribute(attribute) => {
                self.push_indented(&format!("let {} in ", node.meta.label));

                self.walk(&attribute.domain)?;
                if let Some(unit) = &attribute.unit {
                    // Indeed, the unit shall never be multi-line, might be fixed when proper unit
                    // syntax is introduced.
                    self.source.push_str(" [");
                    let mut renderer = Renderer::new();
                    renderer.walk(unit)?;
                    let result = renderer.source;
                    for line in result.lines() {
                        self.source.push_str(&line);
                    }
                    self.source.push_str("]");
                }

                self.source.push_str("\n");
            }

            Requirement(requirement) => {
                self.push_line(&format!("requirement {}", node.meta.label));

                for trace in &requirement.traceability {
                    use entity::TraceabilityKind::*;

                    let kind = match trace.kind {
                        Derivation => "derives",
                        Specialization => "specializes",
                        Refinement => "refines",
                    };

                    self.push_line(&format!("{} {}", kind, trace.target));
                }

                self.push_line("is");

                self.indentation += 1;
                match &requirement.variant {
                    RequirementVariant::Formal(expr) => {
                        let mut renderer = Renderer::new();
                        renderer.walk(expr)?;
                        let result = renderer.source;
                        for line in result.lines() {
                            self.push_line(&line);
                        }
                    }
                    RequirementVariant::Informal(text) => {
                        self.push_line("@@");

                        let unindented = unindent(&text.clone().to_string());
                        for line in unindented.lines() {
                            self.push_line(line);
                        }

                        self.push_line("@@");
                    }
                }
                self.indentation -= 1;
            }
        }

        ControlFlow::Continue(())
    }
}

impl Walk<&expression::Expression> for Renderer {
    fn walk(&mut self, node: &expression::Expression) -> ControlFlow<()> {
        use expression::ExpressionVariant::*;

        match &node.variant {
            Branch(branch) => {
                self.push_indented("if ");
                self.walk(branch.case.condition.as_ref())?;
                self.source.push_str(" then\n");
                self.indentation += 1;
                self.walk(branch.case.consequence.as_ref())?;
                self.indentation -= 1;

                if let Some(otherwise) = &branch.otherwise {
                    self.push_line("else");
                    self.indentation += 1;
                    self.walk(otherwise.as_ref())?;
                    self.indentation -= 1;
                }

                self.push_end();
            }

            When(when) => {
                self.push_line("when");
                self.indentation += 1;
                for (idx, case) in when.cases.iter().enumerate() {
                    if idx > 0 {
                        self.source.push_str(", ");
                    }
                    self.walk(case.condition.as_ref())?;
                    self.source.push_str(" then\n");
                    self.indentation += 1;
                    self.walk(case.consequence.as_ref())?;
                    self.indentation -= 1;
                }
                if let Some(otherwise) = &when.otherwise {
                    self.source.push_str(",\n");
                    self.push_line("otherwise");
                    self.indentation += 1;
                    self.walk(otherwise.as_ref())?;
                    self.indentation -= 1;
                }
                self.push_end();
            }

            Forall(forall) => {
                self.push_indented("forall ");
                self.source.push_str(&forall.set.variable);
                self.source.push_str(" in ");
                self.walk(forall.set.domain.as_ref())?;
                self.source.push_str("\n");

                if let Some(ref filter) = forall.set.filter {
                    self.push_indented("such that ");
                    self.walk(filter.as_ref())?;
                    self.source.push_str("\n");
                }

                self.walk(forall.expression.as_ref())?;
                self.push_end();
            }

            Exists(exists) => {
                self.push_indented("exists ");
                self.source.push_str(&exists.set.variable);
                self.source.push_str(" in ");
                self.walk(exists.set.domain.as_ref())?;

                if let Some(ref filter) = exists.set.filter {
                    self.source.push_str("\n");
                    self.push_indented("such that ");
                    self.indentation += 1;
                    self.walk(filter.as_ref())?;
                    self.indentation -= 1;
                }

                self.push_end();
            }

            Select(select) => {
                self.push_indented("select ");
                self.source.push_str(&select.set.variable);
                self.source.push_str(" in ");
                self.walk(select.set.domain.as_ref())?;

                self.indentation += 1;
                if let Some(ref filter) = select.set.filter {
                    self.source.push_str("\n");
                    self.push_indented("such that ");
                    self.walk(filter.as_ref())?;
                }

                if let Some(ref optimizer) = select.optimizer {
                    use expression::OptimizerKind::*;

                    self.source.push_str("\n");
                    let opt = match optimizer.kind {
                        Minimize => "minimize",
                        Maximize => "maximize",
                    };
                    self.push_indented(&format!("{} ", opt));
                    self.walk(optimizer.expression.as_ref())?;
                }
                self.indentation -= 1;

                self.push_end();
            }

            Aggregation(aggregation) => {
                use expression::Aggregator::*;

                let agg = match aggregation.aggregator {
                    Any => "any",
                    All => "all",
                };
                self.push_line(&format!("{} ", agg));
                self.indentation += 1;

                for (idx, val) in aggregation.expressions.iter().enumerate() {
                    if idx > 0 {
                        self.source.push_str(", ");
                    }

                    self.push_line("");
                    self.walk(val)?;
                }

                self.push_end();
            }

            UnaryOp(unary_op) => {
                use expression::UnaryOperator::*;

                let op = match unary_op.operator {
                    Not => "not",
                    Plus => "+",
                    Negation => "-",
                    Factorial => "!",
                    Previously => "previously",
                    Rise => "rising",
                    Fall => "falling",
                    Eventually => "eventually",
                    Always => "always",
                };

                match unary_op.operator {
                    Factorial => {
                        let needs_parens = unary_op.operand.needs_parentheses(node, false);
                        if needs_parens {
                            self.source.push_str("(");
                        }
                        self.walk(unary_op.operand.as_ref())?;
                        if needs_parens {
                            self.source.push_str(")");
                        }
                        self.source.push_str(&format!("{}", op));
                    }
                    _ => {
                        self.push_indented(&format!("{} ", op));
                        let needs_parens = unary_op.operand.needs_parentheses(node, false);
                        if needs_parens {
                            self.source.push_str("(");
                        }
                        self.walk(unary_op.operand.as_ref())?;
                        if needs_parens {
                            self.source.push_str(")");
                        }
                    }
                }
            }

            BinOp(bin_op) => {
                use expression::BinaryOperator::*;

                let op = match bin_op.operator {
                    And => "and",
                    Or => "or",
                    Xor => "xor",
                    Implies => "implies",
                    Iff => "iff",
                    Equal => "=",
                    NotEqual => "<>",
                    LessThan => "<",
                    GreaterThan => ">",
                    Plus => "+",
                    Minus => "-",
                    Multiply => "*",
                    Divide => "/",
                    Modulus => "%",
                    Power => "^",
                    In => "in",
                    GreaterOrEqual => ">=",
                    LessOrEqual => "<=",
                    Union => "union",
                    Intersection => "intersection",
                    Since => "since",
                    Difference => "difference",
                    Complement => "complement",
                    Includes => "includes",
                };

                self.push_indented("");
                let left_needs_parens = bin_op.left.needs_parentheses(node, false);
                let right_needs_parens = bin_op.right.needs_parentheses(node, true);
                if left_needs_parens {
                    self.source.push_str("(");
                }
                self.walk(bin_op.left.as_ref())?;
                if left_needs_parens {
                    self.source.push_str(")");
                }

                self.source.push_str(&format!(" {} ", op));

                if right_needs_parens {
                    self.source.push_str("(");
                }
                self.walk(bin_op.right.as_ref())?;
                if right_needs_parens {
                    self.source.push_str(")");
                }
            }

            Function(function) => {
                self.walk(function.expression.as_ref())?;
                self.source.push_str("(");
                for (i, arg) in function.arguments.iter().enumerate() {
                    if i > 0 {
                        self.source.push_str(", ");
                    }
                    self.walk(arg)?;
                }
                self.source.push_str(")");
            }

            Set(set) => {
                self.source.push_str("{");
                for (i, elem) in set.elements.iter().enumerate() {
                    if i > 0 {
                        self.source.push_str(", ");
                    }
                    self.walk(elem)?;
                }
                self.source.push_str("}");
            }

            Identifier(r) => self.source.push_str(&r.target.value[..]),

            Number(x) => self.source.push_str(&x.value),

            Boolean(b) => match b.value {
                true => self.source.push_str("true"),
                false => self.source.push_str("false"),
            },

            Undefined => self.source.push_str("undefined"),
        }
        ControlFlow::Continue(())
    }
}

impl Render<entity::Entity> for Renderer {
    fn render(mut self, node: &entity::Entity) -> String {
        let _ = self.walk(node);
        self.source
    }
}

impl Render<expression::Expression> for Renderer {
    fn render(mut self, node: &expression::Expression) -> String {
        let _ = self.walk(node);
        self.source
    }
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            source: String::new(),
            indentation: 0,
        }
    }

    pub fn push_line(&mut self, line: &str) {
        let indentation = " ".repeat(self.indentation * 2);
        self.source.push_str(&format!("{}{}\n", indentation, line));
    }

    pub fn push_entity_end(&mut self, node: &entity::Entity) {
        match &node.variant {
            entity::EntityVariant::Part(_) => self.push_line("part"),
            entity::EntityVariant::Attribute(_) => {}
            entity::EntityVariant::Package(_) => self.push_line("package"),
            entity::EntityVariant::Requirement(_) => self.push_line("requirement\n"),
        };
    }

    pub fn push_end(&mut self) {
        let indentation = " ".repeat(self.indentation * 2);
        self.source.push_str(&format!("\n{}end\n\n", indentation));
    }

    pub fn push_indented(&mut self, str: &str) {
        let indentation = " ".repeat(self.indentation * 2);
        self.source.push_str(&format!("{}{}", indentation, str));
    }
}
