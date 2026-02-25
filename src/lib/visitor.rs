use std::ops::ControlFlow;

use crate::children::{ChildrenIter, ChildrenIterMut};
pub trait WalkKind {}

pub struct WalkPre;
pub struct WalkPost;
pub struct WalkCustom;

impl WalkKind for WalkPre {}
impl WalkKind for WalkPost {}
impl WalkKind for WalkCustom {}

pub trait Visitor<N, B = (), C = ()> {
    type WalkKind: WalkKind;

    fn visit(&mut self, node: N) -> ControlFlow<B, C>;
}

pub trait Walk<N, B = (), C = (), W = WalkCustom> {
    fn walk(&mut self, node: N) -> ControlFlow<B, C>;
}

impl<'a, N, V, B, C> Walk<&'a N, B, C, WalkPre> for V
where
    V: Visitor<&'a N, B, C, WalkKind = WalkPre>,
    N: ChildrenIter + 'a,
{
    fn walk(&mut self, node: &'a N) -> ControlFlow<B, C> {
        let ret = self.visit(node)?;
        for child in node.children_iter() {
            self.walk(child)?;
        }

        ControlFlow::Continue(ret)
    }
}

impl<'a, N, V, B, C> Walk<&'a N, B, C, WalkPost> for V
where
    V: Visitor<&'a N, B, C, WalkKind = WalkPost>,
    N: ChildrenIter + 'a,
{
    fn walk(&mut self, node: &'a N) -> ControlFlow<B, C> {
        for child in node.children_iter() {
            self.walk(child)?;
        }
        self.visit(node)
    }
}

pub trait VisitorMut<N, B = (), C = ()> {
    type WalkKind: WalkKind;

    fn visit_mut(&mut self, node: &mut N) -> ControlFlow<B, C>;
}

pub trait WalkMut<N, B = (), C = (), W = WalkCustom> {
    fn walk_mut(&mut self, node: &mut N) -> ControlFlow<B, C>;
}

impl<N, V, B, C> WalkMut<N, B, C, WalkPre> for V
where
    V: VisitorMut<N, B, C, WalkKind = WalkPre>,
    N: ChildrenIterMut,
{
    fn walk_mut(&mut self, node: &mut N) -> ControlFlow<B, C> {
        let ret = self.visit_mut(node)?;
        for child in node.children_iter_mut() {
            self.walk_mut(child)?;
        }

        ControlFlow::Continue(ret)
    }
}

impl<N, V, B, C> WalkMut<N, B, C, WalkPost> for V
where
    V: VisitorMut<N, B, C, WalkKind = WalkPost>,
    N: ChildrenIterMut,
{
    fn walk_mut(&mut self, node: &mut N) -> ControlFlow<B, C> {
        for child in node.children_iter_mut() {
            self.walk_mut(child)?;
        }
        self.visit_mut(node)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;

    use crate::{mock, syntax::entity::Entity, visitor::Walk};

    use super::{Visitor, WalkPost, WalkPre};

    struct PreTestVisitor {
        pub ids: Vec<u32>,
    }

    struct PostTestVisitor {
        pub ids: Vec<u32>,
    }

    impl<'a> Visitor<&'a Entity> for PreTestVisitor {
        type WalkKind = WalkPre;
        fn visit(&mut self, node: &Entity) -> ControlFlow<()> {
            self.ids.push(node.id.raw());
            ControlFlow::Continue(())
        }
    }

    impl<'a> Visitor<&'a Entity> for PostTestVisitor {
        type WalkKind = WalkPost;
        fn visit(&mut self, node: &Entity) -> ControlFlow<()> {
            self.ids.push(node.id.raw());
            ControlFlow::Continue(())
        }
    }

    #[test]
    fn walks_depth_first() {
        let mut root = mock::entity::simple_tree();
        let mut visitor = PreTestVisitor { ids: vec![] };

        let _ = visitor.walk(&mut root);

        assert_eq!(visitor.ids, vec![1, 2, 4, 3, 5])
    }

    #[test]
    fn walks_reverse_depth_first() {
        let mut root = mock::entity::simple_tree();
        let mut visitor = PostTestVisitor { ids: vec![] };

        let _ = visitor.walk(&mut root);

        assert_eq!(visitor.ids, vec![4, 2, 5, 3, 1])
    }
}
