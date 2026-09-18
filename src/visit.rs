//! Borrowed and mutable depth-first visitors. Override a hook and call its matching
//! `walk`/`walk_mut` function to recurse. Mutable hooks may replace complete nodes.
//! Names are syntax strings, not resolved symbols; a rename pass must track scopes.
//!
//! ```
//! use jsyntax::{ast::*, visit::{VisitMut, walk_mut}};
//! struct Increment;
//! impl VisitMut for Increment {
//!     fn visit_expr(&mut self, node: &mut Expr) {
//!         if let Expr::Literal(Literal::Int(value)) = node { *value += 1; }
//!         walk_mut::expr(self, node);
//!     }
//! }
//! let mut node = Expr::int(41);
//! Increment.visit_expr(&mut node);
//! assert_eq!(node, Expr::int(42));
//! ```

use crate::ast::*;

macro_rules! visitors {
    ($trait:ident, $walk:ident, [$($m:tt)*]) => {
        pub trait $trait {
            fn visit_unit(&mut self, node: &$($m)* CompilationUnit) { $walk::unit(self, node); }
            fn visit_compact_unit(&mut self, node: &$($m)* CompactUnit) { $walk::compact_unit(self, node); }
            fn visit_module_unit(&mut self, node: &$($m)* ModuleUnit) { $walk::module_unit(self, node); }
            fn visit_module(&mut self, node: &$($m)* ModuleDecl) { $walk::module(self, node); }
            fn visit_import(&mut self, node: &$($m)* Import) { $walk::import(self, node); }
            fn visit_type_decl(&mut self, node: &$($m)* TypeDecl) { $walk::type_decl(self, node); }
            fn visit_member(&mut self, node: &$($m)* Member) { $walk::member(self, node); }
            fn visit_method(&mut self, node: &$($m)* MethodDecl) { $walk::method(self, node); }
            fn visit_constructor(&mut self, node: &$($m)* ConstructorDecl) { $walk::constructor(self, node); }
            fn visit_parameter(&mut self, node: &$($m)* Parameter) { $walk::parameter(self, node); }
            fn visit_annotation(&mut self, node: &$($m)* Annotation) { $walk::annotation(self, node); }
            fn visit_annotation_value(&mut self, node: &$($m)* AnnotationValue) { $walk::annotation_value(self, node); }
            fn visit_type(&mut self, node: &$($m)* Type) { $walk::ty(self, node); }
            fn visit_class_type(&mut self, node: &$($m)* ClassType) { $walk::class_type(self, node); }
            fn visit_type_argument(&mut self, node: &$($m)* TypeArgument) { $walk::type_argument(self, node); }
            fn visit_type_parameter(&mut self, node: &$($m)* TypeParameter) { $walk::type_parameter(self, node); }
            fn visit_variable(&mut self, node: &$($m)* Variable) { $walk::variable(self, node); }
            fn visit_initializer(&mut self, node: &$($m)* VariableInitializer) { $walk::initializer(self, node); }
            fn visit_expr(&mut self, node: &$($m)* Expr) { $walk::expr(self, node); }
            fn visit_stmt(&mut self, node: &$($m)* Stmt) { $walk::stmt(self, node); }
            fn visit_block(&mut self, node: &$($m)* Block) { $walk::block(self, node); }
            fn visit_local(&mut self, node: &$($m)* LocalDecl) { $walk::local(self, node); }
            fn visit_pattern(&mut self, node: &$($m)* Pattern) { $walk::pattern(self, node); }
            fn visit_switch(&mut self, node: &$($m)* Switch) { $walk::switch(self, node); }
            fn visit_name(&mut self, _name: &$($m)* String) {}
            fn visit_literal(&mut self, literal: &$($m)* Literal) {
                if let Literal::String(value) | Literal::TextBlock(value) = literal { self.visit_string(value); }
            }
            fn visit_string(&mut self, _value: &$($m)* JavaString) {}
            fn visit_comment(&mut self, _comment: &$($m)* Comment) {}
        }

        pub mod $walk {
            use super::*;

            pub fn unit<V: $trait + ?Sized>(v: &mut V, n: &$($m)* CompilationUnit) {
                for c in &$($m)* n.comments { v.visit_comment(c); }
                if let Some(p) = &$($m)* n.package { annotations(v, &$($m)* p.annotations); v.visit_name(&$($m)* p.name); }
                for i in &$($m)* n.imports { v.visit_import(i); }
                for t in &$($m)* n.types { v.visit_type_decl(t); }
            }
            pub fn compact_unit<V: $trait + ?Sized>(v: &mut V, n: &$($m)* CompactUnit) {
                for c in &$($m)* n.comments { v.visit_comment(c); }
                for i in &$($m)* n.imports { v.visit_import(i); }
                members(v, &$($m)* n.members);
            }
            pub fn module_unit<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ModuleUnit) {
                for i in &$($m)* n.imports { v.visit_import(i); } v.visit_module(&$($m)* n.declaration);
            }
            pub fn module<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ModuleDecl) {
                for c in &$($m)* n.comments { v.visit_comment(c); }
                annotations(v, &$($m)* n.annotations); v.visit_name(&$($m)* n.name);
                for directive in &$($m)* n.directives {
                    match directive {
                        ModuleDirective::Requires { name, .. } | ModuleDirective::Uses(name) => v.visit_name(name),
                        ModuleDirective::Exports { package, to } | ModuleDirective::Opens { package, to } => { v.visit_name(package); for name in to { v.visit_name(name); } }
                        ModuleDirective::Provides { service, implementations } => { v.visit_name(service); for name in implementations { v.visit_name(name); } }
                    }
                }
            }
            pub fn import<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Import) {
                match n {
                    Import::Type(name) | Import::OnDemand(name) | Import::Module(name) => v.visit_name(name),
                    Import::Static { owner, member } => { v.visit_name(owner); if let Some(name) = member { v.visit_name(name); } }
                }
            }
            pub fn metadata<V: $trait + ?Sized>(v: &mut V, n: &$($m)* DeclarationMeta) {
                for c in &$($m)* n.comments { v.visit_comment(c); } annotations(v, &$($m)* n.annotations);
            }
            pub fn type_decl<V: $trait + ?Sized>(v: &mut V, n: &$($m)* TypeDecl) {
                metadata(v, &$($m)* n.meta); v.visit_name(&$($m)* n.name);
                for t in &$($m)* n.type_parameters { v.visit_type_parameter(t); }
                match &$($m)* n.kind {
                    TypeDeclKind::Class { extends, implements, permits } => {
                        if let Some(t) = extends { v.visit_type(t); } types(v, implements); types(v, permits);
                    }
                    TypeDeclKind::Interface { extends, permits } => { types(v, extends); types(v, permits); }
                    TypeDeclKind::Enum { constants, implements } => {
                        for c in constants {
                            for comment in &$($m)* c.comments { v.visit_comment(comment); }
                            annotations(v, &$($m)* c.annotations); v.visit_name(&$($m)* c.name); expressions(v, &$($m)* c.arguments);
                            if let Some(body) = &$($m)* c.body { members(v, body); }
                        }
                        types(v, implements);
                    }
                    TypeDeclKind::Record { components, implements } => { for p in components { v.visit_parameter(p); } types(v, implements); }
                    TypeDeclKind::Annotation => (),
                }
                members(v, &$($m)* n.members);
            }
            pub fn members<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [Member]) { for m in n { v.visit_member(m); } }
            pub fn member<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Member) {
                match n {
                    Member::Field(f) => { metadata(v, &$($m)* f.meta); v.visit_type(&$($m)* f.ty); for var in &$($m)* f.variables { v.visit_variable(var); } }
                    Member::Method(m) => v.visit_method(m),
                    Member::Constructor(c) => v.visit_constructor(c),
                    Member::Type(t) => v.visit_type_decl(t),
                    Member::Initializer { body, .. } => v.visit_block(body),
                    Member::AnnotationElement(e) => {
                        metadata(v, &$($m)* e.meta); v.visit_type(&$($m)* e.ty); v.visit_name(&$($m)* e.name);
                        dimensions(v, &$($m)* e.dimensions); if let Some(value) = &$($m)* e.default { v.visit_annotation_value(value); }
                    }
                    Member::Empty => (),
                }
            }
            pub fn method<V: $trait + ?Sized>(v: &mut V, n: &$($m)* MethodDecl) {
                metadata(v, &$($m)* n.meta); for t in &$($m)* n.type_parameters { v.visit_type_parameter(t); }
                return_type(v, &$($m)* n.return_type); v.visit_name(&$($m)* n.name);
                if let Some(r) = &$($m)* n.receiver { receiver(v, r); }
                for p in &$($m)* n.parameters { v.visit_parameter(p); }
                dimensions(v, &$($m)* n.return_dimensions); types(v, &$($m)* n.throws);
                if let Some(b) = &$($m)* n.body { v.visit_block(b); }
            }
            pub fn constructor<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ConstructorDecl) {
                metadata(v, &$($m)* n.meta); for t in &$($m)* n.type_parameters { v.visit_type_parameter(t); }
                v.visit_name(&$($m)* n.name);
                if let ConstructorParameters::Explicit { receiver: r, parameters } = &$($m)* n.parameters {
                    if let Some(r) = r { receiver(v, r); } for p in parameters { v.visit_parameter(p); }
                }
                types(v, &$($m)* n.throws); v.visit_block(&$($m)* n.body.prologue);
                if let Some(i) = &$($m)* n.body.invocation {
                    if let ConstructorTarget::Super(Some(e)) = &$($m)* i.target { v.visit_expr(e); }
                    types(v, &$($m)* i.type_arguments); expressions(v, &$($m)* i.arguments);
                }
                v.visit_block(&$($m)* n.body.epilogue);
            }
            pub fn parameter<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Parameter) {
                annotations(v, &$($m)* n.annotations); v.visit_type(&$($m)* n.ty); v.visit_name(&$($m)* n.name);
                dimensions(v, &$($m)* n.dimensions); if let Some(a) = &$($m)* n.varargs { annotations(v, a); }
            }
            pub fn receiver<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ReceiverParameter) {
                annotations(v, &$($m)* n.annotations); v.visit_type(&$($m)* n.ty); if let Some(name) = &$($m)* n.qualifier { v.visit_name(name); }
            }
            pub fn annotations<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [Annotation]) { for a in n { v.visit_annotation(a); } }
            pub fn annotation<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Annotation) {
                v.visit_name(&$($m)* n.name);
                match &$($m)* n.arguments {
                    AnnotationArguments::Marker => (),
                    AnnotationArguments::Single(value) => v.visit_annotation_value(value),
                    AnnotationArguments::Named(values) => { for (name, value) in values { v.visit_name(name); v.visit_annotation_value(value); } }
                }
            }
            pub fn annotation_value<V: $trait + ?Sized>(v: &mut V, n: &$($m)* AnnotationValue) {
                match n { AnnotationValue::Expression(e) => v.visit_expr(e), AnnotationValue::Annotation(a) => v.visit_annotation(a), AnnotationValue::Array(values) => for value in values { v.visit_annotation_value(value); } }
            }
            pub fn ty<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Type) {
                match n {
                    Type::Primitive(_) => (), Type::Class(c) => v.visit_class_type(c),
                    Type::Array { element, dimension } => { v.visit_type(element); annotations(v, &$($m)* dimension.annotations); }
                    Type::Annotated { annotations: a, ty } => { annotations(v, a); v.visit_type(ty); }
                }
            }
            pub fn types<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [Type]) { for t in n { v.visit_type(t); } }
            pub fn return_type<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ReturnType) { if let ReturnType::Type(t) = n { v.visit_type(t); } }
            pub fn class_type<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ClassType) {
                v.visit_name(&$($m)* n.name); for a in &$($m)* n.arguments { v.visit_type_argument(a); }
                for s in &$($m)* n.nested { annotations(v, &$($m)* s.annotations); v.visit_name(&$($m)* s.name); for a in &$($m)* s.arguments { v.visit_type_argument(a); } }
            }
            pub fn type_argument<V: $trait + ?Sized>(v: &mut V, n: &$($m)* TypeArgument) {
                match n {
                    TypeArgument::Type(t) => v.visit_type(t),
                    TypeArgument::Wildcard { annotations: a, bound } => {
                        annotations(v, a); if let Some(WildcardBound::Extends(t) | WildcardBound::Super(t)) = bound { v.visit_type(t); }
                    }
                }
            }
            pub fn type_parameter<V: $trait + ?Sized>(v: &mut V, n: &$($m)* TypeParameter) {
                annotations(v, &$($m)* n.annotations); v.visit_name(&$($m)* n.name); types(v, &$($m)* n.bounds);
            }
            pub fn dimensions<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [ArrayDimension]) { for d in n { annotations(v, &$($m)* d.annotations); } }
            pub fn variable<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Variable) {
                v.visit_name(&$($m)* n.name); dimensions(v, &$($m)* n.dimensions); if let Some(i) = &$($m)* n.initializer { v.visit_initializer(i); }
            }
            pub fn initializer<V: $trait + ?Sized>(v: &mut V, n: &$($m)* VariableInitializer) {
                match n { VariableInitializer::Expression(e) => v.visit_expr(e), VariableInitializer::Array(a) => array_initializer(v, a) }
            }
            pub fn array_initializer<V: $trait + ?Sized>(v: &mut V, n: &$($m)* ArrayInitializer) { for i in &$($m)* n.elements { v.visit_initializer(i); } }
            pub fn expressions<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [Expr]) { for e in n { v.visit_expr(e); } }
            pub fn expr<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Expr) {
                match n {
                    Expr::Name(name) => v.visit_name(name), Expr::Literal(l) => v.visit_literal(l),
                    Expr::This(name) | Expr::Super(name) => if let Some(name) = name { v.visit_name(name); },
                    Expr::ClassLiteral(t) => return_type(v, t),
                    Expr::Parenthesized(e) | Expr::Unary { operand: e, .. } => v.visit_expr(e),
                    Expr::Field { target, name } => { v.visit_expr(target); v.visit_name(name); }
                    Expr::ArrayAccess { array, index } => { v.visit_expr(array); v.visit_expr(index); }
                    Expr::Call(c) => {
                        if let Some(t) = &$($m)* c.target { v.visit_expr(t); } v.visit_name(&$($m)* c.name);
                        types(v, &$($m)* c.type_arguments); expressions(v, &$($m)* c.arguments);
                    }
                    Expr::New(n) => {
                        if let Some(e) = &$($m)* n.qualifier { v.visit_expr(e); }
                        types(v, &$($m)* n.constructor_type_arguments); annotations(v, &$($m)* n.annotations); v.visit_class_type(&$($m)* n.ty);
                        expressions(v, &$($m)* n.arguments); if let Some(body) = &$($m)* n.body { members(v, body); }
                    }
                    Expr::NewArray(n) => {
                        v.visit_type(&$($m)* n.element);
                        for d in &$($m)* n.dimensions { annotations(v, &$($m)* d.dimension.annotations); if let Some(e) = &$($m)* d.length { v.visit_expr(e); } }
                        if let Some(i) = &$($m)* n.initializer { array_initializer(v, i); }
                    }
                    Expr::Binary { left, right, .. } | Expr::Assign { target: left, value: right, .. } => { v.visit_expr(left); v.visit_expr(right); }
                    Expr::Conditional { condition, then_value, else_value } => { v.visit_expr(condition); v.visit_expr(then_value); v.visit_expr(else_value); }
                    Expr::Cast { types: t, value } => { types(v, t); v.visit_expr(value); }
                    Expr::InstanceOf { value, test } => {
                        v.visit_expr(value); match test { InstanceOf::Type(t) => v.visit_type(t), InstanceOf::Pattern(p) => v.visit_pattern(p) }
                    }
                    Expr::Lambda { parameters, body } => {
                        match parameters {
                            LambdaParameters::Inferred(names) => for name in names { v.visit_name(name); },
                            LambdaParameters::Explicit(parameters) => for p in parameters { v.visit_parameter(p); },
                            LambdaParameters::Var(parameters) => for p in parameters { annotations(v, &$($m)* p.annotations); v.visit_name(&$($m)* p.name); },
                        }
                        match body { LambdaBody::Expression(e) => v.visit_expr(e), LambdaBody::Block(b) => v.visit_block(b) }
                    }
                    Expr::MethodReference { target, type_arguments, member } => {
                        match target { ReferenceTarget::Expression(e) => v.visit_expr(e), ReferenceTarget::Type(t) => v.visit_type(t) }
                        types(v, type_arguments); if let ReferenceMember::Method(name) = member { v.visit_name(name); }
                    }
                    Expr::Switch(s) => v.visit_switch(s),
                    Expr::Template(t) => {
                        v.visit_expr(&$($m)* t.processor); v.visit_string(&$($m)* t.head);
                        for p in &$($m)* t.parts { v.visit_expr(&$($m)* p.expression); v.visit_string(&$($m)* p.tail); }
                    }
                }
            }
            pub fn block<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Block) { statements(v, &$($m)* n.statements); }
            pub fn statements<V: $trait + ?Sized>(v: &mut V, n: &$($m)* [Stmt]) { for s in n { v.visit_stmt(s); } }
            pub fn stmt<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Stmt) {
                match n {
                    Stmt::Block(b) => v.visit_block(b), Stmt::Empty => (),
                    Stmt::Expression(e) | Stmt::Throw(e) | Stmt::Yield(e) | Stmt::BreakValue(e) => v.visit_expr(e),
                    Stmt::Local(l) => v.visit_local(l), Stmt::Type(t) => v.visit_type_decl(t),
                    Stmt::If { condition, then_branch, else_branch } => { v.visit_expr(condition); v.visit_stmt(then_branch); if let Some(s) = else_branch { v.visit_stmt(s); } }
                    Stmt::While { condition, body } => { v.visit_expr(condition); v.visit_stmt(body); }
                    Stmt::DoWhile { body, condition } => { v.visit_stmt(body); v.visit_expr(condition); }
                    Stmt::For { init, condition, update, body } => {
                        match init { ForInit::Empty => (), ForInit::Local(l) => v.visit_local(l), ForInit::Expressions(e) => expressions(v, e) }
                        if let Some(e) = condition { v.visit_expr(e); } expressions(v, update); v.visit_stmt(body);
                    }
                    Stmt::EnhancedFor { binding, iterable, body } => {
                        match binding {
                            ForBinding::Variable(p) => v.visit_parameter(p),
                            ForBinding::Var { annotations: a, name, .. } => { annotations(v, a); v.visit_name(name); }
                            ForBinding::Pattern(p) => v.visit_pattern(p),
                        }
                        v.visit_expr(iterable); v.visit_stmt(body);
                    }
                    Stmt::Switch(s) => v.visit_switch(s),
                    Stmt::Try(t) => {
                        for r in &$($m)* t.resources { match r { Resource::Declaration(l) => v.visit_local(l), Resource::Reference(e) => v.visit_expr(e) } }
                        v.visit_block(&$($m)* t.body);
                        for c in &$($m)* t.catches { annotations(v, &$($m)* c.annotations); types(v, &$($m)* c.types); v.visit_name(&$($m)* c.name); v.visit_block(&$($m)* c.body); }
                        if let Some(b) = &$($m)* t.finally { v.visit_block(b); }
                    }
                    Stmt::Synchronized { monitor, body } => { v.visit_expr(monitor); v.visit_block(body); }
                    Stmt::Return(value) => if let Some(e) = value { v.visit_expr(e); },
                    Stmt::Break(name) | Stmt::Continue(name) => if let Some(name) = name { v.visit_name(name); },
                    Stmt::Labeled { label, statement } => { v.visit_name(label); v.visit_stmt(statement); }
                    Stmt::Assert { condition, detail } => { v.visit_expr(condition); if let Some(e) = detail { v.visit_expr(e); } }
                    Stmt::Commented { comments, statement } => { for c in comments { v.visit_comment(c); } v.visit_stmt(statement); }
                }
            }
            pub fn local<V: $trait + ?Sized>(v: &mut V, n: &$($m)* LocalDecl) {
                annotations(v, &$($m)* n.annotations); if let LocalType::Explicit(t) = &$($m)* n.ty { v.visit_type(t); }
                for var in &$($m)* n.variables { v.visit_variable(var); }
            }
            pub fn pattern<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Pattern) {
                match n {
                    Pattern::Type { annotations: a, ty, name, .. } => { annotations(v, a); v.visit_type(ty); v.visit_name(name); }
                    Pattern::Record { ty, components, binding } => { v.visit_type(ty); for p in components { v.visit_pattern(p); } if let Some(name) = binding { v.visit_name(name); } }
                    Pattern::Var { annotations: a, name, .. } => { annotations(v, a); v.visit_name(name); }, Pattern::Unnamed => (),
                    Pattern::Parenthesized(p) => v.visit_pattern(p),
                    Pattern::Guarded { pattern, condition } => { v.visit_pattern(pattern); v.visit_expr(condition); }
                }
            }
            pub fn switch<V: $trait + ?Sized>(v: &mut V, n: &$($m)* Switch) {
                v.visit_expr(&$($m)* n.selector);
                match &$($m)* n.body {
                    SwitchBody::Groups(groups) => for g in groups { for l in &$($m)* g.labels { switch_label(v, l); } statements(v, &$($m)* g.statements); },
                    SwitchBody::Rules(rules) => for r in rules {
                        switch_label(v, &$($m)* r.label);
                        match &$($m)* r.body { SwitchRuleBody::Expression(e) | SwitchRuleBody::Throw(e) => v.visit_expr(e), SwitchRuleBody::Block(b) => v.visit_block(b) }
                    }
                }
            }
            pub fn switch_label<V: $trait + ?Sized>(v: &mut V, n: &$($m)* SwitchLabel) {
                if let SwitchLabel::Case { elements, guard } = n {
                    for e in elements { match e { CaseElement::Constant(e) => v.visit_expr(e), CaseElement::Pattern(p) => v.visit_pattern(p), _ => () } }
                    if let Some(e) = guard { v.visit_expr(e); }
                }
            }
        }
    };
}

visitors!(Visit, walk, []);
visitors!(VisitMut, walk_mut, [mut]);
