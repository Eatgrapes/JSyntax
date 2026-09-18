//! Optional adapters for already-decoded ferro-jtype values. No class bytes are read.
//!
//! Default conversion replaces package `/` separators only. Resolve nested classes
//! from your class metadata with [`type_with_names`] or an AST visitor; `$` alone
//! does not establish nesting. Generic signatures are not parsed by this adapter.

use crate::ast::{ClassType, MethodDecl, Parameter, PrimitiveType, ReturnType, Type};
use ferro_jtype::{ClassName, InferredType, MethodDescriptor, ReferenceType, TypeDescriptor};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    AmbiguousIntegral,
    UnknownReference,
    NullType,
    Alternatives,
    NoValue,
    ReturnAddress,
    Conflict,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AmbiguousIntegral => "integral inference has multiple source-type candidates",
            Self::UnknownReference => "reference inference has no known source type",
            Self::NullType => "null does not determine a denotable declaration type",
            Self::Alternatives => {
                "alternative types must be resolved before constructing source syntax"
            }
            Self::NoValue => "bottom inference does not describe a value",
            Self::ReturnAddress => "a JVM return address has no Java source type",
            Self::Conflict => "conflicting inference has no Java source type",
        })
    }
}

impl std::error::Error for ConversionError {}

impl From<ferro_jtype::PrimitiveType> for PrimitiveType {
    fn from(value: ferro_jtype::PrimitiveType) -> Self {
        use ferro_jtype::PrimitiveType as P;
        match value {
            P::Boolean => Self::Boolean,
            P::Byte => Self::Byte,
            P::Short => Self::Short,
            P::Char => Self::Char,
            P::Int => Self::Int,
            P::Long => Self::Long,
            P::Float => Self::Float,
            P::Double => Self::Double,
        }
    }
}

impl From<&ClassName> for ClassType {
    fn from(value: &ClassName) -> Self {
        Self::new(value.as_str().replace('/', "."))
    }
}

/// Uses caller-owned nesting/name information without coupling the AST to a class reader.
pub fn type_with_names(
    descriptor: &TypeDescriptor,
    resolve: &mut impl FnMut(&ClassName) -> ClassType,
) -> Type {
    match descriptor {
        TypeDescriptor::Primitive(primitive) => Type::Primitive((*primitive).into()),
        TypeDescriptor::Reference(name) => Type::Class(resolve(name)),
        TypeDescriptor::Array {
            dimensions,
            element,
        } => {
            let mut ty = type_with_names(element, resolve);
            for _ in 0..*dimensions {
                ty = ty.array();
            }
            ty
        }
    }
}

impl From<&TypeDescriptor> for Type {
    fn from(value: &TypeDescriptor) -> Self {
        type_with_names(value, &mut |name| ClassType::from(name))
    }
}

impl From<&ferro_jtype::ReturnType> for ReturnType {
    fn from(value: &ferro_jtype::ReturnType) -> Self {
        match value {
            ferro_jtype::ReturnType::Void => Self::Void,
            ferro_jtype::ReturnType::Type(ty) => Self::Type(ty.into()),
        }
    }
}

impl TryFrom<&InferredType> for Type {
    type Error = ConversionError;
    fn try_from(value: &InferredType) -> Result<Self, Self::Error> {
        match value {
            InferredType::Int => Ok(Self::INT),
            InferredType::Integral(set) => set
                .exact_type()
                .map(|primitive| Self::Primitive(primitive.into()))
                .ok_or(ConversionError::AmbiguousIntegral),
            InferredType::Float => Ok(Self::FLOAT),
            InferredType::Long => Ok(Self::LONG),
            InferredType::Double => Ok(Self::DOUBLE),
            InferredType::Reference(ReferenceType::Exact(name))
            | InferredType::Uninitialized {
                class_name: name, ..
            }
            | InferredType::UninitializedThis { class_name: name } => Ok(Self::Class(name.into())),
            InferredType::Reference(ReferenceType::Array(ty)) => Ok(ty.into()),
            InferredType::Reference(ReferenceType::Null) => Err(ConversionError::NullType),
            InferredType::Reference(ReferenceType::Unknown) => {
                Err(ConversionError::UnknownReference)
            }
            InferredType::Alternatives(_) => Err(ConversionError::Alternatives),
            InferredType::Bottom => Err(ConversionError::NoValue),
            InferredType::ReturnAddress => Err(ConversionError::ReturnAddress),
            InferredType::Conflict => Err(ConversionError::Conflict),
        }
    }
}

/// Creates a bodyless method with placeholder names `arg0`, `arg1`, ... .
/// Replace names from debug metadata and attach a body or appropriate modifiers.
pub fn method_from_descriptor(
    name: impl Into<String>,
    descriptor: &MethodDescriptor,
) -> MethodDecl {
    let mut method = MethodDecl::new(name, ReturnType::from(descriptor.return_type()));
    method.parameters = descriptor
        .parameters()
        .iter()
        .enumerate()
        .map(|(index, ty)| Parameter::new(ty.into(), format!("arg{index}")))
        .collect();
    method
}
