use super::{
    Annotation, AnnotationValue, ArrayDimension, Block, Comment, Expr, ReturnType, Type,
    TypeParameter, VariableInitializer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Public,
    Protected,
    Private,
    Abstract,
    Static,
    Final,
    Transient,
    Volatile,
    Synchronized,
    Native,
    Strictfp,
    Default,
    Sealed,
    NonSealed,
}

impl Modifier {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Protected => "protected",
            Self::Private => "private",
            Self::Abstract => "abstract",
            Self::Static => "static",
            Self::Final => "final",
            Self::Transient => "transient",
            Self::Volatile => "volatile",
            Self::Synchronized => "synchronized",
            Self::Native => "native",
            Self::Strictfp => "strictfp",
            Self::Default => "default",
            Self::Sealed => "sealed",
            Self::NonSealed => "non-sealed",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeclarationMeta {
    pub comments: Vec<Comment>,
    pub annotations: Vec<Annotation>,
    pub modifiers: Vec<Modifier>,
}

pub trait Decorated: Sized {
    fn meta_mut(&mut self) -> &mut DeclarationMeta;
    fn with_modifier(mut self, modifier: Modifier) -> Self {
        self.meta_mut().modifiers.push(modifier);
        self
    }
    fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.meta_mut().annotations.push(annotation);
        self
    }
    fn with_comment(mut self, comment: Comment) -> Self {
        self.meta_mut().comments.push(comment);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub meta: DeclarationMeta,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub kind: TypeDeclKind,
    pub members: Vec<Member>,
}

impl TypeDecl {
    pub fn new(name: impl Into<String>, kind: TypeDeclKind) -> Self {
        Self {
            meta: DeclarationMeta::default(),
            name: name.into(),
            type_parameters: Vec::new(),
            kind,
            members: Vec::new(),
        }
    }
    pub fn class(name: impl Into<String>) -> Self {
        Self::new(
            name,
            TypeDeclKind::Class {
                extends: None,
                implements: Vec::new(),
                permits: Vec::new(),
            },
        )
    }
    pub fn interface(name: impl Into<String>) -> Self {
        Self::new(
            name,
            TypeDeclKind::Interface {
                extends: Vec::new(),
                permits: Vec::new(),
            },
        )
    }
    pub fn record(
        name: impl Into<String>,
        components: impl IntoIterator<Item = Parameter>,
    ) -> Self {
        Self::new(
            name,
            TypeDeclKind::Record {
                components: components.into_iter().collect(),
                implements: Vec::new(),
            },
        )
    }
    pub fn enumeration(
        name: impl Into<String>,
        constants: impl IntoIterator<Item = EnumConstant>,
    ) -> Self {
        Self::new(
            name,
            TypeDeclKind::Enum {
                constants: constants.into_iter().collect(),
                implements: Vec::new(),
            },
        )
    }
    pub fn annotation(name: impl Into<String>) -> Self {
        Self::new(name, TypeDeclKind::Annotation)
    }
    pub fn with_member(mut self, member: impl Into<Member>) -> Self {
        self.members.push(member.into());
        self
    }
    pub fn with_type_parameter(mut self, parameter: TypeParameter) -> Self {
        self.type_parameters.push(parameter);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclKind {
    Class {
        extends: Option<Type>,
        implements: Vec<Type>,
        permits: Vec<Type>,
    },
    Interface {
        extends: Vec<Type>,
        permits: Vec<Type>,
    },
    Enum {
        constants: Vec<EnumConstant>,
        implements: Vec<Type>,
    },
    Record {
        components: Vec<Parameter>,
        implements: Vec<Type>,
    },
    Annotation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Field(FieldDecl),
    Method(MethodDecl),
    Constructor(ConstructorDecl),
    Type(Box<TypeDecl>),
    Initializer { static_: bool, body: Block },
    AnnotationElement(AnnotationElement),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub meta: DeclarationMeta,
    pub ty: Type,
    pub variables: Vec<Variable>,
}

impl FieldDecl {
    pub fn new(ty: Type, name: impl Into<String>) -> Self {
        Self {
            meta: DeclarationMeta::default(),
            ty,
            variables: vec![Variable::new(name)],
        }
    }
    pub fn initialized(
        ty: Type,
        name: impl Into<String>,
        initializer: impl Into<VariableInitializer>,
    ) -> Self {
        Self {
            meta: DeclarationMeta::default(),
            ty,
            variables: vec![Variable::new(name).initialized(initializer)],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub dimensions: Vec<ArrayDimension>,
    pub initializer: Option<VariableInitializer>,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dimensions: Vec::new(),
            initializer: None,
        }
    }
    pub fn initialized(mut self, initializer: impl Into<VariableInitializer>) -> Self {
        self.initializer = Some(initializer.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub annotations: Vec<Annotation>,
    pub final_: bool,
    pub ty: Type,
    pub name: String,
    pub dimensions: Vec<ArrayDimension>,
    /// An empty annotation list still represents an ordinary varargs ellipsis.
    pub varargs: Option<Vec<Annotation>>,
}

impl Parameter {
    pub fn new(ty: Type, name: impl Into<String>) -> Self {
        Self {
            annotations: Vec::new(),
            final_: false,
            ty,
            name: name.into(),
            dimensions: Vec::new(),
            varargs: None,
        }
    }
    pub fn varargs(mut self) -> Self {
        self.varargs = Some(Vec::new());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiverParameter {
    pub annotations: Vec<Annotation>,
    pub ty: Type,
    pub qualifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub meta: DeclarationMeta,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type: ReturnType,
    pub name: String,
    pub receiver: Option<ReceiverParameter>,
    pub parameters: Vec<Parameter>,
    pub return_dimensions: Vec<ArrayDimension>,
    pub throws: Vec<Type>,
    pub body: Option<Block>,
}

impl MethodDecl {
    pub fn new(name: impl Into<String>, return_type: impl Into<ReturnType>) -> Self {
        Self {
            meta: DeclarationMeta::default(),
            type_parameters: Vec::new(),
            return_type: return_type.into(),
            name: name.into(),
            receiver: None,
            parameters: Vec::new(),
            return_dimensions: Vec::new(),
            throws: Vec::new(),
            body: None,
        }
    }
    pub fn with_parameter(mut self, parameter: Parameter) -> Self {
        self.parameters.push(parameter);
        self
    }
    pub fn with_body(mut self, body: Block) -> Self {
        self.body = Some(body);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorDecl {
    pub meta: DeclarationMeta,
    pub type_parameters: Vec<TypeParameter>,
    pub name: String,
    pub parameters: ConstructorParameters,
    pub throws: Vec<Type>,
    pub body: ConstructorBody,
}

impl ConstructorDecl {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            meta: DeclarationMeta::default(),
            type_parameters: Vec::new(),
            name: name.into(),
            parameters: ConstructorParameters::Explicit {
                receiver: None,
                parameters: Vec::new(),
            },
            throws: Vec::new(),
            body: ConstructorBody::default(),
        }
    }
    pub fn compact(name: impl Into<String>, body: Block) -> Self {
        Self {
            parameters: ConstructorParameters::Compact,
            body: ConstructorBody {
                prologue: Block::default(),
                invocation: None,
                epilogue: body,
            },
            ..Self::new(name)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstructorParameters {
    Explicit {
        receiver: Option<ReceiverParameter>,
        parameters: Vec<Parameter>,
    },
    Compact,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConstructorBody {
    pub prologue: Block,
    pub invocation: Option<ConstructorInvocation>,
    pub epilogue: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorInvocation {
    pub target: ConstructorTarget,
    pub type_arguments: Vec<Type>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstructorTarget {
    This,
    Super(Option<Box<Expr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumConstant {
    pub comments: Vec<Comment>,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub arguments: Vec<Expr>,
    pub body: Option<Vec<Member>>,
}

impl EnumConstant {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            comments: Vec::new(),
            annotations: Vec::new(),
            name: name.into(),
            arguments: Vec::new(),
            body: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationElement {
    pub meta: DeclarationMeta,
    pub ty: Type,
    pub name: String,
    pub dimensions: Vec<ArrayDimension>,
    pub default: Option<AnnotationValue>,
}

macro_rules! decorated {
    ($($ty:ty),* $(,)?) => { $(impl Decorated for $ty { fn meta_mut(&mut self) -> &mut DeclarationMeta { &mut self.meta } })* };
}
decorated!(
    TypeDecl,
    FieldDecl,
    MethodDecl,
    ConstructorDecl,
    AnnotationElement
);

macro_rules! member_from {
    ($($ty:ty => $variant:ident),* $(,)?) => { $(impl From<$ty> for Member { fn from(value: $ty) -> Self { Self::$variant(value) } })* };
}
member_from!(FieldDecl => Field, MethodDecl => Method, ConstructorDecl => Constructor, AnnotationElement => AnnotationElement);
impl From<TypeDecl> for Member {
    fn from(value: TypeDecl) -> Self {
        Self::Type(Box::new(value))
    }
}
