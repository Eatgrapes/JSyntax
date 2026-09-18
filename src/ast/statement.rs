use super::{Annotation, Comment, Expr, LocalType, Parameter, Pattern, Type, TypeDecl, Variable};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn new(statements: impl IntoIterator<Item = Stmt>) -> Self {
        Self {
            statements: statements.into_iter().collect(),
        }
    }
    pub fn push(&mut self, statement: impl Into<Stmt>) {
        self.statements.push(statement.into());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Block),
    Empty,
    Expression(Expr),
    Local(LocalDecl),
    Type(Box<TypeDecl>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    DoWhile {
        body: Box<Stmt>,
        condition: Expr,
    },
    For {
        init: ForInit,
        condition: Option<Expr>,
        update: Vec<Expr>,
        body: Box<Stmt>,
    },
    EnhancedFor {
        binding: ForBinding,
        iterable: Expr,
        body: Box<Stmt>,
    },
    Switch(Switch),
    Try(TryStmt),
    Synchronized {
        monitor: Expr,
        body: Block,
    },
    Return(Option<Expr>),
    Throw(Expr),
    Break(Option<String>),
    Continue(Option<String>),
    Yield(Expr),
    /// Java 12 preview only. Distinct from a labeled break.
    BreakValue(Expr),
    Labeled {
        label: String,
        statement: Box<Stmt>,
    },
    Assert {
        condition: Expr,
        detail: Option<Expr>,
    },
    Commented {
        comments: Vec<Comment>,
        statement: Box<Stmt>,
    },
}

impl Stmt {
    pub fn return_value(value: Expr) -> Self {
        Self::Return(Some(value))
    }
    pub fn expression(value: Expr) -> Self {
        Self::Expression(value)
    }
    pub fn if_then(condition: Expr, then_branch: impl Into<Stmt>) -> Self {
        Self::If {
            condition,
            then_branch: Box::new(then_branch.into()),
            else_branch: None,
        }
    }
}

impl From<Block> for Stmt {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}
impl From<LocalDecl> for Stmt {
    fn from(value: LocalDecl) -> Self {
        Self::Local(value)
    }
}
impl From<TypeDecl> for Stmt {
    fn from(value: TypeDecl) -> Self {
        Self::Type(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDecl {
    pub annotations: Vec<Annotation>,
    pub final_: bool,
    pub ty: LocalType,
    pub variables: Vec<Variable>,
}

impl LocalDecl {
    pub fn initialized(
        ty: impl Into<LocalType>,
        name: impl Into<String>,
        value: impl Into<super::VariableInitializer>,
    ) -> Self {
        Self::new(ty, Variable::new(name).initialized(value))
    }
    pub fn new(ty: impl Into<LocalType>, variable: Variable) -> Self {
        Self {
            annotations: Vec::new(),
            final_: false,
            ty: ty.into(),
            variables: vec![variable],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Empty,
    Local(LocalDecl),
    Expressions(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForBinding {
    Variable(Parameter),
    Var {
        annotations: Vec<Annotation>,
        final_: bool,
        name: String,
    },
    Pattern(Pattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Switch {
    pub selector: Box<Expr>,
    pub body: SwitchBody,
}

impl Switch {
    pub fn rules(selector: Expr, rules: impl IntoIterator<Item = SwitchRule>) -> Self {
        Self {
            selector: Box::new(selector),
            body: SwitchBody::Rules(rules.into_iter().collect()),
        }
    }
    pub fn groups(selector: Expr, groups: impl IntoIterator<Item = SwitchGroup>) -> Self {
        Self {
            selector: Box::new(selector),
            body: SwitchBody::Groups(groups.into_iter().collect()),
        }
    }
}

impl From<Switch> for Expr {
    fn from(value: Switch) -> Self {
        Self::Switch(Box::new(value))
    }
}
impl From<Switch> for Stmt {
    fn from(value: Switch) -> Self {
        Self::Switch(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchBody {
    Groups(Vec<SwitchGroup>),
    Rules(Vec<SwitchRule>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchGroup {
    pub labels: Vec<SwitchLabel>,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchRule {
    pub label: SwitchLabel,
    pub body: SwitchRuleBody,
}

impl SwitchRule {
    pub fn new(label: SwitchLabel, body: impl Into<SwitchRuleBody>) -> Self {
        Self {
            label,
            body: body.into(),
        }
    }
}

impl From<Expr> for SwitchRuleBody {
    fn from(value: Expr) -> Self {
        Self::Expression(value)
    }
}
impl From<Block> for SwitchRuleBody {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchRuleBody {
    Expression(Expr),
    Block(Block),
    Throw(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchLabel {
    Default,
    Case {
        elements: Vec<CaseElement>,
        guard: Option<Box<Expr>>,
    },
}

impl SwitchLabel {
    pub fn constant(expression: Expr) -> Self {
        Self::Case {
            elements: vec![CaseElement::Constant(expression)],
            guard: None,
        }
    }
    pub fn pattern(pattern: Pattern) -> Self {
        Self::Case {
            elements: vec![CaseElement::Pattern(pattern)],
            guard: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseElement {
    Constant(Expr),
    Pattern(Pattern),
    Null,
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub resources: Vec<Resource>,
    pub body: Block,
    pub catches: Vec<Catch>,
    pub finally: Option<Block>,
}

impl TryStmt {
    pub fn new(body: Block) -> Self {
        Self {
            resources: Vec::new(),
            body,
            catches: Vec::new(),
            finally: None,
        }
    }
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }
    pub fn with_catch(mut self, catch: Catch) -> Self {
        self.catches.push(catch);
        self
    }
    pub fn with_finally(mut self, body: Block) -> Self {
        self.finally = Some(body);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource {
    Declaration(LocalDecl),
    Reference(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Catch {
    pub annotations: Vec<Annotation>,
    pub final_: bool,
    pub types: Vec<Type>,
    pub name: String,
    pub body: Block,
}
