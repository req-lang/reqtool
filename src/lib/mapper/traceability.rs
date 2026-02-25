use std::{collections::HashMap, ops::ControlFlow};

use crate::{
    syntax::{NodeId, entity},
    verifier::reference::ReferenceMap,
    visitor::{Visitor, WalkPre},
};
pub type TraceabilityMap = HashMap<NodeId, Vec<NodeId>>;

pub struct Mapper<'a> {
    references: &'a ReferenceMap,
    pub map: TraceabilityMap,
}

impl Visitor<&entity::Entity> for Mapper<'_> {
    type WalkKind = WalkPre;

    fn visit(&mut self, node: &entity::Entity) -> std::ops::ControlFlow<()> {
        use entity::EntityVariant::*;

        if let Requirement(requirement) = &node.variant {
            for traceability in &requirement.traceability {
                if let Some(reference) = self.references.get(&traceability.target.rid) {
                    let source = *reference;
                    match self.map.get_mut(reference) {
                        Some(traces) => {
                            traces.push(node.id);
                        }
                        None => {
                            self.map.insert(source, vec![node.id]);
                        }
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }
}

impl<'a> Mapper<'a> {
    pub fn new(references: &'a ReferenceMap) -> Self {
        Self {
            references,
            map: HashMap::new(),
        }
    }
}
