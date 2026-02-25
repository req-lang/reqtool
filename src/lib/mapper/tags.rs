use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
};

use crate::{
    syntax,
    visitor::{Visitor, WalkPre},
};

pub type TagMap = HashMap<String, HashSet<Option<String>>>;

#[derive(Default, Debug)]
pub struct Mapper {
    pub map: TagMap,
}

impl Visitor<&syntax::entity::Entity> for Mapper {
    type WalkKind = WalkPre;

    fn visit(&mut self, node: &syntax::entity::Entity) -> ControlFlow<()> {
        for tag in &node.meta.tags {
            match self.map.get_mut(&tag.id) {
                Some(values) => {
                    values.insert(tag.value.clone());
                }
                None => {
                    let mut set = HashSet::new();
                    set.insert(tag.value.clone());
                    self.map.insert(tag.id.clone(), set);
                }
            }
        }

        ControlFlow::Continue(())
    }
}
