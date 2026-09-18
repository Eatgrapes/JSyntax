use super::{Annotation, Comment, Member, TypeDecl};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompilationUnit {
    pub comments: Vec<Comment>,
    pub package: Option<PackageDecl>,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDecl>,
}

impl CompilationUnit {
    pub fn new(types: impl IntoIterator<Item = TypeDecl>) -> Self {
        Self {
            types: types.into_iter().collect(),
            ..Self::default()
        }
    }
    pub fn with_package(mut self, name: impl Into<String>) -> Self {
        self.package = Some(PackageDecl {
            annotations: Vec::new(),
            name: name.into(),
        });
        self
    }
    pub fn with_import(mut self, import: Import) -> Self {
        self.imports.push(import);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Import {
    Type(String),
    OnDemand(String),
    Static {
        owner: String,
        member: Option<String>,
    },
    Module(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompactUnit {
    pub comments: Vec<Comment>,
    pub imports: Vec<Import>,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleUnit {
    pub imports: Vec<Import>,
    pub declaration: ModuleDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub comments: Vec<Comment>,
    pub annotations: Vec<Annotation>,
    pub open: bool,
    pub name: String,
    pub directives: Vec<ModuleDirective>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDirective {
    Requires {
        name: String,
        static_: bool,
        transitive: bool,
    },
    Exports {
        package: String,
        to: Vec<String>,
    },
    Opens {
        package: String,
        to: Vec<String>,
    },
    Uses(String),
    Provides {
        service: String,
        implementations: Vec<String>,
    },
}
