use crate::syntax::{self};

/// This module generates basic mapping between syntax node and node id.
/// Might be useful for later usage

impl<'a> From<&'a syntax::entity::Entity> for NodeMap<'a> {
    fn from(node: &'a syntax::entity::Entity) -> Self {
        let mut ret = Self::default();
        for n in node.iter() {
            ret.entities.insert(n.id, n);
            for expression in n.expressions() {
                for e in expression.iter() {
                    ret.expressions.insert(e.id, e);
                }
            }

            for m in n.markups() {
                ret.markups.insert(m.id, m);
            }
        }
        ret
    }
}

#[derive(Default, Debug)]
pub struct NodeMap<'a> {
    pub entities: entity::NodeMap<'a>,
    pub expressions: expression::NodeMap<'a>,
    pub markups: markup::NodeMap<'a>,
}

pub mod entity {
    use std::collections::HashMap;

    use crate::syntax::{NodeId, entity::Entity};

    pub type NodeMap<'a> = HashMap<NodeId, &'a Entity>;
}

pub mod expression {
    use std::collections::HashMap;

    use crate::syntax::{NodeId, expression::Expression};

    pub type NodeMap<'a> = HashMap<NodeId, &'a Expression>;
}

pub mod markup {
    use std::collections::HashMap;

    use crate::syntax::{NodeId, markup::Markup};

    pub type NodeMap<'a> = HashMap<NodeId, &'a Markup>;
}
