use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Range},
};

use error::Error;
use serde_derive::{Deserialize, Serialize};

pub mod entity;
pub mod error;
pub mod expression;
pub mod markup;

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start.index..self.end.index
    }

    #[inline]
    pub fn merge(mut self, other: Self) -> Self {
        self.end = other.end;
        self
    }
}

impl From<(pest::error::LineColLocation, pest::error::InputLocation)> for Span {
    fn from(value: (pest::error::LineColLocation, pest::error::InputLocation)) -> Self {
        use pest::error::InputLocation;
        use pest::error::LineColLocation;

        let (line_col, location) = value;
        match (line_col, location) {
            (LineColLocation::Pos(p), InputLocation::Pos(index)) => Self {
                start: (p, index).into(),
                end: (p, index).into(),
            },

            (LineColLocation::Span(s, e), InputLocation::Span((sidx, eidx))) => Self {
                start: (s, sidx).into(),
                end: (e, eidx).into(),
            },

            (_, _) => unreachable!(),
        }
    }
}

impl Add<Position> for Span {
    type Output = Self;

    fn add(self, rhs: Position) -> Self::Output {
        Self {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span<'_>) -> Self {
        Self {
            start: (value.start_pos().line_col(), value.start()).into(),
            end: (value.end_pos().line_col(), value.end()).into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl Position {
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
        }
    }

    pub fn new(line: usize, column: usize, index: usize) -> Self {
        Self {
            line,
            column,
            index,
        }
    }

    pub fn next(pos: Position) -> Self {
        Self {
            line: pos.line,
            column: pos.column + 1,
            index: pos.index + 1,
        }
    }
}

impl From<((usize, usize), usize)> for Position {
    fn from(value: ((usize, usize), usize)) -> Self {
        let ((line, column), index) = value;
        Self {
            line,
            column,
            index,
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line + rhs.line - 1,
            column: self.column + rhs.column - 1,
            index: self.index + rhs.index,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub span: Span,
}

impl Context {
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    pub fn parse<R: pest::RuleType>(pair: &pest::iterators::Pair<R>, offset: Position) -> Self {
        let span = Span::from(pair.as_span()) + offset;
        Context { span }
    }
}

pub type ContextMap = HashMap<NodeId, Context>;

#[derive(Default, Debug)]
pub struct NodeParser<'a> {
    pub context: ContextMap,
    pub errors: Vec<Error>,
    last_comment: Option<markup::Markup>,
    last_tags: Vec<(&'a str, Option<&'a str>)>,
    state: entity::parser::State,
    start: Position,
    offset: Position,
}

static mut NODE_ID_GENERATOR: u32 = 0;
static mut REF_ID_GENERATOR: u32 = 0;

// This id is not deterministic but guarantees that each parsed is uniquely identified.
// Two parsing of the same spec in the same process will have different ids
#[derive(
    Serialize, Deserialize, PartialOrd, Ord, Default, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct NodeId(u32);

impl From<u32> for NodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let NodeId(raw) = self;
        write!(f, "#{}", raw)
    }
}

impl NodeId {
    pub fn new() -> Self {
        unsafe {
            let id = NodeId(NODE_ID_GENERATOR);
            NODE_ID_GENERATOR += 1;
            id
        }
    }

    pub fn raw(&self) -> u32 {
        let Self(raw) = self;
        *raw
    }
}

// This id is not deterministic but guarantees that each parsed is uniquely identified.
// Two parsing of the same spec in the same process will have different ids
#[derive(
    Serialize, Deserialize, PartialOrd, Ord, Default, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ReferenceId(u32);

impl From<u32> for ReferenceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl ReferenceId {
    pub fn new() -> Self {
        unsafe {
            let id = ReferenceId(REF_ID_GENERATOR);
            REF_ID_GENERATOR += 1;
            id
        }
    }

    pub fn raw(&self) -> u32 {
        let Self(raw) = self;
        *raw
    }
}
