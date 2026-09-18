use super::{Printer, Result};
use crate::ast::*;
use crate::version::Feature;

impl Printer<'_> {
    pub(super) fn block(&mut self, block: &Block) -> Result {
        self.open_block();
        self.statements(&block.statements)?;
        self.close_block();
        Ok(())
    }

    pub(super) fn statements(&mut self, statements: &[Stmt]) -> Result {
        for statement in statements {
            self.statement(statement)?;
        }
        Ok(())
    }

    pub(super) fn statement(&mut self, statement: &Stmt) -> Result {
        match statement {
            Stmt::Block(block) => self.block(block)?,
            Stmt::Empty => self.text(";"),
            Stmt::Expression(expression) => {
                self.statement_expression(expression)?;
                self.text(";");
            }
            Stmt::Local(local) => {
                self.local(local, false)?;
                self.text(";");
            }
            Stmt::Type(declaration) => {
                self.require(Feature::InnerClasses)?;
                if matches!(
                    declaration.kind,
                    TypeDeclKind::Enum { .. } | TypeDeclKind::Interface { .. }
                ) && self.options.language.version.number() < 16
                {
                    return Err(Self::invalid(
                        "local enum and interface declarations require Java 16",
                    ));
                }
                if matches!(declaration.kind, TypeDeclKind::Annotation) {
                    return Err(Self::invalid(
                        "annotation interfaces cannot be local declarations",
                    ));
                }
                return self.type_decl(declaration);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.text("if (");
                self.expression(condition)?;
                self.text(") ");
                self.controlled(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.text(" else ");
                    self.controlled(else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                self.text("while (");
                self.expression(condition)?;
                self.text(") ");
                self.controlled(body)?;
            }
            Stmt::DoWhile { body, condition } => {
                self.text("do ");
                self.controlled(body)?;
                self.text(" while (");
                self.expression(condition)?;
                self.text(");");
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.text("for (");
                match init {
                    ForInit::Empty => (),
                    ForInit::Local(local) => self.local(local, false)?,
                    ForInit::Expressions(expressions) => {
                        self.separated(expressions, ", ", Self::statement_expression)?
                    }
                }
                self.text("; ");
                if let Some(condition) = condition {
                    self.expression(condition)?;
                }
                self.text("; ");
                self.separated(update, ", ", Self::statement_expression)?;
                self.text(") ");
                self.controlled(body)?;
            }
            Stmt::EnhancedFor {
                binding,
                iterable,
                body,
            } => {
                self.require(Feature::EnhancedFor)?;
                self.text("for (");
                match binding {
                    ForBinding::Variable(parameter) => {
                        if parameter.varargs.is_some() {
                            return Err(Self::invalid(
                                "an enhanced-for variable cannot be varargs",
                            ));
                        }
                        self.parameter(parameter, true)?;
                    }
                    ForBinding::Var {
                        annotations,
                        final_,
                        name,
                    } => {
                        self.require(Feature::Var)?;
                        self.annotations(annotations, false)?;
                        if *final_ {
                            self.text("final ");
                        }
                        self.text("var ");
                        self.binding(name, true)?;
                    }
                    ForBinding::Pattern(pattern) => {
                        self.require(Feature::RecordFor)?;
                        self.pattern(pattern)?;
                    }
                }
                self.text(" : ");
                self.expression(iterable)?;
                self.text(") ");
                self.controlled(body)?;
            }
            Stmt::Switch(switch) => self.switch(switch, false)?,
            Stmt::Try(try_stmt) => self.try_statement(try_stmt)?,
            Stmt::Synchronized { monitor, body } => {
                self.text("synchronized (");
                self.expression(monitor)?;
                self.text(") ");
                self.block(body)?;
            }
            Stmt::Return(value) => {
                self.text("return");
                if let Some(value) = value {
                    self.text(" ");
                    self.expression(value)?;
                }
                self.text(";");
            }
            Stmt::Throw(value) => {
                self.text("throw ");
                self.expression(value)?;
                self.text(";");
            }
            Stmt::Break(label) | Stmt::Continue(label) => {
                self.text(if matches!(statement, Stmt::Break(_)) {
                    "break"
                } else {
                    "continue"
                });
                if let Some(label) = label {
                    self.text(" ");
                    self.identifier(label)?;
                }
                self.text(";");
            }
            Stmt::Yield(value) => {
                self.require(Feature::Yield)?;
                if self.switch_expressions == 0 {
                    return Err(Self::invalid(
                        "yield requires an enclosing switch expression",
                    ));
                }
                self.text("yield ");
                self.expression(value)?;
                self.text(";");
            }
            Stmt::BreakValue(value) => {
                self.require(Feature::BreakValue)?;
                if self.legacy_switches == 0 {
                    return Err(Self::invalid(
                        "a value break requires a Java 12 switch expression",
                    ));
                }
                self.text("break ");
                self.expression(value)?;
                self.text(";");
            }
            Stmt::Labeled { label, statement } => {
                if matches!(statement.as_ref(), Stmt::Local(_) | Stmt::Type(_)) {
                    return Err(Self::invalid(
                        "a label requires a statement, not a declaration",
                    ));
                }
                self.identifier(label)?;
                self.text(": ");
                return self.statement(statement);
            }
            Stmt::Assert { condition, detail } => {
                self.require(Feature::Assert)?;
                self.text("assert ");
                self.expression(condition)?;
                if let Some(detail) = detail {
                    self.text(" : ");
                    self.expression(detail)?;
                }
                self.text(";");
            }
            Stmt::Commented {
                comments,
                statement,
            } => {
                self.comments(comments)?;
                return self.statement(statement);
            }
        }
        self.newline();
        Ok(())
    }

    fn controlled(&mut self, statement: &Stmt) -> Result {
        if let Stmt::Block(block) = statement {
            self.block(block)
        } else {
            self.open_block();
            self.statement(statement)?;
            self.close_block();
            Ok(())
        }
    }

    pub(super) fn statement_expression(&mut self, expression: &Expr) -> Result {
        if !expression.is_statement_expression() {
            return Err(Self::invalid(
                "only assignments, updates, method invocations and object creation are statement expressions",
            ));
        }
        self.expression(expression)
    }

    pub(super) fn local(&mut self, local: &LocalDecl, resource: bool) -> Result {
        self.annotations(&local.annotations, false)?;
        if local.final_ {
            self.text("final ");
        }
        match &local.ty {
            LocalType::Explicit(ty) => self.ty(ty)?,
            LocalType::Var => {
                self.require(Feature::Var)?;
                if local.variables.len() != 1
                    || local.variables[0].initializer.is_none()
                    || !local.variables[0].dimensions.is_empty()
                {
                    return Err(Self::invalid(
                        "var requires exactly one initialized variable without declarator dimensions",
                    ));
                }
                if matches!(
                    local.variables[0].initializer,
                    Some(VariableInitializer::Array(_))
                ) {
                    return Err(Self::invalid("var cannot infer a bare array initializer"));
                }
                self.text("var");
            }
        }
        if resource && (local.variables.len() != 1 || local.variables[0].initializer.is_none()) {
            return Err(Self::invalid(
                "a resource declaration requires one initialized variable",
            ));
        }
        if local
            .variables
            .iter()
            .any(|variable| variable.name == "_" && variable.initializer.is_none())
            && self.options.language.version.number() >= 9
        {
            return Err(Self::invalid(
                "an unnamed local variable requires an initializer",
            ));
        }
        self.text(" ");
        self.variables(&local.variables, true)
    }

    fn try_statement(&mut self, statement: &TryStmt) -> Result {
        if statement.resources.is_empty()
            && statement.catches.is_empty()
            && statement.finally.is_none()
        {
            return Err(Self::invalid("try requires resources, a catch or finally"));
        }
        self.text("try");
        if !statement.resources.is_empty() {
            self.require(Feature::TryResources)?;
            self.text(" (");
            self.separated(&statement.resources, "; ", |p, resource| match resource {
                Resource::Declaration(local) => p.local(local, true),
                Resource::Reference(expression) => {
                    p.require(Feature::ResourceReferences)?;
                    if !matches!(expression, Expr::Name(_) | Expr::Field { .. }) {
                        return Err(Self::invalid(
                            "a resource reference must be a name or field access",
                        ));
                    }
                    p.expression(expression)
                }
            })?;
            self.text(")");
        }
        self.text(" ");
        self.block(&statement.body)?;
        for catch in &statement.catches {
            if catch.types.is_empty() {
                return Err(Self::invalid("a catch requires an exception type"));
            }
            if catch.types.len() > 1 {
                self.require(Feature::MultiCatch)?;
            }
            self.text(" catch (");
            self.annotations(&catch.annotations, false)?;
            if catch.final_ {
                self.text("final ");
            }
            self.separated(&catch.types, " | ", Self::class_reference)?;
            self.text(" ");
            self.binding(&catch.name, true)?;
            self.text(") ");
            self.block(&catch.body)?;
        }
        if let Some(finally) = &statement.finally {
            self.text(" finally ");
            self.block(finally)?;
        }
        Ok(())
    }

    pub(super) fn switch(&mut self, switch: &Switch, expression: bool) -> Result {
        self.text("switch (");
        self.expression(&switch.selector)?;
        self.text(") ");
        self.open_block();
        match &switch.body {
            SwitchBody::Groups(groups) => {
                if expression && groups.is_empty() {
                    return Err(Self::invalid("a switch expression requires a case"));
                }
                for group in groups {
                    if group.labels.is_empty() {
                        return Err(Self::invalid("a switch group requires a label"));
                    }
                    for label in &group.labels {
                        self.switch_label(label)?;
                        self.text(":");
                        self.newline();
                    }
                    self.depth += 1;
                    self.statements(&group.statements)?;
                    self.depth -= 1;
                }
            }
            SwitchBody::Rules(rules) => {
                self.require(Feature::SwitchRules)?;
                if rules.is_empty() {
                    return Err(Self::invalid("an arrow switch requires a rule"));
                }
                for rule in rules {
                    self.switch_label(&rule.label)?;
                    self.text(" -> ");
                    match &rule.body {
                        SwitchRuleBody::Expression(value) => {
                            if expression {
                                self.expression(value)?;
                            } else {
                                self.statement_expression(value)?;
                            }
                            self.text(";");
                        }
                        SwitchRuleBody::Block(block) => self.block(block)?,
                        SwitchRuleBody::Throw(value) => {
                            self.text("throw ");
                            self.expression(value)?;
                            self.text(";");
                        }
                    }
                    self.newline();
                }
            }
        }
        self.close_block();
        Ok(())
    }

    fn switch_label(&mut self, label: &SwitchLabel) -> Result {
        match label {
            SwitchLabel::Default => self.text("default"),
            SwitchLabel::Case { elements, guard } => {
                if elements.is_empty() {
                    return Err(Self::invalid("a case label cannot be empty"));
                }
                if elements.len() > 1 {
                    self.require(Feature::SwitchRules)?;
                }
                if elements
                    .iter()
                    .any(|element| matches!(element, CaseElement::Default))
                    && !matches!(
                        elements.as_slice(),
                        [CaseElement::Null, CaseElement::Default]
                    )
                {
                    return Err(Self::invalid(
                        "default in a case list must follow a single null label",
                    ));
                }
                let pattern_count = elements
                    .iter()
                    .filter(|element| matches!(element, CaseElement::Pattern(_)))
                    .count();
                if pattern_count > 0
                    && elements.len() > 1
                    && self.options.language.version.number() >= 21
                {
                    if pattern_count != elements.len() {
                        return Err(Self::invalid(
                            "patterns cannot be mixed with constants in a case label",
                        ));
                    }
                    self.require(Feature::UnnamedVariables)?;
                }
                self.text("case ");
                self.separated(elements, ", ", |p, element| match element {
                    CaseElement::Constant(expression) => {
                        match expression {
                            Expr::Literal(Literal::Null) => p.require(Feature::PatternSwitch)?,
                            Expr::Literal(Literal::String(_))
                                if p.options.language.version.number() < 7 =>
                            {
                                return Err(Self::invalid("string switch labels require Java 7"));
                            }
                            Expr::Literal(
                                Literal::Boolean(_)
                                | Literal::Long(_)
                                | Literal::Float(_)
                                | Literal::Double(_),
                            ) => p.require(Feature::PrimitivePatterns)?,
                            _ => (),
                        }
                        p.expr(expression, 2)
                    }
                    CaseElement::Pattern(pattern) => {
                        p.require(Feature::PatternSwitch)?;
                        if matches!(pattern, Pattern::Var { .. } | Pattern::Unnamed) {
                            return Err(Self::invalid("a case pattern must have a type"));
                        }
                        p.pattern(pattern)
                    }
                    CaseElement::Null => {
                        p.require(Feature::PatternSwitch)?;
                        p.text("null");
                        Ok(())
                    }
                    CaseElement::Default => {
                        p.require(Feature::PatternSwitch)?;
                        p.text("default");
                        Ok(())
                    }
                })?;
                if let Some(guard) = guard {
                    self.require(Feature::WhenGuards)?;
                    if pattern_count == 0 {
                        return Err(Self::invalid("a when guard requires a pattern case"));
                    }
                    self.text(" when ");
                    self.expr(guard, 2)?;
                }
            }
        }
        Ok(())
    }
}
