use std::{collections::HashMap, ops::ControlFlow};

use crate::{
    diagnostic::Diagnostic,
    mapper::path::{Path, PathMap},
    syntax::entity::Entity,
    visitor::{Visitor, WalkPre},
};

pub struct Verifier<'a> {
    paths: &'a PathMap,
    nodes_by_path: HashMap<&'a Path, &'a Entity>,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Visitor<&'a Entity> for Verifier<'a> {
    type WalkKind = WalkPre;

    fn visit(&mut self, node: &'a Entity) -> ControlFlow<()> {
        use crate::diagnostic::DiagnosticKind::RedefinedEntity;

        let path = self.paths.get(&node.id).unwrap();

        if let Some(_) = self.nodes_by_path.insert(&path, node) {
            let err = Diagnostic::new(node.id, RedefinedEntity(path.clone()));
            self.diagnostics.push(err);
        }

        ControlFlow::Continue(())
    }
}

impl<'a> Verifier<'a> {
    pub fn new(paths: &'a PathMap) -> Self {
        Self {
            paths,
            diagnostics: Vec::default(),
            nodes_by_path: HashMap::default(),
        }
    }
}

pub type PathNodeMap<'a> = HashMap<&'a Path, &'a Entity>;
