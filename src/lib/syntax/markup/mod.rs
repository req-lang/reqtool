use serde_derive::{Deserialize, Serialize};
use tokenizer::{IntoTokens, Token};

use super::{Context, NodeId, NodeParser, Position, ReferenceId, Span, entity::Reference};

pub mod tokenizer;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum Item {
    Text(String),
    Reference(Reference),
}

impl Item {
    pub fn to_string(self) -> String {
        match self {
            Item::Text(s) => s,
            Item::Reference(r) => format!("{{{}}}", r.value),
        }
    }

    pub fn from_token(id: NodeId, token: Token<'_>) -> Self {
        match token {
            Token::Text(text) => Item::Text(text.to_string()),
            Token::Reference(ident) => {
                let rid = ReferenceId::new();
                Item::Reference(Reference::new(id, rid, ident.to_string()))
            }
        }
    }
}

///This container own the markdown text associated with markup tokens
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Markup {
    pub id: NodeId,
    pub value: Vec<Item>,
}

impl From<String> for Markup {
    fn from(value: String) -> Self {
        let id = NodeId::new();
        let tokens = value.char_indices().tokens();
        let value = tokens.map(|token| Item::from_token(id, token)).collect();

        Self { id, value }
    }
}

impl PartialEq for Markup {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Markup {
    pub fn to_string(self) -> String {
        self.value
            .into_iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn references(&self) -> impl Iterator<Item = &Reference> {
        self.value.iter().filter_map(|item| match item {
            Item::Reference(r) => Some(r),
            _ => None,
        })
    }

    pub fn references_mut(&mut self) -> impl Iterator<Item = &mut Reference> {
        self.value.iter_mut().filter_map(|item| match item {
            Item::Reference(r) => Some(r),
            _ => None,
        })
    }

    pub fn push_str(&mut self, str: &str) {
        let items = str
            .char_indices()
            .tokens()
            .map(|t| Item::from_token(self.id, t));
        self.value.extend(items);
    }
}

impl NodeParser<'_> {
    pub fn parse_markup(&mut self, value: String, end: Position) -> Markup {
        let container = Markup::from(value);
        let span = Span::new(self.offset, end);

        self.context.insert(container.id, Context::new(span));
        container
    }
}
