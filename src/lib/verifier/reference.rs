use std::{collections::HashMap, ops::ControlFlow};

use itertools::Itertools;

use crate::{
    children::{Children, ChildrenIter},
    diagnostic::Diagnostic,
    mapper::path::{Path, PathMap},
    syntax::{
        self, NodeId, ReferenceId,
        entity::{Entity, Reference},
        expression::Expression,
    },
    visitor::{Visitor, Walk, WalkCustom},
};

pub struct Verifier<'a, 'b> {
    paths: &'a PathMap,
    inverse_paths: HashMap<&'a Path, &'a NodeId>,
    parent: Vec<&'a Path>,
    local: HashMap<&'b str, &'b NodeId>,
    imports: Vec<Vec<&'a Path>>,
    pub results: HashMap<ReferenceId, Result<NodeId, Diagnostic>>,
}

impl<'a, 'b> Walk<&'b Entity> for Verifier<'a, 'b> {
    fn walk(&mut self, node: &'b Entity) -> ControlFlow<()> {
        self.visit(node)?;

        if let Some(children) = node.children() {
            self.parent.push(&self.paths.get(&node.id).unwrap());
            self.imports.push(
                node.imports()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| self.results.get(&r.rid).unwrap().as_ref().ok())
                    .map(|found| self.paths.get(&found).unwrap())
                    .collect(),
            );
            for n in children {
                self.walk(n)?;
            }
            self.parent.pop();
            self.imports.pop();
        }

        ControlFlow::Continue(())
    }
}

impl<'a, 'b> Visitor<&'b Entity> for Verifier<'a, 'b> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &'b Entity) -> ControlFlow<()> {
        for reference in node.references() {
            let found = self.find(reference);
            self.results.insert(reference.rid, found);
        }

        for expression in node.expressions() {
            self.walk(expression)?;
        }

        for markup in node.markups() {
            for reference in markup.references() {
                let found = self.find(reference);
                self.results.insert(reference.rid, found);
            }
        }

        ControlFlow::Continue(())
    }
}

impl<'a, 'b> Walk<&'b Expression> for Verifier<'a, 'b> {
    fn walk(&mut self, node: &'b Expression) -> ControlFlow<()> {
        self.visit(node)?;

        let old = match node.set_element() {
            None => None,
            Some(set) => self.local.insert(&set.variable, &set.domain.id),
        };

        for child in node.children_iter() {
            self.walk(child)?;
        }

        if let Some(set) = node.set_element() {
            let label = &set.variable[..];
            match old {
                None => self.local.remove(label),
                Some(id) => self.local.insert(label, id),
            };
        }

        ControlFlow::Continue(())
    }
}

impl<'a, 'b> Visitor<&'b Expression> for Verifier<'a, 'b> {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &'b Expression) -> ControlFlow<()> {
        use syntax::expression::ExpressionVariant::*;

        match &node.variant {
            Identifier(ident) if !is_builtin(&ident.target.value) => {
                let found = self.find(&ident.target);
                self.results.insert(ident.target.rid, found);
            }

            _ => {}
        }

        ControlFlow::Continue(())
    }
}

impl Verifier<'_, '_> {
    fn find(&self, reference: &Reference) -> Result<NodeId, Diagnostic> {
        use crate::diagnostic::DiagnosticKind::{AmbiguousReference, ReferenceNotFound};

        if let Some(id) = self.local.get(&reference.value[..]) {
            return Ok(**id);
        }

        let path = Path::from(reference.value.to_string());
        let as_path = std::iter::once(path);

        let parent = self.parent.last();
        let as_inner = parent.map(|p| p.appended(&reference.value)).into_iter();

        let imports = self.imports.last().unwrap();
        let as_imported = imports.iter().map(|p| p.merged(&reference.value));

        let mut hypotheses = as_inner
            .chain(as_path)
            .chain(as_imported)
            .filter_map(|p| self.inverse_paths.get(&Path::from(p)))
            .unique();

        let id = reference.id;
        match hypotheses.next() {
            None => Err(Diagnostic::new(id, ReferenceNotFound(reference.clone()))),
            Some(found) => match hypotheses.next() {
                None => Ok(**found),
                Some(_) => Err(Diagnostic::new(id, AmbiguousReference(reference.clone()))),
            },
        }
    }
}

impl<'a> Verifier<'a, '_> {
    pub fn new(paths: &'a PathMap) -> Self {
        let inverse_paths = paths.iter().map(|(id, path)| (path, id)).collect();
        Self {
            paths,
            inverse_paths,
            parent: Vec::new(),
            local: HashMap::new(),
            results: HashMap::new(),
            imports: vec![Vec::new()],
        }
    }
}

pub type ReferenceMap = HashMap<ReferenceId, NodeId>;

#[inline]
pub fn is_builtin(val: &str) -> bool {
    match val {
        "real" | "integer" | "boolean" | "string" => true,
        _ => false,
    }
}
