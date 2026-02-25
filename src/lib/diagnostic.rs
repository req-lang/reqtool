use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use serde_derive::Serialize;

pub type CheckResult<T> = (T, Vec<Diagnostic>);

use crate::{
    mapper::path::Path,
    syntax::{NodeId, entity::Reference},
    verifier::typing::TypeKind,
};

#[derive(Serialize, PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct DiagnosticId(u32);

impl DiagnosticId {
    pub fn new() -> Self {
        unsafe {
            let did = DiagnosticId(DIAGNOSTIC_ID_GENERATOR);
            DIAGNOSTIC_ID_GENERATOR += 1;
            did
        }
    }
}

static mut DIAGNOSTIC_ID_GENERATOR: u32 = 0;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DiagnosticHint(pub String);

#[derive(Debug, Clone, Serialize, Eq, PartialEq, thiserror::Error)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum DiagnosticKind {
    #[error("Import not found: {0}")]
    ImportNotFound(Path),
    #[error("Reference not found: {0}")]
    ReferenceNotFound(Reference),
    #[error("Ambiguous reference: {0}")]
    AmbiguousReference(Reference),
    #[error("Redefined entity: {0}")]
    RedefinedEntity(Path),
    #[error("Expected {0} got {1}")]
    UnexpectedType(TypeKind, TypeKind),
    #[error("Not an attribute: {0}")]
    NotAnAttribute(Reference),
    #[error("Not a set: {0}")]
    NotASet(TypeKind),
}

impl PartialOrd for DiagnosticKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        DiagnosticSeverity::from(self).partial_cmp(&other.into())
    }
}

impl Ord for DiagnosticKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        DiagnosticSeverity::from(self).cmp(&other.into())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Critical,
    Severe,
    Moderate,
    Light,
}

impl From<&DiagnosticKind> for DiagnosticSeverity {
    fn from(kind: &DiagnosticKind) -> DiagnosticSeverity {
        use DiagnosticKind::*;
        use DiagnosticSeverity::*;

        match kind {
            ImportNotFound(_) => Critical,
            ReferenceNotFound(_) => Critical,
            AmbiguousReference(_) => Critical,
            RedefinedEntity(_) => Critical,
            UnexpectedType(_, _) => Critical,
            NotAnAttribute(_) => Critical,
            NotASet(_) => Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Diagnostic {
    pub did: DiagnosticId,
    pub id: NodeId,
    pub kind: DiagnosticKind,
    pub severity: DiagnosticSeverity,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Diagnostic {
    pub fn new(id: NodeId, kind: DiagnosticKind) -> Self {
        let did = DiagnosticId::new();
        let severity = DiagnosticSeverity::from(&kind);
        Self {
            did,
            id,
            kind,
            severity,
        }
    }

    pub fn propagated(referer: NodeId, err: Diagnostic) -> Self {
        Self {
            id: referer,
            did: err.did,
            kind: err.kind,
            severity: err.severity,
        }
    }
}

impl std::error::Error for Diagnostic {}

pub struct ResultErr<U, T, E>(pub (U, Result<T, E>));

impl<U, T, E> ResultErr<U, T, E> {
    fn unwrap(self) -> (U, Result<T, E>) {
        let Self((key, result)) = self;
        (key, result)
    }
}

impl<T, E, U> FromIterator<ResultErr<U, T, E>> for (HashMap<U, T>, Vec<E>)
where
    U: Eq + std::hash::Hash,
{
    fn from_iter<I: IntoIterator<Item = ResultErr<U, T, E>>>(iter: I) -> (HashMap<U, T>, Vec<E>) {
        let (ok, errs): (Vec<_>, Vec<_>) = iter
            .into_iter()
            .map(ResultErr::unwrap)
            .partition(|(_, result)| result.is_ok());
        (
            ok.into_iter()
                .map(|(u, ok)| (u, ok.ok().unwrap()))
                .collect(),
            errs.into_iter()
                .map(|(_, err)| err.err().unwrap())
                .collect(),
        )
    }
}
