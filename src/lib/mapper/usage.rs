use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
};

use crate::{
    syntax::{
        NodeId,
        entity::{self, Reference},
        expression,
    },
    verifier::reference::ReferenceMap,
    visitor::{Visitor, Walk, WalkPre},
};

pub type UsageMap = HashMap<NodeId, HashSet<NodeId>>;

pub struct Mapper<'a> {
    references: &'a ReferenceMap,
    current: Option<NodeId>,
    pub map: UsageMap,
}

impl Visitor<&entity::Entity> for Mapper<'_> {
    type WalkKind = WalkPre;

    fn visit(&mut self, node: &entity::Entity) -> std::ops::ControlFlow<()> {
        use entity::EntityVariant::*;
        use entity::RequirementVariant::*;

        if let Requirement(requirement) = &node.variant {
            self.current = Some(node.id);
            match &requirement.variant {
                Formal(node) => {
                    self.walk(node)?;
                }
                Informal(container) => {
                    for reference in container.references() {
                        self.insert(reference);
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }
}

impl Visitor<&expression::Expression> for Mapper<'_> {
    type WalkKind = WalkPre;

    fn visit(&mut self, node: &expression::Expression) -> ControlFlow<(), ()> {
        use expression::ExpressionVariant::*;

        if let Identifier(ident) = &node.variant {
            self.insert(&ident.target);
        }

        ControlFlow::Continue(())
    }
}

impl Mapper<'_> {
    pub fn insert(&mut self, reference: &Reference) {
        let requirement = self.current.unwrap();
        if let Some(id) = self.references.get(&reference.rid) {
            match self.map.get_mut(id) {
                Some(usages) => {
                    usages.insert(requirement);
                }
                None => {
                    let mut set = HashSet::new();
                    set.insert(requirement);
                    self.map.insert(*id, set);
                }
            }
        }
    }
}

impl<'a> Mapper<'a> {
    pub fn new(references: &'a ReferenceMap) -> Self {
        Self {
            references,
            current: None,
            map: HashMap::new(),
        }
    }
}
