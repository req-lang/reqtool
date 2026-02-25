pub mod entity {
    use crate::syntax::entity::{Entity, EntityVariant, Package};

    pub fn simple_tree() -> Entity {
        let mut first = Entity::default();
        let mut second = Entity::default();
        let mut third = Entity::default();
        let mut fourth = Entity::default();
        let mut fifth = Entity::default();

        first.id = 1.into();
        second.id = 2.into();
        third.id = 3.into();
        fourth.id = 4.into();
        fifth.id = 5.into();

        let variant = EntityVariant::Package(Package::new(vec![], vec![fourth]));
        second.variant = variant;

        let variant = EntityVariant::Package(Package::new(vec![], vec![fifth]));
        third.variant = variant;

        let variant = EntityVariant::Package(Package::new(vec![], vec![second, third]));
        first.variant = variant;

        return first;
    }
}

pub mod generator {
    use crate::children::ChildrenMut;
    use crate::syntax::entity::{Entity, EntityVariant, Requirement, RequirementVariant};
    use crate::syntax::{NodeId, markup};

    pub struct ExponentialSizeIterator {
        value: f64,
    }

    impl ExponentialSizeIterator {
        pub fn new() -> Self {
            Self { value: 1.0 }
        }
    }

    impl Iterator for ExponentialSizeIterator {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            let ret = self.value;
            self.value *= (10.0 as f64).sqrt();
            Some(ret as u64)
        }
    }

    pub trait Generate {
        fn generate(&self) -> Entity;

        fn size(&self) -> u64;
    }

    // Count of nodes: packages * depth * (requirement + 1)
    pub struct Simple {
        pub packages: u64,
        pub depth: u64,
        pub requirements: u64,
        pub words: u64,
    }

    impl Generate for Simple {
        fn generate(&self) -> Entity {
            let mut root = Entity::default();
            root.id = NodeId::new();
            root.meta.label = format!("root");

            for p in 0..self.packages {
                let mut pkg = Entity::default();
                pkg.id = NodeId::new();
                pkg.meta.label = format!("package_{p}");
                root.children_mut().unwrap().push(pkg);

                let mut prev = root.children_mut().unwrap().last_mut().unwrap();
                for d in 0..self.depth {
                    let mut curr = Entity::default();
                    curr.id = NodeId::new();
                    curr.meta.label = format!("package_{p}_{d}");

                    for r in 0..self.requirements {
                        let mut req = Entity::default();
                        let variant = RequirementVariant::Informal(markup::Markup::from(
                            vec!["word"]
                                .into_iter()
                                .cycle()
                                .take(self.words as usize)
                                .collect::<Vec<_>>()
                                .join(" "),
                        ));

                        req.id = NodeId::new();
                        req.variant = EntityVariant::Requirement(Requirement::new(variant));
                        req.meta.label = format!("requirement_{p}_{d}_{r}");

                        curr.children_mut().unwrap().push(req);
                    }

                    prev.children_mut().unwrap().push(curr);
                    prev = prev.children_mut().unwrap().last_mut().unwrap();
                }
            }

            root
        }

        fn size(&self) -> u64 {
            self.packages * (self.depth * (self.requirements + 1) + 1) + 1
        }
    }

    impl Simple {
        pub fn new() -> Self {
            Self {
                packages: 10,
                depth: 5,
                requirements: 5,
                words: 200,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn simple_has_expected_size() {
            let mut generator = Simple::new();

            let root = generator.generate();
            assert_eq!(root.iter().count() as u64, generator.size());

            generator.packages = 5;
            let root = generator.generate();
            assert_eq!(root.iter().count() as u64, generator.size());
        }
    }
}
