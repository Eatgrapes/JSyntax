use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub arguments: AnnotationArguments,
}

impl Annotation {
    pub fn marker(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: AnnotationArguments::Marker,
        }
    }
    pub fn single(name: impl Into<String>, value: impl Into<AnnotationValue>) -> Self {
        Self {
            name: name.into(),
            arguments: AnnotationArguments::Single(Box::new(value.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationArguments {
    Marker,
    Single(Box<AnnotationValue>),
    Named(Vec<(String, AnnotationValue)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationValue {
    Expression(Box<Expr>),
    Annotation(Annotation),
    Array(Vec<AnnotationValue>),
}

impl From<Expr> for AnnotationValue {
    fn from(value: Expr) -> Self {
        Self::Expression(Box::new(value))
    }
}

impl From<Annotation> for AnnotationValue {
    fn from(value: Annotation) -> Self {
        Self::Annotation(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comment {
    Line(String),
    Block(String),
    Javadoc(String),
}
