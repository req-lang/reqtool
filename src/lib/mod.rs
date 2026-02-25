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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// For commercial licensing, see COMMERCIAL.md

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
