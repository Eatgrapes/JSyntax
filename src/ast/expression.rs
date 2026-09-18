use super::{
    Annotation, ArrayDimension, AssignOp, BinaryOp, Block, ClassType, JavaString, Literal, Member,
    Parameter, Pattern, ReturnType, Switch, Type, UnaryOp,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Name(String),
    Literal(Literal),
    This(Option<String>),
    Super(Option<String>),
    ClassLiteral(ReturnType),
    Parenthesized(Box<Expr>),
    Field {
        target: Box<Expr>,
        name: String,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Call(CallExpr),
    New(NewExpr),
    NewArray(NewArrayExpr),
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        then_value: Box<Expr>,
        else_value: Box<Expr>,
    },
    Cast {
        types: Vec<Type>,
        value: Box<Expr>,
    },
    InstanceOf {
        value: Box<Expr>,
        test: InstanceOf,
    },
    Lambda {
        parameters: LambdaParameters,
        body: LambdaBody,
    },
    MethodReference {
        target: ReferenceTarget,
        type_arguments: Vec<Type>,
        member: ReferenceMember,
    },
    Switch(Box<Switch>),
    Template(StringTemplate),
}

impl Expr {
    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }
    pub const fn int(value: i32) -> Self {
        Self::Literal(Literal::Int(value))
    }
    pub const fn long(value: i64) -> Self {
        Self::Literal(Literal::Long(value))
    }
    pub const fn boolean(value: bool) -> Self {
        Self::Literal(Literal::Boolean(value))
    }
    pub fn string(value: impl Into<JavaString>) -> Self {
        Self::Literal(Literal::String(value.into()))
    }
    pub fn field(self, name: impl Into<String>) -> Self {
        Self::Field {
            target: Box::new(self),
            name: name.into(),
        }
    }
    pub fn call(self, name: impl Into<String>, arguments: impl IntoIterator<Item = Expr>) -> Self {
        Self::Call(CallExpr {
            target: Some(Box::new(self)),
            name: name.into(),
            type_arguments: Vec::new(),
            arguments: arguments.into_iter().collect(),
        })
    }
    pub fn invoke(name: impl Into<String>, arguments: impl IntoIterator<Item = Expr>) -> Self {
        Self::Call(CallExpr {
            target: None,
            name: name.into(),
            type_arguments: Vec::new(),
            arguments: arguments.into_iter().collect(),
        })
    }
    pub fn binary(self, op: BinaryOp, right: Expr) -> Self {
        Self::Binary {
            left: Box::new(self),
            op,
            right: Box::new(right),
        }
    }
    pub fn unary(self, op: UnaryOp) -> Self {
        Self::Unary {
            op,
            operand: Box::new(self),
        }
    }
    pub fn assign(self, value: Expr) -> Self {
        Self::Assign {
            target: Box::new(self),
            op: AssignOp::Assign,
            value: Box::new(value),
        }
    }
    pub fn index(self, index: Expr) -> Self {
        Self::ArrayAccess {
            array: Box::new(self),
            index: Box::new(index),
        }
    }
    pub fn cast(self, ty: Type) -> Self {
        Self::Cast {
            types: vec![ty],
            value: Box::new(self),
        }
    }
    pub fn parenthesized(self) -> Self {
        Self::Parenthesized(Box::new(self))
    }
    pub fn conditional(self, then_value: Expr, else_value: Expr) -> Self {
        Self::Conditional {
            condition: Box::new(self),
            then_value: Box::new(then_value),
            else_value: Box::new(else_value),
        }
    }
    pub fn instance_of(self, ty: Type) -> Self {
        Self::InstanceOf {
            value: Box::new(self),
            test: InstanceOf::Type(ty),
        }
    }
    pub fn matches(self, pattern: Pattern) -> Self {
        Self::InstanceOf {
            value: Box::new(self),
            test: InstanceOf::Pattern(Box::new(pattern)),
        }
    }
    pub fn new_object(ty: ClassType, arguments: impl IntoIterator<Item = Expr>) -> Self {
        Self::New(NewExpr {
            qualifier: None,
            constructor_type_arguments: Vec::new(),
            annotations: Vec::new(),
            ty,
            diamond: false,
            arguments: arguments.into_iter().collect(),
            body: None,
        })
    }
    pub fn is_statement_expression(&self) -> bool {
        matches!(self, Self::Assign { .. } | Self::Call(_) | Self::New(_))
            || matches!(self, Self::Unary { op, .. } if op.is_update())
    }
}

impl From<Literal> for Expr {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub target: Option<Box<Expr>>,
    pub name: String,
    pub type_arguments: Vec<Type>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    pub qualifier: Option<Box<Expr>>,
    pub constructor_type_arguments: Vec<Type>,
    pub annotations: Vec<Annotation>,
    pub ty: ClassType,
    pub diamond: bool,
    pub arguments: Vec<Expr>,
    pub body: Option<Vec<Member>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewArrayExpr {
    pub element: Type,
    pub dimensions: Vec<ArrayLength>,
    pub initializer: Option<ArrayInitializer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLength {
    pub dimension: ArrayDimension,
    pub length: Option<Expr>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrayInitializer {
    pub elements: Vec<VariableInitializer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableInitializer {
    Expression(Expr),
    Array(ArrayInitializer),
}

impl From<Expr> for VariableInitializer {
    fn from(value: Expr) -> Self {
        Self::Expression(value)
    }
}
impl From<ArrayInitializer> for VariableInitializer {
    fn from(value: ArrayInitializer) -> Self {
        Self::Array(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceOf {
    Type(Type),
    Pattern(Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaParameters {
    Inferred(Vec<String>),
    Explicit(Vec<Parameter>),
    Var(Vec<VarLambdaParameter>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarLambdaParameter {
    pub annotations: Vec<Annotation>,
    pub final_: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    Expression(Box<Expr>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceTarget {
    Expression(Box<Expr>),
    Type(Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceMember {
    Method(String),
    Constructor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringTemplate {
    pub processor: Box<Expr>,
    pub multiline: bool,
    pub head: JavaString,
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplatePart {
    pub expression: Expr,
    pub tail: JavaString,
}
