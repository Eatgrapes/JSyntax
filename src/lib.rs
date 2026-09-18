//! Construct and transform Java syntax, then emit source for an explicit language level.
//!
//! ```
//! use jsyntax::prelude::*;
//!
//! let method = MethodDecl::new("answer", Type::INT)
//!     .with_modifier(Modifier::Public)
//!     .with_modifier(Modifier::Static)
//!     .with_body(Block::new([Stmt::return_value(Expr::int(42))]));
//! let class = TypeDecl::class("Answer")
//!     .with_modifier(Modifier::Public)
//!     .with_member(method);
//! let source = CompilationUnit::new([class]).to_java()?;
//! assert!(source.contains("return 42;"));
//! # Ok::<(), jsyntax::EmitError>(())
//! ```
//!
//! AST fields are intentionally editable for decompiler passes. Emission checks
//! lexical validity, structural constraints and syntax feature availability; it
//! is not a Java type checker, name resolver, source parser or bytecode reader.
//! Callers retain responsibility for symbol resolution, flow analysis and
//! semantic restrictions (including type-dependent language-version rules).
//! Names use Java source spelling, not JVM descriptors. No implicit imports,
//! renaming, desugaring or binary-name-to-nested-name rewriting is performed.
//! Identifier character categories follow the bundled Unicode data, rather than
//! reproducing every historical JDK's Unicode tables. Literals are emitted from
//! values with canonical spelling; this is not a lossless source round-tripper.
//!
//! Set [`PrintOptions::language`] to select a target (the default is Java 27
//! without preview). Unsupported syntax returns [`EmitError`], never an implicit
//! downgrade. Java 12 value-breaks and withdrawn preview forms are distinct nodes.
//! Use [`visit::Visit`] for analysis and [`visit::VisitMut`] for rewriting.
//! Enable the `ferro-jtype` feature for descriptor and inferred-type adapters.

pub mod ast;
pub mod emit;
pub mod version;
pub mod visit;

#[cfg(feature = "ferro-jtype")]
pub mod ferro;

pub use emit::{EmitError, ErrorKind, JavaSource, LineEnding, PrintOptions};
pub use version::{Feature, FeatureStatus, JavaVersion, LanguageLevel};

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::{JavaSource, JavaVersion, LanguageLevel, PrintOptions};
}
