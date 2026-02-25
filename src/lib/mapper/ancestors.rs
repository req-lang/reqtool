use std::{collections::HashMap, ops::ControlFlow};

use crate::{
    children::{Children, ChildrenIter},
    syntax::{NodeId, entity, expression},
    visitor::{Visitor, Walk, WalkCustom},
};

#[derive(Default, Debug)]
struct Mapper {
    current: Vec<NodeId>,
    map: AncestorsMap,
}

impl Walk<&entity::Entity> for Mapper {
    fn walk(&mut self, node: &entity::Entity) -> ControlFlow<()> {
        self.visit(node)?;

        self.current.push(node.id);
        if let Some(children) = node.children() {
            for n in children {
                self.walk(n)?;
            }
        }
        self.current.pop();

        ControlFlow::Continue(())
    }
}

impl Visitor<&entity::Entity> for Mapper {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &entity::Entity) -> ControlFlow<()> {
        self.map.insert(node.id, self.current.clone());

        self.current.push(node.id);
        for m in node.markups() {
            self.map.insert(m.id, self.current.clone());
        }

        for e in node.expressions() {
            self.walk(e)?;
        }
        self.current.pop();

        ControlFlow::Continue(())
    }
}

impl Walk<&expression::Expression> for Mapper {
    fn walk(&mut self, node: &expression::Expression) -> ControlFlow<()> {
        self.visit(node)?;

        self.current.push(node.id);
        for n in node.children_iter() {
            self.walk(n)?;
        }
        self.current.pop();

        ControlFlow::Continue(())
    }
}

impl Visitor<&expression::Expression> for Mapper {
    type WalkKind = WalkCustom;

    fn visit(&mut self, node: &expression::Expression) -> ControlFlow<()> {
        self.map.insert(node.id, self.current.clone());
        ControlFlow::Continue(())
    }
}

impl From<&entity::Entity> for AncestorsMap {
    fn from(node: &entity::Entity) -> Self {
        let mut mapper = Mapper::default();
        let _ = mapper.walk(node);
        mapper.map
    }
}

pub type AncestorsMap = HashMap<NodeId, Vec<NodeId>>;
