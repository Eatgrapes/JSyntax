use super::{Printer, Result};
use crate::ast::*;
use crate::version::Feature;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BodyKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
    Anonymous,
    Compact,
    Standalone,
}

impl Printer<'_> {
    pub(super) fn metadata(&mut self, meta: &DeclarationMeta, allowed: &[Modifier]) -> Result {
        self.comments(&meta.comments)?;
        self.annotations(&meta.annotations, true)?;
        for (index, modifier) in meta.modifiers.iter().enumerate() {
            if !allowed.contains(modifier) {
                return Err(Self::invalid("modifier is not allowed on this declaration"));
            }
            if meta.modifiers[..index].contains(modifier) {
                return Err(Self::invalid("duplicate modifier"));
            }
            match modifier {
                Modifier::Strictfp => self.require(Feature::Strictfp)?,
                Modifier::Default => self.require(Feature::InterfaceMethods)?,
                Modifier::Sealed | Modifier::NonSealed => self.require(Feature::SealedTypes)?,
                _ => (),
            }
        }
        for group in [
            &[Modifier::Public, Modifier::Protected, Modifier::Private][..],
            &[Modifier::Final, Modifier::Sealed, Modifier::NonSealed][..],
            &[Modifier::Final, Modifier::Abstract][..],
        ] {
            if meta
                .modifiers
                .iter()
                .filter(|modifier| group.contains(modifier))
                .count()
                > 1
            {
                return Err(Self::invalid("conflicting modifiers"));
            }
        }
        for modifier in &meta.modifiers {
            self.text(modifier.keyword());
            self.text(" ");
        }
        Ok(())
    }

    pub(super) fn type_decl(&mut self, declaration: &TypeDecl) -> Result {
        let saved = (self.switch_expressions, self.legacy_switches);
        self.switch_expressions = 0;
        self.legacy_switches = 0;
        let result = self.type_declaration(declaration).map_err(|mut error| {
            error.context.push(declaration.name.clone());
            error
        });
        (self.switch_expressions, self.legacy_switches) = saved;
        result
    }

    fn type_declaration(&mut self, declaration: &TypeDecl) -> Result {
        use Modifier::*;
        self.metadata(
            &declaration.meta,
            &[
                Public, Protected, Private, Abstract, Static, Final, Strictfp, Sealed, NonSealed,
            ],
        )?;
        let (keyword, body_kind) = match &declaration.kind {
            TypeDeclKind::Class { .. } => ("class", BodyKind::Class),
            TypeDeclKind::Interface { .. } => ("interface", BodyKind::Interface),
            TypeDeclKind::Enum { .. } => {
                self.require(Feature::Enums)?;
                ("enum", BodyKind::Enum)
            }
            TypeDeclKind::Record { .. } => {
                self.require(Feature::Records)?;
                ("record", BodyKind::Record)
            }
            TypeDeclKind::Annotation => {
                self.require(Feature::Annotations)?;
                ("@interface", BodyKind::Annotation)
            }
        };
        if matches!(
            body_kind,
            BodyKind::Enum | BodyKind::Record | BodyKind::Annotation
        ) && declaration
            .meta
            .modifiers
            .iter()
            .any(|m| matches!(m, Sealed | NonSealed))
        {
            return Err(Self::invalid(
                "sealed and non-sealed apply only to classes and interfaces",
            ));
        }
        if matches!(body_kind, BodyKind::Enum | BodyKind::Annotation)
            && !declaration.type_parameters.is_empty()
        {
            return Err(Self::invalid(
                "enums and annotation interfaces cannot declare type parameters",
            ));
        }
        self.text(keyword);
        self.text(" ");
        self.type_identifier(&declaration.name)?;
        self.type_parameters(&declaration.type_parameters)?;
        match &declaration.kind {
            TypeDeclKind::Class {
                extends,
                implements,
                permits,
            } => {
                if let Some(ty) = extends {
                    self.text(" extends ");
                    self.class_reference(ty)?;
                }
                self.type_clause(" implements ", implements)?;
                self.permits(permits, &declaration.meta)?;
            }
            TypeDeclKind::Interface { extends, permits } => {
                self.type_clause(" extends ", extends)?;
                self.permits(permits, &declaration.meta)?;
            }
            TypeDeclKind::Enum { implements, .. } => {
                self.type_clause(" implements ", implements)?
            }
            TypeDeclKind::Record {
                components,
                implements,
            } => {
                if components
                    .iter()
                    .any(|component| component.final_ || !component.dimensions.is_empty())
                {
                    return Err(Self::invalid(
                        "record components cannot be final or use declarator dimensions",
                    ));
                }
                self.text("(");
                self.parameters(components, false)?;
                self.text(")");
                self.type_clause(" implements ", implements)?;
            }
            TypeDeclKind::Annotation => (),
        }
        self.text(" ");
        self.open_block();
        if let TypeDeclKind::Enum { constants, .. } = &declaration.kind {
            for (index, constant) in constants.iter().enumerate() {
                self.comments(&constant.comments)?;
                self.annotations(&constant.annotations, true)?;
                self.identifier(&constant.name)?;
                if !constant.arguments.is_empty() {
                    self.arguments(&constant.arguments)?;
                }
                if let Some(body) = &constant.body {
                    self.text(" ");
                    self.anonymous_body(body)?;
                }
                if index + 1 != constants.len() {
                    self.text(",");
                }
                self.newline();
            }
            if !declaration.members.is_empty() {
                self.text(";");
                self.newline();
            }
        }
        self.members(&declaration.members, body_kind, Some(&declaration.name))?;
        self.close_block();
        self.newline();
        Ok(())
    }

    pub(super) fn class_reference(&mut self, ty: &Type) -> Result {
        match ty {
            Type::Class(_) => self.ty(ty),
            Type::Annotated { ty: inner, .. } if matches!(inner.as_ref(), Type::Class(_)) => {
                self.ty(ty)
            }
            _ => Err(Self::invalid("a class or interface type is required here")),
        }
    }

    fn type_clause(&mut self, keyword: &str, types: &[Type]) -> Result {
        if !types.is_empty() {
            self.text(keyword);
            self.separated(types, ", ", Self::class_reference)?;
        }
        Ok(())
    }

    fn permits(&mut self, types: &[Type], meta: &DeclarationMeta) -> Result {
        if !types.is_empty() {
            self.require(Feature::SealedTypes)?;
            if !meta.modifiers.contains(&Modifier::Sealed) {
                return Err(Self::invalid("a permits clause requires sealed"));
            }
            self.type_clause(" permits ", types)?;
        }
        Ok(())
    }

    pub(super) fn anonymous_body(&mut self, members: &[Member]) -> Result {
        let saved = (self.switch_expressions, self.legacy_switches);
        self.switch_expressions = 0;
        self.legacy_switches = 0;
        self.open_block();
        self.members(members, BodyKind::Anonymous, None)?;
        self.close_block();
        (self.switch_expressions, self.legacy_switches) = saved;
        Ok(())
    }

    pub(super) fn members(
        &mut self,
        members: &[Member],
        kind: BodyKind,
        name: Option<&str>,
    ) -> Result {
        for (index, member) in members.iter().enumerate() {
            if index != 0 {
                self.newline();
            }
            self.member(member, kind, name)?;
        }
        Ok(())
    }

    pub(super) fn standalone_member(&mut self, member: &Member) -> Result {
        self.member(member, BodyKind::Standalone, None)
    }

    fn member(&mut self, member: &Member, kind: BodyKind, name: Option<&str>) -> Result {
        match member {
            Member::Type(declaration) => {
                self.require(Feature::InnerClasses)?;
                self.type_decl(declaration)
            }
            Member::Field(field) => self.field(field),
            Member::Method(method) => {
                if kind == BodyKind::Annotation {
                    return Err(Self::invalid(
                        "use AnnotationElement for annotation interface elements",
                    ));
                }
                let modifiers = &method.meta.modifiers;
                if modifiers.contains(&Modifier::Default)
                    && !matches!(kind, BodyKind::Interface | BodyKind::Standalone)
                {
                    return Err(Self::invalid("default methods belong to interfaces"));
                }
                if kind == BodyKind::Interface {
                    if method.body.is_some() || modifiers.contains(&Modifier::Static) {
                        self.require(Feature::InterfaceMethods)?;
                    }
                    if modifiers.contains(&Modifier::Private) {
                        self.require(Feature::PrivateInterfaceMethods)?;
                    }
                    if method.body.is_some()
                        && !modifiers.iter().any(|m| {
                            matches!(m, Modifier::Default | Modifier::Static | Modifier::Private)
                        })
                    {
                        return Err(Self::invalid(
                            "an interface method body requires default, static or private",
                        ));
                    }
                } else if kind != BodyKind::Standalone
                    && method.body.is_none()
                    && !modifiers
                        .iter()
                        .any(|m| matches!(m, Modifier::Abstract | Modifier::Native))
                {
                    return Err(Self::invalid(
                        "a bodyless class method requires abstract or native",
                    ));
                }
                self.method(method)
            }
            Member::Constructor(constructor) => {
                if matches!(
                    kind,
                    BodyKind::Interface
                        | BodyKind::Annotation
                        | BodyKind::Anonymous
                        | BodyKind::Compact
                ) {
                    return Err(Self::invalid("constructors are not allowed in this body"));
                }
                if let Some(name) = name
                    && constructor.name != name
                {
                    return Err(Self::invalid(
                        "constructor name must match the enclosing type",
                    ));
                }
                if matches!(constructor.parameters, ConstructorParameters::Compact)
                    && !matches!(kind, BodyKind::Record | BodyKind::Standalone)
                {
                    return Err(Self::invalid("compact constructors belong to records"));
                }
                self.constructor(constructor)
            }
            Member::Initializer { static_, body } => {
                if matches!(kind, BodyKind::Interface | BodyKind::Annotation)
                    || (kind == BodyKind::Record && !static_)
                {
                    return Err(Self::invalid(
                        "initializer block is not allowed in this body",
                    ));
                }
                if !static_ {
                    self.require(Feature::InnerClasses)?;
                }
                if *static_ {
                    self.text("static ");
                }
                self.block(body)?;
                self.newline();
                Ok(())
            }
            Member::AnnotationElement(element) => {
                if !matches!(kind, BodyKind::Annotation | BodyKind::Standalone) {
                    return Err(Self::invalid(
                        "annotation elements belong to annotation interfaces",
                    ));
                }
                self.annotation_element(element)
            }
            Member::Empty => {
                self.text(";");
                self.newline();
                Ok(())
            }
        }
    }

    pub(super) fn field(&mut self, field: &FieldDecl) -> Result {
        use Modifier::*;
        self.metadata(
            &field.meta,
            &[
                Public, Protected, Private, Static, Final, Transient, Volatile,
            ],
        )?;
        if field.meta.modifiers.contains(&Final) && field.meta.modifiers.contains(&Volatile) {
            return Err(Self::invalid("a field cannot be both final and volatile"));
        }
        self.ty(&field.ty)?;
        self.text(" ");
        self.variables(&field.variables, false)?;
        self.text(";");
        self.newline();
        Ok(())
    }

    pub(super) fn variables(&mut self, variables: &[Variable], unnamed: bool) -> Result {
        if variables.is_empty() {
            return Err(Self::invalid(
                "a variable declaration must contain a variable",
            ));
        }
        self.separated(variables, ", ", |p, variable| {
            p.binding(&variable.name, unnamed)?;
            for dimension in &variable.dimensions {
                p.dimension(dimension)?;
            }
            if let Some(initializer) = &variable.initializer {
                p.text(" = ");
                p.initializer(initializer)?;
            }
            Ok(())
        })
    }

    pub(super) fn method(&mut self, method: &MethodDecl) -> Result {
        self.method_declaration(method).map_err(|mut error| {
            error.context.push(method.name.clone());
            error
        })
    }

    fn method_declaration(&mut self, method: &MethodDecl) -> Result {
        use Modifier::*;
        self.metadata(
            &method.meta,
            &[
                Public,
                Protected,
                Private,
                Abstract,
                Static,
                Final,
                Synchronized,
                Native,
                Strictfp,
                Default,
            ],
        )?;
        if method.body.is_some()
            && method
                .meta
                .modifiers
                .iter()
                .any(|modifier| matches!(modifier, Abstract | Native))
        {
            return Err(Self::invalid(
                "abstract and native methods cannot have a body",
            ));
        }
        self.type_parameters(&method.type_parameters)?;
        if !method.type_parameters.is_empty() {
            self.text(" ");
        }
        self.return_type(&method.return_type)?;
        self.text(" ");
        self.identifier(&method.name)?;
        self.text("(");
        if let Some(receiver) = &method.receiver {
            self.receiver(receiver)?;
            if !method.parameters.is_empty() {
                self.text(", ");
            }
        }
        self.parameters(&method.parameters, false)?;
        self.text(")");
        if matches!(method.return_type, ReturnType::Void) && !method.return_dimensions.is_empty() {
            return Err(Self::invalid("void cannot have array dimensions"));
        }
        for dimension in &method.return_dimensions {
            self.dimension(dimension)?;
        }
        self.type_clause(" throws ", &method.throws)?;
        if let Some(body) = &method.body {
            self.text(" ");
            self.block(body)?;
        } else {
            self.text(";");
        }
        self.newline();
        Ok(())
    }

    pub(super) fn constructor(&mut self, constructor: &ConstructorDecl) -> Result {
        self.constructor_declaration(constructor)
            .map_err(|mut error| {
                error.context.push(constructor.name.clone());
                error
            })
    }

    fn constructor_declaration(&mut self, constructor: &ConstructorDecl) -> Result {
        use Modifier::*;
        self.metadata(&constructor.meta, &[Public, Protected, Private])?;
        self.type_parameters(&constructor.type_parameters)?;
        if !constructor.type_parameters.is_empty() {
            self.text(" ");
        }
        self.type_identifier(&constructor.name)?;
        match &constructor.parameters {
            ConstructorParameters::Explicit {
                receiver,
                parameters,
            } => {
                self.text("(");
                if let Some(receiver) = receiver {
                    self.receiver(receiver)?;
                    if !parameters.is_empty() {
                        self.text(", ");
                    }
                }
                self.parameters(parameters, false)?;
                self.text(")");
                self.type_clause(" throws ", &constructor.throws)?;
            }
            ConstructorParameters::Compact => {
                self.require(Feature::Records)?;
                if !constructor.type_parameters.is_empty()
                    || !constructor.throws.is_empty()
                    || constructor.body.invocation.is_some()
                {
                    return Err(Self::invalid(
                        "compact constructors cannot declare type parameters, throws or explicit constructor invocations",
                    ));
                }
            }
        }
        self.text(" ");
        self.open_block();
        if !constructor.body.prologue.statements.is_empty() && constructor.body.invocation.is_some()
        {
            self.require(Feature::FlexibleConstructors)?;
        }
        self.statements(&constructor.body.prologue.statements)?;
        if let Some(invocation) = &constructor.body.invocation {
            if let ConstructorTarget::Super(Some(qualifier)) = &invocation.target {
                self.require(Feature::InnerClasses)?;
                self.expr(qualifier, 15)?;
                self.text(".");
            }
            self.explicit_type_arguments(&invocation.type_arguments)?;
            self.text(if matches!(invocation.target, ConstructorTarget::This) {
                "this"
            } else {
                "super"
            });
            self.arguments(&invocation.arguments)?;
            self.text(";");
            self.newline();
        }
        self.statements(&constructor.body.epilogue.statements)?;
        self.close_block();
        self.newline();
        Ok(())
    }

    pub(super) fn annotation_element(&mut self, element: &AnnotationElement) -> Result {
        self.require(Feature::Annotations)?;
        self.metadata(&element.meta, &[Modifier::Public, Modifier::Abstract])?;
        self.ty(&element.ty)?;
        self.text(" ");
        self.identifier(&element.name)?;
        self.text("()");
        for dimension in &element.dimensions {
            self.dimension(dimension)?;
        }
        if let Some(value) = &element.default {
            self.text(" default ");
            self.annotation_value(value)?;
        }
        self.text(";");
        self.newline();
        Ok(())
    }
}
