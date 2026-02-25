use std::fmt::Display;

use pest;

use super::{entity::tokenizer::Token, Span};

#[derive(Debug, Clone)]
pub struct Error {
    pub span: Span,
    pub source: String,
    pub kind: ErrorKind,
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for Error {
    fn from(error: pest::error::Error<R>) -> Self {
        let source = error.line().to_string();
        let span: Span = (error.line_col, error.location).into();
        let kind = match error.variant {
            pest::error::ErrorVariant::ParsingError { .. } => ErrorKind::UnexpectedToken,
            pest::error::ErrorVariant::CustomError { message } => ErrorKind::Custom(message),
        };

        Self { span, source, kind }
    }
}

impl Error {
    pub fn new(span: Span, source: String, kind: ErrorKind) -> Self {
        Self { span, source, kind }
    }

    pub fn from_token(token: &Token, kind: ErrorKind) -> Self {
        Self {
            span: token.span,
            source: token.as_str().to_string(),
            kind,
        }
    }

    pub fn empty() -> Self {
        Self {
            span: Span::default(),
            source: String::new(),
            kind: ErrorKind::Empty,
        }
    }

    pub fn unterminated(token: &Token<'_>) -> Self {
        Self {
            span: token.span,
            source: token.as_str().to_string(),
            kind: ErrorKind::Unterminated,
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;

        match &self {
            Custom(message) => write!(f, "{}", message),
            MissingIdentifier => write!(f, "Missing identifier"),
            Unterminated => write!(f, "Unterminated entity"),
            UnexpectedToken => write!(f, "Unexpected token"),
            UnexpectedKeyword => write!(f, "Unexpected keyword"),
            UnexpectedExpression => write!(f, "Unexpected expression"),
            BadIdentifier => write!(f, "Bad identifier"),
            MissingExpression => write!(f, "Missing expression"),
            IllFormedRequirement => write!(f, "Requirement has no expression neither traceability"),
            IllFormedAttribute => write!(f, "Attribute has no domain"),
            Empty => write!(f, "Empty input"),
            MissingEnd => write!(f, "Entity end is not present"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} at \"{}\"",
            self.span.start, self.kind, self.source
        )
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Empty,
    Custom(String),
    MissingIdentifier,
    Unterminated,
    UnexpectedToken,
    UnexpectedKeyword,
    UnexpectedExpression,
    BadIdentifier,
    MissingExpression,
    IllFormedRequirement,
    IllFormedAttribute,
    MissingEnd,
}
