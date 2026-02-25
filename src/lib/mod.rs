use diagnostic::{Diagnostic, ResultErr};
use mapper::{
    ancestors::{self, AncestorsMap},
    id,
    path::{self, PathMap},
    traceability::{self, TraceabilityMap},
    usage::{self, UsageMap},
};
use serde_derive::Serialize;
use syntax::entity::Entity;
use verifier::{
    definition::{self},
    reference::{self, ReferenceMap},
    typing::{self, TypeMap},
};
use visitor::Walk;

use crate::mapper::tags::{self, TagMap};

pub mod children;
pub mod diagnostic;
pub mod iter;
pub mod mapper;
pub mod mock;
pub mod renderer;
pub mod syntax;
pub mod verifier;
pub mod visitor;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Analysis {
    pub paths: PathMap,
    pub ancestors: AncestorsMap,
    pub references: ReferenceMap,
    pub types: TypeMap,
    pub traceabilities: TraceabilityMap,
    pub usages: UsageMap,
    pub tags: TagMap,
    pub diagnostics: Vec<Diagnostic>,
}

impl From<&Entity> for Analysis {
    fn from(root: &Entity) -> Self {
        let paths = path::PathMap::from(root);
        let nodes_by_id = id::NodeMap::from(root);
        let ancestors = ancestors::AncestorsMap::from(root);

        let mut verifier = definition::Verifier::new(&paths);
        let _ = verifier.walk(root);
        let mut diagnostics = verifier.diagnostics;

        let mut verifier = reference::Verifier::new(&paths);
        let _ = verifier.walk(root);
        let (references, errs) = verifier.results.into_iter().map(ResultErr).collect();
        diagnostics.extend(errs);

        let mut verifier = typing::Verifier::new(&references, &nodes_by_id);
        let _ = verifier.prewalk(root);
        let _ = verifier.walk(root);
        let (types, errs) = verifier.results.into_iter().map(ResultErr).collect();
        diagnostics.extend(errs);

        let mut mapper = traceability::Mapper::new(&references);
        let _ = mapper.walk(root);
        let traceabilities = mapper.map;

        let mut mapper = usage::Mapper::new(&references);
        let _ = mapper.walk(root);
        let usages = mapper.map;

        let mut mapper = tags::Mapper::default();
        let _ = mapper.walk(root);
        let tags = mapper.map;

        Self {
            paths,
            ancestors,
            references,
            types,
            traceabilities,
            diagnostics,
            usages,
            tags,
        }
    }
}
