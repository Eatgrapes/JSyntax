use super::{Printer, Result};
use crate::ast::*;
use crate::version::Feature;

impl Printer<'_> {
    pub(super) fn ty(&mut self, ty: &Type) -> Result {
        match ty {
            Type::Primitive(primitive) => self.text(primitive.keyword()),
            Type::Class(class) => self.class_type(class)?,
            Type::Array { .. } => {
                let mut dimensions = Vec::new();
                let mut element = ty;
                while let Type::Array {
                    element: next,
                    dimension,
                } = element
                {
                    dimensions.push(dimension);
                    element = next;
                }
                self.ty(element)?;
                for dimension in dimensions {
                    self.dimension(dimension)?;
                }
            }
            Type::Annotated { annotations, ty } => {
                self.require(Feature::TypeAnnotations)?;
                if let Type::Class(class) = ty.as_ref() {
                    self.class_type_annotated(class, annotations)?;
                } else {
                    self.annotations(annotations, false)?;
                    self.ty(ty)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn class_type(&mut self, class: &ClassType) -> Result {
        self.class_type_annotated(class, &[])
    }

    pub(super) fn class_type_annotated(
        &mut self,
        class: &ClassType,
        annotations: &[Annotation],
    ) -> Result {
        if let Some((qualifier, name)) = class.name.rsplit_once('.') {
            self.qualified(qualifier, false)?;
            self.text(".");
            self.annotations(annotations, false)?;
            self.type_identifier(name)?;
        } else {
            self.annotations(annotations, false)?;
            self.type_identifier(&class.name)?;
        }
        self.type_arguments(&class.arguments)?;
        for segment in &class.nested {
            self.text(".");
            if !segment.annotations.is_empty() {
                self.require(Feature::TypeAnnotations)?;
            }
            self.annotations(&segment.annotations, false)?;
            self.type_identifier(&segment.name)?;
            self.type_arguments(&segment.arguments)?;
        }
        Ok(())
    }

    pub(super) fn type_arguments(&mut self, arguments: &[TypeArgument]) -> Result {
        if arguments.is_empty() {
            return Ok(());
        }
        self.require(Feature::Generics)?;
        self.text("<");
        self.separated(arguments, ", ", |p, arg| match arg {
            TypeArgument::Type(ty) => p.reference_type(ty),
            TypeArgument::Wildcard { annotations, bound } => {
                if !annotations.is_empty() {
                    p.require(Feature::TypeAnnotations)?;
                }
                p.annotations(annotations, false)?;
                p.text("?");
                match bound {
                    Some(WildcardBound::Extends(ty)) => {
                        p.text(" extends ");
                        p.reference_type(ty)?;
                    }
                    Some(WildcardBound::Super(ty)) => {
                        p.text(" super ");
                        p.reference_type(ty)?;
                    }
                    None => (),
                }
                Ok(())
            }
        })?;
        self.close_type_arguments();
        Ok(())
    }

    fn close_type_arguments(&mut self) {
        if self.options.language.version.number() < 7 && self.output.ends_with('>') {
            self.text(" ");
        }
        self.text(">");
    }

    pub(super) fn explicit_type_arguments(&mut self, arguments: &[Type]) -> Result {
        if !arguments.is_empty() {
            self.require(Feature::Generics)?;
            self.text("<");
            self.separated(arguments, ", ", Self::reference_type)?;
            self.close_type_arguments();
        }
        Ok(())
    }

    pub(super) fn reference_type(&mut self, ty: &Type) -> Result {
        if ty.primitive().is_some() {
            return Err(Self::invalid("a reference type is required here"));
        }
        self.ty(ty)
    }

    pub(super) fn type_parameters(&mut self, parameters: &[TypeParameter]) -> Result {
        if parameters.is_empty() {
            return Ok(());
        }
        self.require(Feature::Generics)?;
        self.text("<");
        self.separated(parameters, ", ", |p, parameter| {
            if !parameter.annotations.is_empty() {
                p.require(Feature::TypeAnnotations)?;
            }
            p.annotations(&parameter.annotations, false)?;
            p.type_identifier(&parameter.name)?;
            if !parameter.bounds.is_empty() {
                p.text(" extends ");
                p.separated(&parameter.bounds, " & ", Self::reference_type)?;
            }
            Ok(())
        })?;
        self.close_type_arguments();
        Ok(())
    }

    pub(super) fn return_type(&mut self, ty: &ReturnType) -> Result {
        match ty {
            ReturnType::Void => {
                self.text("void");
                Ok(())
            }
            ReturnType::Type(ty) => self.ty(ty),
        }
    }

    pub(super) fn dimension(&mut self, dimension: &ArrayDimension) -> Result {
        if !dimension.annotations.is_empty() {
            self.require(Feature::TypeAnnotations)?;
            self.text(" ");
            self.annotations(&dimension.annotations, false)?;
        }
        self.text("[]");
        Ok(())
    }

    pub(super) fn annotation(&mut self, annotation: &Annotation) -> Result {
        self.require(Feature::Annotations)?;
        self.text("@");
        self.qualified(&annotation.name, true)?;
        match &annotation.arguments {
            AnnotationArguments::Marker => (),
            AnnotationArguments::Single(value) => {
                self.text("(");
                self.annotation_value(value)?;
                self.text(")");
            }
            AnnotationArguments::Named(values) => {
                self.text("(");
                self.separated(values, ", ", |p, (name, value)| {
                    p.identifier(name)?;
                    p.text(" = ");
                    p.annotation_value(value)
                })?;
                self.text(")");
            }
        }
        Ok(())
    }

    pub(super) fn annotations(&mut self, annotations: &[Annotation], lines: bool) -> Result {
        for annotation in annotations {
            self.annotation(annotation)?;
            if lines {
                self.newline();
            } else {
                self.text(" ");
            }
        }
        Ok(())
    }

    pub(super) fn annotation_value(&mut self, value: &AnnotationValue) -> Result {
        match value {
            AnnotationValue::Expression(expression) => self.expr(expression, 2),
            AnnotationValue::Annotation(annotation) => self.annotation(annotation),
            AnnotationValue::Array(values) => {
                self.text("{");
                self.separated(values, ", ", Self::annotation_value)?;
                self.text("}");
                Ok(())
            }
        }
    }

    pub(super) fn parameter(&mut self, parameter: &Parameter, unnamed: bool) -> Result {
        self.annotations(&parameter.annotations, false)?;
        if parameter.final_ {
            self.text("final ");
        }
        self.ty(&parameter.ty)?;
        if let Some(annotations) = &parameter.varargs {
            self.require(Feature::Varargs)?;
            if !parameter.dimensions.is_empty() {
                return Err(Self::invalid(
                    "varargs parameters cannot have declarator dimensions",
                ));
            }
            if !annotations.is_empty() {
                self.require(Feature::TypeAnnotations)?;
                self.text(" ");
                self.annotations(annotations, false)?;
            }
            self.text("...");
        }
        self.text(" ");
        self.binding(&parameter.name, unnamed)?;
        for dimension in &parameter.dimensions {
            self.dimension(dimension)?;
        }
        Ok(())
    }

    pub(super) fn parameters(&mut self, parameters: &[Parameter], unnamed: bool) -> Result {
        for (index, parameter) in parameters.iter().enumerate() {
            if index != 0 {
                self.text(", ");
            }
            if parameter.varargs.is_some() && index + 1 != parameters.len() {
                return Err(Self::invalid("only the last parameter may be varargs"));
            }
            self.parameter(parameter, unnamed)?;
        }
        Ok(())
    }

    pub(super) fn receiver(&mut self, receiver: &ReceiverParameter) -> Result {
        self.require(Feature::TypeAnnotations)?;
        self.annotations(&receiver.annotations, false)?;
        self.ty(&receiver.ty)?;
        self.text(" ");
        if let Some(qualifier) = &receiver.qualifier {
            self.identifier(qualifier)?;
            self.text(".");
        }
        self.text("this");
        Ok(())
    }

    pub(super) fn pattern(&mut self, pattern: &Pattern) -> Result {
        self.pattern_in(pattern, false)
    }

    fn pattern_in(&mut self, pattern: &Pattern, component: bool) -> Result {
        match pattern {
            Pattern::Type {
                annotations,
                final_,
                ty,
                name,
            } => {
                self.require(Feature::InstanceOfPatterns)?;
                if ty.primitive().is_some() && !component {
                    self.require(Feature::PrimitivePatterns)?;
                }
                self.annotations(annotations, false)?;
                if *final_ {
                    self.text("final ");
                }
                self.ty(ty)?;
                self.text(" ");
                self.binding(name, true)?;
            }
            Pattern::Record {
                ty,
                components,
                binding,
            } => {
                self.require(Feature::RecordPatterns)?;
                self.reference_type(ty)?;
                self.text("(");
                self.separated(components, ", ", |p, pattern| p.pattern_in(pattern, true))?;
                self.text(")");
                if let Some(name) = binding {
                    self.require(Feature::RecordPatternBindings)?;
                    self.text(" ");
                    self.identifier(name)?;
                }
            }
            Pattern::Var {
                annotations,
                final_,
                name,
            } => {
                self.require(Feature::RecordPatterns)?;
                if !component {
                    return Err(Self::invalid("var patterns belong in record components"));
                }
                self.annotations(annotations, false)?;
                if *final_ {
                    self.text("final ");
                }
                self.text("var ");
                self.binding(name, true)?;
            }
            Pattern::Unnamed => {
                self.require(Feature::UnnamedVariables)?;
                if !component {
                    return Err(Self::invalid(
                        "an unnamed pattern belongs in a record component",
                    ));
                }
                self.text("_");
            }
            Pattern::Parenthesized(pattern) => {
                self.require(Feature::ParenthesizedPatterns)?;
                self.text("(");
                self.pattern_in(pattern, component)?;
                self.text(")");
            }
            Pattern::Guarded { pattern, condition } => {
                self.require(Feature::LegacyPatterns)?;
                self.pattern(pattern)?;
                self.text(" && ");
                self.expr(condition, 5)?;
            }
        }
        Ok(())
    }
}
