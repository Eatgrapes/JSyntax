//! Checked Java source emission. No partial source is returned on an error.

mod declaration;
mod expression;
mod lexical;
mod statement;
mod types;
mod unit;

use crate::ast::*;
use crate::version::{Feature, LanguageLevel};
use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineEnding {
    #[default]
    Lf,
    CrLf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintOptions {
    pub language: LanguageLevel,
    pub indent: String,
    pub line_ending: LineEnding,
}

impl PrintOptions {
    pub fn new(language: LanguageLevel) -> Self {
        Self {
            language,
            ..Self::default()
        }
    }
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            language: LanguageLevel::default(),
            indent: "    ".into(),
            line_ending: LineEnding::Lf,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError {
    pub kind: ErrorKind,
    /// Innermost declaration names first, useful when reporting an invalid decompiler node.
    pub context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    FeatureUnavailable {
        feature: Feature,
        language: LanguageLevel,
    },
    InvalidIdentifier(String),
    InvalidStructure(&'static str),
    InvalidComment,
    InvalidIndent,
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::FeatureUnavailable { feature, language } => write!(
                f,
                "{feature:?} is unavailable in Java {}{}",
                language.version,
                if language.preview {
                    " with preview enabled"
                } else {
                    ""
                }
            )?,
            ErrorKind::InvalidIdentifier(name) => write!(f, "invalid Java identifier: {name:?}")?,
            ErrorKind::InvalidStructure(reason) => f.write_str(reason)?,
            ErrorKind::InvalidComment => {
                f.write_str("comment contains a terminator or a Unicode escape")?
            }
            ErrorKind::InvalidIndent => {
                f.write_str("indentation must contain only spaces or tabs")?
            }
        }
        for name in &self.context {
            write!(f, " in {name}")?;
        }
        Ok(())
    }
}

impl std::error::Error for EmitError {}

pub trait JavaSource: private::Sealed {
    fn to_java(&self) -> Result<String> {
        self.to_java_with(&PrintOptions::default())
    }
    fn to_java_with(&self, options: &PrintOptions) -> Result<String>;
}

mod private {
    pub trait Sealed {}
}

type Result<T = ()> = std::result::Result<T, EmitError>;

struct Printer<'a> {
    options: &'a PrintOptions,
    output: String,
    depth: usize,
    line_start: bool,
    switch_expressions: usize,
    legacy_switches: usize,
}

impl<'a> Printer<'a> {
    fn new(options: &'a PrintOptions) -> Result<Self> {
        if options.indent.chars().any(|ch| !matches!(ch, ' ' | '\t')) {
            return Err(Self::error(ErrorKind::InvalidIndent));
        }
        Ok(Self {
            options,
            output: String::new(),
            depth: 0,
            line_start: true,
            switch_expressions: 0,
            legacy_switches: 0,
        })
    }
    fn error(kind: ErrorKind) -> EmitError {
        EmitError {
            kind,
            context: Vec::new(),
        }
    }
    fn invalid(reason: &'static str) -> EmitError {
        Self::error(ErrorKind::InvalidStructure(reason))
    }
    fn require(&self, feature: Feature) -> Result {
        if self.options.language.supports(feature) {
            Ok(())
        } else {
            Err(Self::error(ErrorKind::FeatureUnavailable {
                feature,
                language: self.options.language,
            }))
        }
    }
    fn text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.line_start {
            for _ in 0..self.depth {
                self.output.push_str(&self.options.indent);
            }
            self.line_start = false;
        }
        self.output.push_str(text);
    }
    fn newline(&mut self) {
        self.output.push_str(match self.options.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        });
        self.line_start = true;
    }
    fn open_block(&mut self) {
        self.text("{");
        self.newline();
        self.depth += 1;
    }
    fn close_block(&mut self) {
        self.depth -= 1;
        self.text("}");
    }
    fn separated<T>(
        &mut self,
        values: &[T],
        separator: &str,
        mut emit: impl FnMut(&mut Self, &T) -> Result,
    ) -> Result {
        for (index, value) in values.iter().enumerate() {
            if index != 0 {
                self.text(separator);
            }
            emit(self, value)?;
        }
        Ok(())
    }
}

macro_rules! source {
    ($($ty:ty => $method:ident),* $(,)?) => { $(
        impl private::Sealed for $ty {}
        impl JavaSource for $ty {
            fn to_java_with(&self, options: &PrintOptions) -> std::result::Result<String, EmitError> {
                let mut printer = Printer::new(options)?;
                printer.$method(self)?;
                Ok(printer.output)
            }
        }
    )* };
}

source!(
    CompilationUnit => compilation_unit, CompactUnit => compact_unit, ModuleUnit => module_unit,
    ModuleDecl => module, TypeDecl => type_decl, Member => standalone_member,
    FieldDecl => field, MethodDecl => method, ConstructorDecl => constructor,
    AnnotationElement => annotation_element, Expr => expression, Stmt => statement,
    Block => block, Type => ty, ClassType => class_type, Annotation => annotation,
    Pattern => pattern, Literal => literal,
);
