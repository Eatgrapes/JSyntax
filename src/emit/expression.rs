use super::{Printer, Result};
use crate::ast::*;
use crate::version::Feature;

impl Printer<'_> {
    pub(super) fn expression(&mut self, expression: &Expr) -> Result {
        self.expr(expression, 0)
    }

    pub(super) fn expr(&mut self, expression: &Expr, minimum: u8) -> Result {
        let parentheses = precedence(expression) < minimum;
        if parentheses {
            self.text("(");
        }
        match expression {
            Expr::Name(name) => self.qualified(name, false)?,
            Expr::Literal(literal) => self.literal(literal)?,
            Expr::This(qualifier) | Expr::Super(qualifier) => {
                if let Some(qualifier) = qualifier {
                    self.require(Feature::InnerClasses)?;
                    self.qualified(qualifier, true)?;
                    self.text(".");
                }
                self.text(if matches!(expression, Expr::This(_)) {
                    "this"
                } else {
                    "super"
                });
            }
            Expr::ClassLiteral(ty) => {
                self.require(Feature::ClassLiterals)?;
                self.return_type(ty)?;
                self.text(".class");
            }
            Expr::Parenthesized(inner) => {
                self.text("(");
                self.expression(inner)?;
                self.text(")");
            }
            Expr::Field { target, name } => {
                self.primary(target)?;
                self.text(".");
                self.identifier(name)?;
            }
            Expr::ArrayAccess { array, index } => {
                self.primary(array)?;
                self.text("[");
                self.expression(index)?;
                self.text("]");
            }
            Expr::Call(call) => self.call(call)?,
            Expr::New(new) => self.new_object(new)?,
            Expr::NewArray(new) => self.new_array(new)?,
            Expr::Unary { op, operand } => {
                if op.is_update() && !assignable(operand) {
                    return Err(Self::invalid(
                        "an increment or decrement operand must be assignable",
                    ));
                }
                if op.is_postfix() {
                    self.expr(operand, 14)?;
                    self.text(op.token());
                } else {
                    self.text(op.token());
                    // Keep unary signs apart from ++/-- and preserve negative literal grouping.
                    if matches!(
                        op,
                        UnaryOp::Plus
                            | UnaryOp::Minus
                            | UnaryOp::PreIncrement
                            | UnaryOp::PreDecrement
                    ) {
                        self.text("(");
                        self.expression(operand)?;
                        self.text(")");
                    } else {
                        self.expr(operand, 13)?;
                    }
                }
            }
            Expr::Binary { left, op, right } => {
                self.expr(left, op.precedence())?;
                self.text(" ");
                self.text(op.token());
                self.text(" ");
                self.expr(right, op.precedence() + 1)?;
            }
            Expr::Assign { target, op, value } => {
                if !assignable(target) {
                    return Err(Self::invalid(
                        "assignment target must be a name, field or array element",
                    ));
                }
                self.expression(target)?;
                self.text(" ");
                self.text(op.token());
                self.text(" ");
                self.expr(value, 1)?;
            }
            Expr::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                self.expr(condition, 3)?;
                self.text(" ? ");
                self.expression(then_value)?;
                self.text(" : ");
                self.expr(
                    else_value,
                    if matches!(else_value.as_ref(), Expr::Lambda { .. }) {
                        0
                    } else {
                        2
                    },
                )?;
            }
            Expr::Cast { types, value } => {
                if types.is_empty() {
                    return Err(Self::invalid("a cast requires at least one type"));
                }
                if types.len() > 1 {
                    self.require(Feature::IntersectionCast)?;
                }
                self.text("(");
                self.separated(types, " & ", Self::ty)?;
                self.text(") ");
                // Isolate both reference casts before unary signs and lambda bodies.
                self.text("(");
                self.expression(value)?;
                self.text(")");
            }
            Expr::InstanceOf { value, test } => {
                self.expr(value, 9)?;
                self.text(" instanceof ");
                match test {
                    InstanceOf::Type(ty) => {
                        if ty.primitive().is_some() {
                            self.require(Feature::PrimitivePatterns)?;
                        }
                        self.ty(ty)?;
                    }
                    InstanceOf::Pattern(pattern) => {
                        if matches!(pattern.as_ref(), Pattern::Unnamed | Pattern::Var { .. }) {
                            return Err(Self::invalid(
                                "instanceof requires a type or record pattern",
                            ));
                        }
                        self.pattern(pattern)?;
                    }
                }
            }
            Expr::Lambda { parameters, body } => {
                self.require(Feature::Lambda)?;
                self.text("(");
                match parameters {
                    LambdaParameters::Inferred(names) => {
                        self.separated(names, ", ", |p, name| p.binding(name, true))?
                    }
                    LambdaParameters::Explicit(parameters) => self.parameters(parameters, true)?,
                    LambdaParameters::Var(parameters) => {
                        self.require(Feature::LambdaVar)?;
                        self.separated(parameters, ", ", |p, parameter| {
                            p.annotations(&parameter.annotations, false)?;
                            if parameter.final_ {
                                p.text("final ");
                            }
                            p.text("var ");
                            p.binding(&parameter.name, true)
                        })?;
                    }
                }
                self.text(") -> ");
                let saved = (self.switch_expressions, self.legacy_switches);
                self.switch_expressions = 0;
                self.legacy_switches = 0;
                match body {
                    LambdaBody::Expression(value) => self.expression(value)?,
                    LambdaBody::Block(block) => self.block(block)?,
                }
                (self.switch_expressions, self.legacy_switches) = saved;
            }
            Expr::MethodReference {
                target,
                type_arguments,
                member,
            } => {
                self.require(Feature::Lambda)?;
                match target {
                    ReferenceTarget::Expression(value) => self.primary(value)?,
                    ReferenceTarget::Type(ty) => self.ty(ty)?,
                }
                self.text("::");
                self.explicit_type_arguments(type_arguments)?;
                match member {
                    ReferenceMember::Method(name) => self.identifier(name)?,
                    ReferenceMember::Constructor => {
                        if !matches!(target, ReferenceTarget::Type(_)) {
                            return Err(Self::invalid("a constructor reference requires a type"));
                        }
                        self.text("new");
                    }
                }
            }
            Expr::Switch(switch) => {
                self.require(Feature::SwitchExpressions)?;
                self.switch_expressions += 1;
                if self.options.language.version.number() == 12 {
                    self.legacy_switches += 1;
                }
                self.switch(switch, true)?;
                self.switch_expressions -= 1;
                if self.options.language.version.number() == 12 {
                    self.legacy_switches -= 1;
                }
            }
            Expr::Template(template) => self.template(template)?,
        }
        if parentheses {
            self.text(")");
        }
        Ok(())
    }

    fn primary(&mut self, expression: &Expr) -> Result {
        if matches!(
            expression,
            Expr::Literal(
                Literal::Int(_) | Literal::Long(_) | Literal::Float(_) | Literal::Double(_)
            )
        ) {
            self.text("(");
            self.expression(expression)?;
            self.text(")");
            Ok(())
        } else {
            self.expr(expression, 15)
        }
    }

    fn call(&mut self, call: &CallExpr) -> Result {
        if let Some(target) = &call.target {
            self.primary(target)?;
            self.text(".");
        } else {
            if !call.type_arguments.is_empty() {
                return Err(Self::invalid(
                    "explicit method type arguments require a target (such as this)",
                ));
            }
            if call.name == "yield" && self.options.language.supports(Feature::Yield) {
                return Err(Self::invalid(
                    "yield cannot be an unqualified method invocation",
                ));
            }
        }
        self.explicit_type_arguments(&call.type_arguments)?;
        self.identifier(&call.name)?;
        self.arguments(&call.arguments)
    }

    pub(super) fn arguments(&mut self, arguments: &[Expr]) -> Result {
        self.text("(");
        self.separated(arguments, ", ", Self::expression)?;
        self.text(")");
        Ok(())
    }

    fn new_object(&mut self, new: &NewExpr) -> Result {
        if let Some(qualifier) = &new.qualifier {
            self.require(Feature::InnerClasses)?;
            self.primary(qualifier)?;
            self.text(".");
        }
        self.text("new ");
        self.explicit_type_arguments(&new.constructor_type_arguments)?;
        if !new.constructor_type_arguments.is_empty() {
            self.text(" ");
        }
        if !new.annotations.is_empty() {
            self.require(Feature::TypeAnnotations)?;
        }
        self.class_type_annotated(&new.ty, &new.annotations)?;
        if new.diamond {
            let arguments = new
                .ty
                .nested
                .last()
                .map_or(&new.ty.arguments, |segment| &segment.arguments);
            if !arguments.is_empty() {
                return Err(Self::invalid(
                    "diamond cannot be combined with explicit class type arguments",
                ));
            }
            self.require(Feature::Diamond)?;
            if new.body.is_some() {
                self.require(Feature::AnonymousDiamond)?;
            }
            self.text("<>");
        }
        self.arguments(&new.arguments)?;
        if let Some(members) = &new.body {
            self.require(Feature::InnerClasses)?;
            self.text(" ");
            self.anonymous_body(members)?;
        }
        Ok(())
    }

    fn new_array(&mut self, new: &NewArrayExpr) -> Result {
        if new.dimensions.is_empty() {
            return Err(Self::invalid("an array creation requires dimensions"));
        }
        if matches!(new.element, Type::Array { .. }) {
            return Err(Self::invalid(
                "array creation dimensions belong in dimensions, not the element type",
            ));
        }
        let mut saw_unsized = false;
        for dimension in &new.dimensions {
            if dimension.length.is_none() {
                saw_unsized = true;
            } else if saw_unsized || new.initializer.is_some() {
                return Err(Self::invalid(
                    "sized dimensions must precede unsized dimensions and cannot accompany an initializer",
                ));
            }
        }
        if new.initializer.is_none() && new.dimensions[0].length.is_none() {
            return Err(Self::invalid(
                "an array creation requires a length or initializer",
            ));
        }
        self.text("new ");
        self.ty(&new.element)?;
        for dimension in &new.dimensions {
            if !dimension.dimension.annotations.is_empty() {
                self.require(Feature::TypeAnnotations)?;
                self.text(" ");
                self.annotations(&dimension.dimension.annotations, false)?;
            }
            self.text("[");
            if let Some(length) = &dimension.length {
                self.expression(length)?;
            }
            self.text("]");
        }
        if let Some(initializer) = &new.initializer {
            self.text(" ");
            self.array_initializer(initializer)?;
        }
        Ok(())
    }

    pub(super) fn initializer(&mut self, initializer: &VariableInitializer) -> Result {
        match initializer {
            VariableInitializer::Expression(expression) => self.expression(expression),
            VariableInitializer::Array(array) => self.array_initializer(array),
        }
    }

    pub(super) fn array_initializer(&mut self, initializer: &ArrayInitializer) -> Result {
        self.text("{");
        self.separated(&initializer.elements, ", ", Self::initializer)?;
        self.text("}");
        Ok(())
    }

    fn template(&mut self, template: &StringTemplate) -> Result {
        self.require(Feature::StringTemplates)?;
        self.primary(&template.processor)?;
        self.text(".");
        self.text(if template.multiline { "\"\"\"" } else { "\"" });
        if template.multiline {
            self.newline();
        }
        self.escaped(&template.head, false, template.multiline);
        for part in &template.parts {
            self.text("\\{");
            self.expression(&part.expression)?;
            self.text("}");
            self.escaped(&part.tail, false, template.multiline);
        }
        self.text(if template.multiline { "\"\"\"" } else { "\"" });
        Ok(())
    }
}

fn precedence(expression: &Expr) -> u8 {
    match expression {
        Expr::Lambda { .. } => 0,
        Expr::Assign { .. } => 1,
        Expr::Conditional { .. } => 2,
        Expr::Binary { op, .. } => op.precedence(),
        Expr::InstanceOf { .. } => 9,
        Expr::Cast { .. } => 13,
        Expr::Unary { op, .. } => {
            if op.is_postfix() {
                14
            } else {
                13
            }
        }
        Expr::MethodReference { .. } => 0,
        Expr::Literal(Literal::Int(value)) if *value < 0 => 13,
        Expr::Literal(Literal::Long(value)) if *value < 0 => 13,
        Expr::Literal(Literal::Float(value)) if value.is_sign_negative() => 13,
        Expr::Literal(Literal::Double(value)) if value.is_sign_negative() => 13,
        _ => 15,
    }
}

fn assignable(expression: &Expr) -> bool {
    match expression {
        Expr::Name(_) | Expr::Field { .. } | Expr::ArrayAccess { .. } => true,
        Expr::Parenthesized(inner) => assignable(inner),
        _ => false,
    }
}
