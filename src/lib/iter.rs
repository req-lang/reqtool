use crate::children::{ChildrenIter, ChildrenIterMut};

pub struct NodeIter<'a, N> {
    stack: Vec<&'a N>,
}

impl<'a, N> NodeIter<'a, N> {
    pub fn new(node: &'a N) -> Self {
        NodeIter { stack: vec![node] }
    }
}

impl<'a, N> Iterator for NodeIter<'a, N>
where
    N: ChildrenIter,
{
    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let children = node.children_iter().collect::<Vec<_>>().into_iter().rev();
        self.stack.extend(children);
        Some(node)
    }
}

pub struct UnsafeNodeIterMut<'a, N> {
    stack: Vec<&'a mut N>,
}

impl<'a, N> UnsafeNodeIterMut<'a, N> {
    pub fn new(node: &'a mut N) -> Self {
        UnsafeNodeIterMut { stack: vec![node] }
    }
}

impl<'a, N> Iterator for UnsafeNodeIterMut<'a, N>
where
    N: ChildrenIterMut,
{
    type Item = &'a mut N;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let ret = std::ptr::from_mut(node);
        let children = node
            .children_iter_mut()
            .collect::<Vec<_>>()
            .into_iter()
            .rev();
        self.stack.extend(children);
        Some(unsafe { ret.as_mut().unwrap() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock,
        syntax::{NodeId, entity::EntityVariant},
    };

    #[test]
    fn is_depth_first() {
        let root = mock::entity::simple_tree();
        let ids: Vec<NodeId> = NodeIter::new(&root).map(|n| n.id).collect();

        assert_eq!(ids, vec![1.into(), 2.into(), 4.into(), 3.into(), 5.into()])
    }

    #[test]
    fn modifies_nodes() {
        let mut root = mock::entity::simple_tree();
        UnsafeNodeIterMut::new(&mut root)
            .enumerate()
            .for_each(|(idx, n)| {
                n.id = NodeId::from(idx as u32);
            });

        let ids: Vec<NodeId> = NodeIter::new(&root).map(|n| n.id).collect();
        assert_eq!(ids, vec![0.into(), 1.into(), 2.into(), 3.into(), 4.into()])
    }

    #[test]
    fn deletes_nodes() {
        let mut root = mock::entity::simple_tree();
        let target = UnsafeNodeIterMut::new(&mut root)
            .find(|n| n.id == 1.into())
            .unwrap();

        match &mut target.variant {
            EntityVariant::Package(package) => package.children.pop(),
            _ => None,
        };

        let ids: Vec<NodeId> = NodeIter::new(&root).map(|n| n.id).collect();
        assert_eq!(ids, vec![1.into(), 2.into(), 4.into()])
    }
}
