use std::{collections::HashMap, fmt::Display, ops::ControlFlow};

use serde_derive::Serialize;

use crate::{
    children::ChildrenIter,
    diagnostic::{Diagnostic, DiagnosticKind},
    mapper::id::NodeMap,
    syntax::{
        self, NodeId,
        entity::{Entity, EntityVariant},
        expression::{Case, Expression, ExpressionVariant, SetElement},
    },
    verifier::reference::ReferenceMap,
    visitor::{Visitor, Walk, WalkCustom},
};

pub struct Verifier<'a> {
    nodes_by_id: &'a NodeMap<'a>,
    references: &'a ReferenceMap,
    pub results: HashMap<NodeId, Result<TypeKind, Diagnostic>>,
}

impl<'a> Walk<&Entity> for Verifier<'a> {
    fn walk(&mut self, node: &Entity) -> ControlFlow<()> {
        use EntityVariant::*;

        for child in node.children_iter() {
            match child.variant {
                Attribute(_) => continue,
                _ => self.walk(child)?,
            }
        }

        self.visit(node)
    }
}

impl<'a> Visitor<&Entity> for Verifier<'a> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &Entity) -> ControlFlow<()> {
        for expression in node.expressions() {
            self.walk(expression)?;
        }
        ControlFlow::Continue(())
    }
}

impl<'a> Walk<&Expression> for Verifier<'a> {
    fn walk(&mut self, node: &Expression) -> ControlFlow<()> {
        if let Some(set) = node.set_element() {
            self.walk(set.domain.as_ref())?;
        }

        for child in node.children_iter() {
            self.walk(child)?;
        }

        self.visit(node)?;
        ControlFlow::Continue(())
    }
}

impl<'a> Visitor<&Expression> for Verifier<'a> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &Expression) -> ControlFlow<()> {
        let result = self.eval(node);
        self.results.insert(node.id, result);
        ControlFlow::Continue(())
    }
}

impl Verifier<'_> {
    #[inline]
    fn get_result(&self, referer: &Expression, node: &Expression) -> Result<TypeKind, Diagnostic> {
        self.results
            .get(&node.id)
            .unwrap()
            .clone()
            .map_err(|err| Diagnostic::propagated(referer.id, err))
    }

    fn check_set_element(
        &self,
        node: &Expression,
        set: &SetElement,
    ) -> Result<TypeKind, Diagnostic> {
        use DiagnosticKind::NotASet;
        use DiagnosticKind::UnexpectedType;
        use TypeKind::*;

        let domain = &set.domain;
        let typ = match self.get_result(node, domain)? {
            Set(t) => t,
            typ => return Err(Diagnostic::new(domain.id, NotASet(typ))),
        };

        if let Some(filter) = &set.filter {
            let typ = self.get_result(node, filter)?;
            if typ != Boolean {
                let id = filter.id;
                return Err(Diagnostic::new(id, UnexpectedType(Boolean, typ)));
            }
        }

        Ok(*typ.clone())
    }

    fn check_case(
        &self,
        node: &Expression,
        expected: &Option<TypeKind>,
        case: &Case,
    ) -> Result<TypeKind, Diagnostic> {
        use DiagnosticKind::UnexpectedType;
        use TypeKind::*;

        let cond = self.get_result(node, &case.condition)?;
        if cond != Boolean {
            let id = case.condition.id;
            return Err(Diagnostic::new(id, UnexpectedType(Boolean, cond)));
        }

        let cons = self.get_result(node, &case.consequence)?;
        match expected {
            Some(t) if cons == *t => Ok(cons),
            Some(t) => {
                let id = case.consequence.id;
                Err(Diagnostic::new(id, UnexpectedType(t.clone(), cons)))
            }
            None => Ok(cons),
        }
    }

    fn eval(&mut self, node: &Expression) -> Result<TypeKind, Diagnostic> {
        {
            use DiagnosticKind::NotASet;
            use DiagnosticKind::NotAnAttribute;
            use DiagnosticKind::UnexpectedType;
            use EntityVariant::*;
            use ExpressionVariant::*;

            match &node.variant {
                Branch(branch) => {
                    let els = match &branch.otherwise {
                        Some(expr) => Some(self.get_result(node, &expr)?),
                        None => None,
                    };

                    self.check_case(node, &els, &branch.case)
                }

                When(when) => {
                    let mut cases = when.cases.iter();
                    let expected = match &when.otherwise {
                        Some(expr) => Some(self.get_result(node, &expr)?),
                        None => Some(self.get_result(node, &cases.next().unwrap().consequence)?),
                    };

                    for c in cases {
                        self.check_case(node, &expected, c)?;
                    }

                    Ok(expected.unwrap())
                }

                Forall(forall) => {
                    use TypeKind::*;

                    self.check_set_element(node, &forall.set)?;

                    let expr = forall.expression.as_ref();
                    let typ = self.get_result(node, expr)?;
                    if typ != Boolean {
                        let id = expr.id;
                        return Err(Diagnostic::new(id, UnexpectedType(Boolean, typ)));
                    }

                    Ok(Boolean)
                }

                Exists(exists) => {
                    use TypeKind::*;

                    self.check_set_element(node, &exists.set)?;
                    Ok(Boolean)
                }

                Select(select) => {
                    use TypeKind::*;

                    let typ = self.check_set_element(node, &select.set)?;
                    match &select.optimizer {
                        None => Ok(typ),
                        Some(opt) => match self.get_result(node, &opt.expression)? {
                            Number => Ok(Number),
                            t => {
                                let id = opt.expression.id;
                                Err(Diagnostic::new(id, UnexpectedType(Number, t)))
                            }
                        },
                    }
                }

                Aggregation(aggregation) => {
                    use TypeKind::*;

                    for expression in &aggregation.expressions {
                        let type_expr = self.get_result(node, expression)?;
                        if type_expr != Boolean {
                            let id = expression.id;
                            return Err(Diagnostic::new(id, UnexpectedType(Boolean, type_expr)));
                        }
                    }

                    Ok(Boolean)
                }

                UnaryOp(unary_op) => {
                    use TypeKind::*;
                    use syntax::expression::UnaryOperator::*;

                    let type_op = self.get_result(node, &unary_op.operand)?;

                    let expected = match unary_op.operator {
                        Plus | Negation | Factorial => Number,
                        Not | Previously | Rise | Fall | Eventually | Always => Boolean,
                    };

                    match type_op {
                        actual if actual != expected => {
                            let id = unary_op.operand.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, actual)))
                        }
                        _ => Ok(expected),
                    }
                }

                BinOp(bin_op) => {
                    use TypeKind::*;
                    use syntax::expression::BinaryOperator::*;

                    let lhs = self.get_result(node, &bin_op.left)?;
                    let rhs = self.get_result(node, &bin_op.right)?;

                    let (expected, result) = match &bin_op.operator {
                        Plus | Minus | Multiply | Divide | Modulus | Power => (Number, Number),
                        And | Or | Xor | Implies | Iff | Since => (Boolean, Boolean),
                        GreaterThan | LessThan | GreaterOrEqual | LessOrEqual => (Number, Boolean),
                        NotEqual | Equal => (lhs.clone(), Boolean),
                        Union | Intersection | Difference | Complement => {
                            (lhs.clone(), lhs.clone())
                        }
                        In => match &rhs {
                            TypeKind::Set(rhs) => (*rhs.clone(), Boolean),
                            _ => return Err(Diagnostic::new(bin_op.right.id, NotASet(rhs))),
                        },
                        Includes => match &rhs {
                            TypeKind::Set(rhs) => (TypeKind::Set(rhs.clone()), Boolean),
                            _ => return Err(Diagnostic::new(bin_op.right.id, NotASet(rhs))),
                        },
                    };

                    match (&bin_op.operator, lhs, rhs) {
                        (In, lhs, _) if lhs == expected => Ok(result),
                        (In, lhs, _) => {
                            let id = bin_op.left.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, lhs)))
                        }
                        (Union | Intersection | Includes | Complement | Difference, lhs, _)
                            if !lhs.is_set() =>
                        {
                            let id = bin_op.left.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, lhs)))
                        }
                        (Union | Intersection | Includes | Complement | Difference, _, rhs)
                            if !rhs.is_set() =>
                        {
                            let id = bin_op.right.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, rhs)))
                        }
                        (_, lhs, _) if lhs != expected => {
                            let id = bin_op.left.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, lhs)))
                        }
                        (_, _, rhs) if rhs != expected => {
                            let id = bin_op.right.id;
                            Err(Diagnostic::new(id, UnexpectedType(expected, rhs)))
                        }
                        (_, _, _) => Ok(result),
                    }
                }

                Set(set) => {
                    let mut se = set.elements.iter();
                    let type_first = self.get_result(node, &se.next().unwrap())?;

                    for elem in se {
                        let type_elem = self.get_result(node, &elem)?;
                        if type_elem != type_first {
                            let id = elem.id;
                            return Err(Diagnostic::new(id, UnexpectedType(type_first, type_elem)));
                        }
                    }

                    Ok(TypeKind::Set(Box::new(type_first)))
                }

                Function(_) => todo!(),

                Identifier(ident) if is_builtin_number(&ident.target.value) => {
                    Ok(TypeKind::Set(Box::new(TypeKind::Number)))
                }

                Identifier(ident) if is_builtin_boolean(&ident.target.value) => {
                    Ok(TypeKind::Set(Box::new(TypeKind::Boolean)))
                }

                Identifier(ident) => match self.references.get(&ident.target.rid) {
                    Some(id) => match &self.nodes_by_id.entities.get(id) {
                        Some(entity) => match &entity.variant {
                            Attribute(attribute) => match self.results.get(&attribute.domain.id) {
                                Some(Ok(TypeKind::Set(typ))) => Ok(*typ.clone()),
                                _ => Ok(TypeKind::Undefined),
                            },

                            _ => Err(Diagnostic::new(*id, NotAnAttribute(ident.target.clone()))),
                        },

                        None => match self.nodes_by_id.expressions.get(id) {
                            Some(expr) => match self.results.get(&expr.id) {
                                Some(Ok(TypeKind::Set(typ))) => Ok(*typ.clone()),
                                _ => Ok(TypeKind::Undefined),
                            },
                            None => Ok(TypeKind::Undefined),
                        },
                    },

                    None => Ok(TypeKind::Undefined),
                },

                Number(_) => Ok(TypeKind::Number),
                Boolean(_) => Ok(TypeKind::Boolean),
                Undefined => Ok(TypeKind::Undefined), // Undefined behavior should be studied
            }
        }
    }
}

pub type TypeMap = HashMap<NodeId, TypeKind>;

impl<'a> Verifier<'a> {
    pub fn new(references: &'a ReferenceMap, nodes_by_id: &'a NodeMap) -> Self {
        Self {
            references,
            nodes_by_id,
            results: HashMap::new(),
        }
    }

    pub fn prewalk(&mut self, node: &Entity) {
        use EntityVariant::*;
        let attributes: Vec<_> = node
            .iter()
            .filter(|n| match &n.variant {
                Attribute(_) => true,
                _ => false,
            })
            .collect();

        for attr in attributes {
            let _ = self.visit(attr);
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum TypeKind {
    Number,
    Boolean,
    Undefined,
    Set(Box<TypeKind>),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TypeKind {
    pub fn is_set(&self) -> bool {
        match self {
            TypeKind::Set(_) => true,
            _ => false,
        }
    }
}

#[inline]
pub fn is_builtin_number(val: &str) -> bool {
    match val {
        "real" | "integer" => true,
        _ => false,
    }
}

#[inline]
pub fn is_builtin_boolean(val: &str) -> bool {
    match val {
        "boolean" => true,
        _ => false,
    }
}
