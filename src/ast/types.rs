use super::Annotation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
}

impl PrimitiveType {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Byte => "byte",
            Self::Short => "short",
            Self::Char => "char",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Class(ClassType),
    Array {
        element: Box<Type>,
        dimension: ArrayDimension,
    },
    Annotated {
        annotations: Vec<Annotation>,
        ty: Box<Type>,
    },
}

impl Type {
    pub const BOOLEAN: Self = Self::Primitive(PrimitiveType::Boolean);
    pub const BYTE: Self = Self::Primitive(PrimitiveType::Byte);
    pub const SHORT: Self = Self::Primitive(PrimitiveType::Short);
    pub const CHAR: Self = Self::Primitive(PrimitiveType::Char);
    pub const INT: Self = Self::Primitive(PrimitiveType::Int);
    pub const LONG: Self = Self::Primitive(PrimitiveType::Long);
    pub const FLOAT: Self = Self::Primitive(PrimitiveType::Float);
    pub const DOUBLE: Self = Self::Primitive(PrimitiveType::Double);

    pub fn named(name: impl Into<String>) -> Self {
        Self::Class(ClassType::new(name))
    }

    pub fn array(self) -> Self {
        Self::Array {
            element: Box::new(self),
            dimension: ArrayDimension::default(),
        }
    }

    pub fn annotated(self, annotations: impl IntoIterator<Item = Annotation>) -> Self {
        match self {
            Self::Array {
                element,
                mut dimension,
            } => {
                dimension.annotations.extend(annotations);
                Self::Array { element, dimension }
            }
            Self::Annotated {
                annotations: mut existing,
                ty,
            } => {
                existing.extend(annotations);
                Self::Annotated {
                    annotations: existing,
                    ty,
                }
            }
            ty => Self::Annotated {
                annotations: annotations.into_iter().collect(),
                ty: Box::new(ty),
            },
        }
    }

    pub fn primitive(&self) -> Option<PrimitiveType> {
        match self {
            Self::Primitive(value) => Some(*value),
            Self::Annotated { ty, .. } => ty.primitive(),
            _ => None,
        }
    }
}

impl From<ClassType> for Type {
    fn from(value: ClassType) -> Self {
        Self::Class(value)
    }
}

/// A qualified first segment followed by explicitly known nested class segments.
/// `Outer$Inner` is preserved unless the caller supplies nesting information.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassType {
    pub name: String,
    pub arguments: Vec<TypeArgument>,
    pub nested: Vec<TypeSegment>,
}

impl ClassType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: Vec::new(),
            nested: Vec::new(),
        }
    }

    pub fn with_argument(mut self, argument: impl Into<TypeArgument>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn with_nested(mut self, segment: TypeSegment) -> Self {
        self.nested.push(segment);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSegment {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub arguments: Vec<TypeArgument>,
}

impl TypeSegment {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Vec::new(),
            name: name.into(),
            arguments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeArgument {
    Type(Type),
    Wildcard {
        annotations: Vec<Annotation>,
        bound: Option<WildcardBound>,
    },
}

impl TypeArgument {
    pub fn wildcard() -> Self {
        Self::Wildcard {
            annotations: Vec::new(),
            bound: None,
        }
    }
    pub fn extends(ty: Type) -> Self {
        Self::Wildcard {
            annotations: Vec::new(),
            bound: Some(WildcardBound::Extends(ty)),
        }
    }
    pub fn super_type(ty: Type) -> Self {
        Self::Wildcard {
            annotations: Vec::new(),
            bound: Some(WildcardBound::Super(ty)),
        }
    }
}

impl From<Type> for TypeArgument {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WildcardBound {
    Extends(Type),
    Super(Type),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrayDimension {
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub bounds: Vec<Type>,
}

impl TypeParameter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Vec::new(),
            name: name.into(),
            bounds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnType {
    Void,
    Type(Type),
}

impl From<Type> for ReturnType {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalType {
    Explicit(Type),
    Var,
}

impl From<Type> for LocalType {
    fn from(value: Type) -> Self {
        Self::Explicit(value)
    }
}
