use super::{NodeId, ReferenceId, expression, markup};
use crate::{
    children::{Children, ChildrenIter, ChildrenIterMut, ChildrenMut, IntoChildren},
    iter::{NodeIter, UnsafeNodeIterMut},
};
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use tokenizer::Keyword;

pub mod parser;
pub mod tokenizer;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Entity {
    pub id: NodeId,
    #[serde(flatten)]
    pub meta: EntityMeta,
    #[serde(flatten)]
    pub variant: EntityVariant,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant && self.meta == other.meta
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            id: NodeId::default(),
            meta: EntityMeta::default(),
            variant: EntityVariant::Package(Package::default()),
        }
    }
}

impl Children for Entity {
    fn children(&self) -> Option<&Vec<Entity>> {
        match &self.variant {
            EntityVariant::Part(part) => Some(&part.children),
            EntityVariant::Package(package) => Some(&package.children),
            _ => None,
        }
    }
}

impl ChildrenIter for Entity {
    fn children_iter(&self) -> impl Iterator<Item = &Self> {
        self.children().into_iter().map(|c| c.iter()).flatten()
    }
}

impl ChildrenMut for Entity {
    fn children_mut(&mut self) -> Option<&mut Vec<Entity>> {
        match &mut self.variant {
            EntityVariant::Part(part) => Some(&mut part.children),
            EntityVariant::Package(package) => Some(&mut package.children),
            _ => None,
        }
    }
}

impl IntoChildren for Entity {
    fn into_children(self) -> Option<Vec<Self>> {
        match self.variant {
            EntityVariant::Part(part) => Some(part.children),
            EntityVariant::Package(package) => Some(package.children),
            _ => None,
        }
    }
}

impl ChildrenIterMut for Entity {
    fn children_iter_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        self.children_mut()
            .into_iter()
            .map(|c| c.iter_mut())
            .flatten()
    }
}

impl Entity {
    pub fn new(id: NodeId, entity: EntityMeta, variant: EntityVariant) -> Self {
        Entity {
            id,
            meta: entity,
            variant,
        }
    }

    pub fn part(&self) -> Option<&Part> {
        match &self.variant {
            EntityVariant::Part(part) => Some(part),
            _ => None,
        }
    }

    pub fn into_package(self) -> Option<Package> {
        match self.variant {
            EntityVariant::Package(package) => Some(package),
            _ => None,
        }
    }

    pub fn package(&self) -> Option<&Package> {
        match &self.variant {
            EntityVariant::Package(package) => Some(package),
            _ => None,
        }
    }

    pub fn package_mut(&mut self) -> Option<&mut Package> {
        match &mut self.variant {
            EntityVariant::Package(package) => Some(package),
            _ => None,
        }
    }

    pub fn requirement(&self) -> Option<&Requirement> {
        match &self.variant {
            EntityVariant::Requirement(requirement) => Some(requirement),
            _ => None,
        }
    }

    pub fn requirement_mut(&mut self) -> Option<&mut Requirement> {
        match &mut self.variant {
            EntityVariant::Requirement(requirement) => Some(requirement),
            _ => None,
        }
    }

    pub fn references(&self) -> impl Iterator<Item = &Reference> {
        let mut ret = Vec::new();

        match &self.variant {
            EntityVariant::Package(package) => {
                ret.extend(&package.imports);
            }

            EntityVariant::Part(part) => {
                ret.extend(&part.imports);
            }

            EntityVariant::Requirement(requirement) => {
                ret.extend(requirement.traceability.iter().map(|t| &t.target));
            }

            _ => {}
        }

        ret.into_iter()
    }

    pub fn references_mut(&mut self) -> impl Iterator<Item = &mut Reference> {
        let mut ret = Vec::new();

        match &mut self.variant {
            EntityVariant::Package(package) => {
                ret.extend(&mut package.imports);
            }

            EntityVariant::Part(part) => {
                ret.extend(&mut part.imports);
            }

            EntityVariant::Requirement(requirement) => {
                ret.extend(requirement.traceability.iter_mut().map(|t| &mut t.target));
            }
            _ => {}
        }

        ret.into_iter()
    }

    pub fn expressions(&self) -> impl Iterator<Item = &expression::Expression> {
        match &self.variant {
            EntityVariant::Attribute(attribute) => Some(&attribute.domain),
            EntityVariant::Requirement(requirement) => match &requirement.variant {
                RequirementVariant::Informal(_) => None,
                RequirementVariant::Formal(expr) => Some(expr),
            },
            _ => None,
        }
        .into_iter()
    }

    pub fn expressions_mut(&mut self) -> impl Iterator<Item = &mut expression::Expression> {
        match &mut self.variant {
            EntityVariant::Attribute(attribute) => Some(&mut attribute.domain),
            EntityVariant::Requirement(requirement) => match &mut requirement.variant {
                RequirementVariant::Informal(_) => None,
                RequirementVariant::Formal(expr) => Some(expr),
            },
            _ => None,
        }
        .into_iter()
    }

    pub fn markups(&self) -> impl Iterator<Item = &markup::Markup> {
        let mut ret = Vec::new();

        if let Some(comment) = &self.meta.comment {
            ret.push(comment)
        }

        match &self.variant {
            EntityVariant::Requirement(requirement) => match &requirement.variant {
                RequirementVariant::Informal(markup) => ret.push(markup),
                RequirementVariant::Formal(_) => {}
            },
            _ => {}
        }

        ret.into_iter()
    }

    pub fn markups_mut(&mut self) -> impl Iterator<Item = &mut markup::Markup> {
        let mut ret = Vec::new();

        if let Some(comment) = &mut self.meta.comment {
            ret.push(comment)
        }

        match &mut self.variant {
            EntityVariant::Requirement(requirement) => match &mut requirement.variant {
                RequirementVariant::Informal(markup) => ret.push(markup),
                RequirementVariant::Formal(_) => {}
            },
            _ => {}
        }

        ret.into_iter()
    }

    pub fn imports(&self) -> Option<&Vec<Reference>> {
        match &self.variant {
            EntityVariant::Part(part) => Some(&part.imports),
            EntityVariant::Package(package) => Some(&package.imports),
            _ => None,
        }
    }

    pub fn imports_mut(&mut self) -> Option<&mut Vec<Reference>> {
        match &mut self.variant {
            EntityVariant::Part(part) => Some(&mut part.imports),
            EntityVariant::Package(package) => Some(&mut package.imports),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Self> {
        NodeIter::new(self)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        UnsafeNodeIterMut::new(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum EntityVariant {
    Part(Part),
    Attribute(Attribute),
    Package(Package),
    Requirement(Requirement),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Part {
    pub imports: Vec<Reference>,
    pub children: Vec<Entity>,
}

impl Part {
    pub fn new(imports: Vec<Reference>, children: Vec<Entity>) -> Self {
        Part { imports, children }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Attribute {
    pub domain: expression::Expression,
    pub unit: Option<expression::Expression>,
}

impl Attribute {
    pub fn new(domain: expression::Expression) -> Self {
        Attribute { domain, unit: None }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Package {
    pub imports: Vec<Reference>,
    pub children: Vec<Entity>,
}

impl Package {
    pub fn new(imports: Vec<Reference>, children: Vec<Entity>) -> Self {
        Package { imports, children }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Requirement {
    pub traceability: Vec<TraceabilityLink>,
    pub variant: RequirementVariant,
}

impl Requirement {
    pub fn new(variant: RequirementVariant) -> Self {
        Requirement {
            traceability: Vec::new(),
            variant,
        }
    }

    pub fn informal(&self) -> Option<&markup::Markup> {
        match &self.variant {
            RequirementVariant::Informal(markup) => Some(markup),
            _ => None,
        }
    }

    pub fn informal_mut(&mut self) -> Option<&mut markup::Markup> {
        match &mut self.variant {
            RequirementVariant::Informal(markup) => Some(markup),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum RequirementVariant {
    Formal(expression::Expression),
    Informal(markup::Markup),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum TraceabilityKind {
    Specialization,
    Refinement,
    Derivation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct TraceabilityLink {
    #[serde(rename = "type")]
    pub kind: TraceabilityKind,
    pub target: Reference,
}

impl TraceabilityLink {
    pub fn new(kind: TraceabilityKind, target: Reference) -> Self {
        TraceabilityLink { kind, target }
    }
}

impl<'a> TryFrom<&'a Keyword> for TraceabilityKind {
    type Error = &'a Keyword;

    fn try_from(value: &'a Keyword) -> Result<Self, Self::Error> {
        match value {
            Keyword::Refines => Ok(TraceabilityKind::Refinement),
            Keyword::Specializes => Ok(TraceabilityKind::Specialization),
            Keyword::Derives => Ok(TraceabilityKind::Derivation),
            v => Err(v),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Reference {
    pub id: NodeId,
    pub rid: ReferenceId,
    pub value: String,
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Reference {
    pub fn new(id: NodeId, rid: ReferenceId, value: String) -> Self {
        Reference { id, rid, value }
    }

    pub fn len(&self) -> usize {
        self.value.split("::").count()
    }

    pub fn replace_label(&mut self, label: &str) {
        match self.value.rfind("::") {
            Some(slice_idx) => self.value.replace_range(slice_idx.., label),
            None => self.value = label.to_string(),
        }
    }

    pub fn label(&self) -> &str {
        match self.value.rfind("::") {
            Some(slice_idx) => &self.value[slice_idx + 2..],
            None => &self.value,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct EntityMeta {
    pub label: String,
    pub tags: Vec<Tag>,
    pub comment: Option<markup::Markup>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Tag {
    pub id: String,
    pub value: Option<String>,
}

impl Tag {
    pub fn new(id: String, value: Option<String>) -> Self {
        Self { id, value }
    }
}
