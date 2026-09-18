use super::{Annotation, Expr, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Type {
        annotations: Vec<Annotation>,
        final_: bool,
        ty: Type,
        name: String,
    },
    Record {
        ty: Type,
        components: Vec<Pattern>,
        binding: Option<String>,
    },
    /// An inferred component binding in a record pattern.
    Var {
        annotations: Vec<Annotation>,
        final_: bool,
        name: String,
    },
    Unnamed,
    /// Java 17-20 preview syntax.
    Parenthesized(Box<Pattern>),
    /// Java 17/18 preview syntax, before `when` guards.
    Guarded {
        pattern: Box<Pattern>,
        condition: Box<Expr>,
    },
}

impl Pattern {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var {
            annotations: Vec::new(),
            final_: false,
            name: name.into(),
        }
    }
    pub fn binding(ty: Type, name: impl Into<String>) -> Self {
        Self::Type {
            annotations: Vec::new(),
            final_: false,
            ty,
            name: name.into(),
        }
    }
    pub fn record(ty: Type, components: impl IntoIterator<Item = Pattern>) -> Self {
        Self::Record {
            ty,
            components: components.into_iter().collect(),
            binding: None,
        }
    }
}
