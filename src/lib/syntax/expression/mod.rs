use super::{Context, Error, NodeId, NodeParser, ReferenceId, entity};
use crate::{
    children::{ChildrenIter, ChildrenIterMut},
    iter::NodeIter,
};
use parser::Rule;
use serde_derive::{Deserialize, Serialize};

pub mod parser;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Expression {
    pub id: NodeId,
    #[serde(flatten)]
    pub variant: ExpressionVariant,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression {
            id: NodeId::default(),
            variant: ExpressionVariant::Undefined,
        }
    }
}

impl ChildrenIter for Expression {
    fn children_iter(&self) -> impl Iterator<Item = &Expression> {
        use ExpressionVariant::*;

        let mut children = Vec::new();
        match &self.variant {
            Branch(branch) => {
                let bo = &branch.otherwise;
                children.push(branch.case.condition.as_ref());
                children.push(branch.case.consequence.as_ref());
                children.extend(bo.iter().map(|c| c.as_ref()));
            }

            When(when) => {
                let cases = when
                    .cases
                    .iter()
                    .map(|c| [c.condition.as_ref(), c.consequence.as_ref()])
                    .flatten();

                children.extend(cases);
                children.extend(when.otherwise.iter().map(|c| c.as_ref()));
            }

            Forall(forall) => {
                let fs = &forall.set;
                children.push(&*fs.domain);
                children.extend(fs.filter.iter().map(|n| n.as_ref()));
                children.push(forall.expression.as_ref());
            }

            Exists(exists) => {
                let es = &exists.set;
                children.push(es.domain.as_ref());
                children.extend(es.filter.iter().map(|n| n.as_ref()));
            }

            Select(select) => {
                let ss = &select.set;
                let so = &select.optimizer;
                children.push(ss.domain.as_ref());
                children.extend(ss.filter.iter().map(|n| n.as_ref()));
                children.extend(so.iter().map(|n| n.expression.as_ref()));
            }

            Aggregation(aggregation) => {
                let ae = &aggregation.expressions;
                children.extend(ae.iter());
            }

            UnaryOp(unary) => {
                children.push(unary.operand.as_ref());
            }

            BinOp(binary) => {
                children.push(binary.left.as_ref());
                children.push(binary.right.as_ref());
            }

            Function(function) => {
                let fa = &function.arguments;
                children.push(function.expression.as_ref());
                children.extend(fa.iter());
            }

            Set(set) => {
                let se = &set.elements;
                children.extend(se.iter());
            }

            Identifier(_) => {}
            Number(_) => {}
            Boolean(_) => {}
            Undefined => {}
        };

        children.into_iter()
    }
}

impl ChildrenIterMut for Expression {
    fn children_iter_mut(&mut self) -> impl Iterator<Item = &mut Expression> {
        use ExpressionVariant::*;

        let mut children = Vec::new();
        match &mut self.variant {
            Branch(branch) => {
                let bo = &mut branch.otherwise;
                children.push(branch.case.condition.as_mut());
                children.push(branch.case.consequence.as_mut());
                children.extend(bo.iter_mut().map(|c| c.as_mut()));
            }
            When(when) => {
                let cases = when
                    .cases
                    .iter_mut()
                    .map(|c| [c.condition.as_mut(), c.consequence.as_mut()])
                    .flatten();
                children.extend(cases);
                children.extend(when.otherwise.iter_mut().map(|c| c.as_mut()));
            }
            Forall(forall) => {
                let fs = &mut forall.set;
                children.push(fs.domain.as_mut());
                children.extend(fs.filter.iter_mut().map(|n| n.as_mut()));
                children.push(forall.expression.as_mut());
            }
            Exists(exists) => {
                let es = &mut exists.set;
                children.push(es.domain.as_mut());
                children.extend(es.filter.iter_mut().map(|n| n.as_mut()));
            }
            Select(select) => {
                let ss = &mut select.set;
                let so = &mut select.optimizer;
                children.push(ss.domain.as_mut());
                children.extend(ss.filter.iter_mut().map(|n| n.as_mut()));
                children.extend(so.iter_mut().map(|n| n.expression.as_mut()));
            }
            Aggregation(aggregation) => {
                let ae = &mut aggregation.expressions;
                children.extend(ae.iter_mut());
            }
            UnaryOp(unary) => {
                children.push(unary.operand.as_mut());
            }
            BinOp(binary) => {
                children.push(binary.left.as_mut());
                children.push(binary.right.as_mut());
            }
            Function(function) => {
                let fa = &mut function.arguments;
                children.push(function.expression.as_mut());
                children.extend(fa.iter_mut());
            }
            Set(set) => {
                let se = &mut set.elements;
                children.extend(se.iter_mut());
            }

            Identifier(_) => {}
            Number(_) => {}
            Boolean(_) => {}
            Undefined => {}
        };

        children.into_iter()
    }
}

impl Expression {
    pub fn new(id: NodeId, variant: ExpressionVariant) -> Self {
        Expression { id, variant }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Self> {
        NodeIter::new(self)
    }

    pub fn needs_parentheses(&self, parent: &Expression, is_right: bool) -> bool {
        let parent_prec = parent.variant.precedence();
        let child_prec = self.variant.precedence();

        if child_prec < parent_prec {
            return true;
        }

        if child_prec == parent_prec {
            if let (Some(parent_assoc), Some(_)) =
                (parent.variant.associativity(), self.variant.associativity())
            {
                match parent_assoc {
                    Associativity::Right => !is_right,
                    Associativity::Left => is_right,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn set_element(&self) -> Option<&SetElement> {
        use ExpressionVariant::*;

        match &self.variant {
            Forall(forall) => Some(&forall.set),
            Exists(exists) => Some(&exists.set),
            Select(select) => Some(&select.set),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ExpressionVariant {
    Branch(Branch),
    When(When),
    Forall(Forall),
    Exists(Exists),
    Select(Select),
    Aggregation(Aggregation),

    UnaryOp(UnaryOp),
    BinOp(BinOp),
    Function(Function),

    Identifier(Identifier),
    Number(Number),
    Boolean(Boolean),
    Undefined,

    Set(Set),
}

impl ExpressionVariant {
    pub fn precedence(&self) -> i32 {
        match self {
            ExpressionVariant::BinOp(bin_op) => bin_op.operator.precedence(),
            ExpressionVariant::UnaryOp(unary_op) => unary_op.operator.precedence(),
            ExpressionVariant::Function(_) => 160,
            _ => 1000, // Highest precedence for literals, identifiers, etc.
        }
    }

    pub fn associativity(&self) -> Option<Associativity> {
        match self {
            ExpressionVariant::BinOp(bin_op) => Some(bin_op.operator.associativity()),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Branch {
    pub case: Case,
    pub otherwise: Option<Box<Expression>>,
}

impl Branch {
    pub fn new(case: Case) -> Self {
        Branch {
            case,
            otherwise: None,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct When {
    pub cases: Vec<Case>,
    pub otherwise: Option<Box<Expression>>,
}

impl When {
    pub fn new(cases: Vec<Case>) -> Self {
        When {
            cases,
            otherwise: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Case {
    pub condition: Box<Expression>,
    pub consequence: Box<Expression>,
}

impl Case {
    pub fn new(condition: Expression, consequence: Expression) -> Self {
        Case {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Forall {
    pub set: SetElement,
    pub expression: Box<Expression>,
}

impl Forall {
    pub fn new(set: SetElement, expression: Expression) -> Self {
        Forall {
            set,
            expression: Box::new(expression),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Exists {
    pub set: SetElement,
}

impl Exists {
    pub fn new(set: SetElement) -> Self {
        Exists { set }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Select {
    pub set: SetElement,
    pub optimizer: Option<Optimizer>,
}

impl Select {
    pub fn new(set: SetElement) -> Self {
        Select {
            set,
            optimizer: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Optimizer {
    pub kind: OptimizerKind,
    pub expression: Box<Expression>,
}

impl Optimizer {
    pub fn new(kind: OptimizerKind, expression: Expression) -> Self {
        Optimizer {
            kind,
            expression: Box::new(expression),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum OptimizerKind {
    Minimize,
    Maximize,
}

impl From<Rule> for OptimizerKind {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::mini => OptimizerKind::Minimize,
            Rule::maxi => OptimizerKind::Maximize,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SetElement {
    pub variable: String,
    pub domain: Box<Expression>,
    pub filter: Option<Box<Expression>>,
}

impl SetElement {
    pub fn new(variable: String, domain: Expression) -> Self {
        SetElement {
            variable,
            domain: Box::new(domain),
            filter: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Aggregation {
    pub aggregator: Aggregator,
    pub expressions: Vec<Expression>,
}

impl Aggregation {
    pub fn new(aggregator: Aggregator) -> Self {
        Aggregation {
            aggregator,
            expressions: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Aggregator {
    All,
    Any,
}

impl From<Rule> for Aggregator {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::all => Aggregator::All,
            Rule::any => Aggregator::Any,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Identifier {
    pub target: entity::Reference,
}

impl Identifier {
    pub fn new(target: entity::Reference) -> Self {
        Self { target }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

impl UnaryOp {
    pub fn new(operator: UnaryOperator, operand: Expression) -> Self {
        UnaryOp {
            operator,
            operand: Box::new(operand),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum UnaryOperator {
    Plus,
    Negation,
    Not,
    Factorial,
    Previously,
    Rise,
    Fall,
    Eventually,
    Always,
}

impl From<Rule> for UnaryOperator {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::plus => UnaryOperator::Plus,
            Rule::neg => UnaryOperator::Negation,
            Rule::not => UnaryOperator::Not,
            Rule::fac => UnaryOperator::Factorial,
            Rule::previous => UnaryOperator::Previously,
            Rule::rise => UnaryOperator::Rise,
            Rule::fall => UnaryOperator::Fall,
            Rule::event => UnaryOperator::Eventually,
            Rule::always => UnaryOperator::Always,
            _ => unreachable!(),
        }
    }
}

impl UnaryOperator {
    pub fn precedence(&self) -> i32 {
        match self {
            UnaryOperator::Not => 40,
            UnaryOperator::Previously
            | UnaryOperator::Rise
            | UnaryOperator::Fall
            | UnaryOperator::Eventually
            | UnaryOperator::Always => 50,
            UnaryOperator::Negation | UnaryOperator::Plus => 140,
            UnaryOperator::Factorial => 150,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct BinOp {
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl BinOp {
    pub fn new(operator: BinaryOperator, left: Expression, right: Expression) -> Self {
        BinOp {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Power,

    And,
    Or,
    Xor,
    In,
    Implies,
    Iff,

    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
    Equal,

    Union,
    Intersection,
    Difference,
    Complement,
    Includes,

    Since,
}

impl From<Rule> for BinaryOperator {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::add => BinaryOperator::Plus,
            Rule::sub => BinaryOperator::Minus,
            Rule::mul => BinaryOperator::Multiply,
            Rule::div => BinaryOperator::Divide,
            Rule::rem => BinaryOperator::Modulus,
            Rule::pow => BinaryOperator::Power,

            Rule::and => BinaryOperator::And,
            Rule::or => BinaryOperator::Or,
            Rule::xor => BinaryOperator::Xor,
            Rule::ins => BinaryOperator::In,
            Rule::implies => BinaryOperator::Implies,
            Rule::iff => BinaryOperator::Iff,

            Rule::gt => BinaryOperator::GreaterThan,
            Rule::lt => BinaryOperator::LessThan,
            Rule::ge => BinaryOperator::GreaterOrEqual,
            Rule::le => BinaryOperator::LessOrEqual,
            Rule::neq => BinaryOperator::NotEqual,
            Rule::eq => BinaryOperator::Equal,

            Rule::uni => BinaryOperator::Union,
            Rule::inter => BinaryOperator::Intersection,
            Rule::diff => BinaryOperator::Difference,
            Rule::comp => BinaryOperator::Complement,
            Rule::incl => BinaryOperator::Includes,

            Rule::since => BinaryOperator::Since,
            _ => unreachable!(),
        }
    }
}

impl BinaryOperator {
    pub fn precedence(&self) -> i32 {
        match self {
            BinaryOperator::Iff | BinaryOperator::Implies => 10,
            BinaryOperator::Or | BinaryOperator::Xor => 20,
            BinaryOperator::And => 30,
            BinaryOperator::Since => 60,
            BinaryOperator::Union | BinaryOperator::Difference => 70,
            BinaryOperator::Intersection => 71,
            BinaryOperator::Complement => 72,
            BinaryOperator::In | BinaryOperator::Includes => 80,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 90,
            BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessOrEqual
            | BinaryOperator::GreaterOrEqual => 100,
            BinaryOperator::Plus | BinaryOperator::Minus => 110,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulus => 120,
            BinaryOperator::Power => 130,
        }
    }

    pub fn associativity(&self) -> Associativity {
        match self {
            BinaryOperator::Power => Associativity::Right,
            _ => Associativity::Left,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Function {
    pub expression: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl Function {
    pub fn new(expression: Expression) -> Self {
        Function {
            expression: Box::new(expression),
            arguments: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Set {
    pub elements: Vec<Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Number {
    pub kind: NumberKind,
    pub value: String,
}

impl Number {
    pub fn new(kind: NumberKind) -> Self {
        Number {
            kind,
            value: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum NumberKind {
    Integer,
    Real,
    Infinity,
}

impl From<Rule> for NumberKind {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::int => NumberKind::Integer,
            Rule::real => NumberKind::Real,
            Rule::infinity => NumberKind::Infinity,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Boolean {
    pub value: bool,
}

impl From<Rule> for Boolean {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::tru => Self { value: true },
            Rule::fals => Self { value: false },
            _ => unreachable!(),
        }
    }
}
