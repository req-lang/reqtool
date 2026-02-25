pub trait Children
where
    Self: Sized,
{
    fn children(&self) -> Option<&Vec<Self>>;
}

pub trait ChildrenMut
where
    Self: Sized,
{
    fn children_mut(&mut self) -> Option<&mut Vec<Self>>;
}

pub trait ChildrenIter {
    fn children_iter(&self) -> impl Iterator<Item = &Self>;
}

pub trait ChildrenIterMut {
    fn children_iter_mut(&mut self) -> impl Iterator<Item = &mut Self>;
}

pub trait IntoChildren
where
    Self: Sized,
{
    fn into_children(self) -> Option<Vec<Self>>;
}
