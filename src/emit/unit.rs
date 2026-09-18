use super::declaration::BodyKind;
use super::{Printer, Result};
use crate::ast::*;
use crate::version::Feature;

impl Printer<'_> {
    pub(super) fn compilation_unit(&mut self, unit: &CompilationUnit) -> Result {
        self.comments(&unit.comments)?;
        if let Some(package) = &unit.package {
            self.annotations(&package.annotations, true)?;
            self.text("package ");
            self.qualified(&package.name, false)?;
            self.text(";");
            self.newline();
            self.newline();
        }
        self.imports(&unit.imports)?;
        for (index, declaration) in unit.types.iter().enumerate() {
            if declaration.meta.modifiers.iter().any(|m| {
                matches!(
                    m,
                    Modifier::Private | Modifier::Protected | Modifier::Static
                )
            }) {
                return Err(Self::invalid(
                    "top-level types cannot be private, protected or static",
                ));
            }
            if index != 0 {
                self.newline();
            }
            self.type_decl(declaration)?;
        }
        Ok(())
    }

    pub(super) fn compact_unit(&mut self, unit: &CompactUnit) -> Result {
        self.require(Feature::CompactUnits)?;
        self.comments(&unit.comments)?;
        self.imports(&unit.imports)?;
        self.members(&unit.members, BodyKind::Compact, None)
    }

    pub(super) fn imports(&mut self, imports: &[Import]) -> Result {
        for import in imports {
            self.text("import ");
            match import {
                Import::Type(name) => self.qualified(name, true)?,
                Import::OnDemand(name) => {
                    self.qualified(name, false)?;
                    self.text(".*");
                }
                Import::Static { owner, member } => {
                    self.require(Feature::StaticImports)?;
                    self.text("static ");
                    self.qualified(owner, true)?;
                    self.text(".");
                    if let Some(member) = member {
                        self.identifier(member)?;
                    } else {
                        self.text("*");
                    }
                }
                Import::Module(name) => {
                    self.require(Feature::ModuleImports)?;
                    self.text("module ");
                    self.qualified(name, false)?;
                }
            }
            self.text(";");
            self.newline();
        }
        if !imports.is_empty() {
            self.newline();
        }
        Ok(())
    }

    pub(super) fn module_unit(&mut self, unit: &ModuleUnit) -> Result {
        self.require(Feature::Modules)?;
        self.imports(&unit.imports)?;
        self.module(&unit.declaration)
    }

    pub(super) fn module(&mut self, module: &ModuleDecl) -> Result {
        self.require(Feature::Modules)?;
        self.comments(&module.comments)?;
        self.annotations(&module.annotations, true)?;
        if module.open {
            self.text("open ");
        }
        self.text("module ");
        self.qualified(&module.name, false)?;
        self.text(" ");
        self.open_block();
        for directive in &module.directives {
            match directive {
                ModuleDirective::Requires {
                    name,
                    static_,
                    transitive,
                } => {
                    self.text("requires ");
                    if *static_ {
                        self.text("static ");
                    }
                    if *transitive {
                        self.text("transitive ");
                    }
                    self.qualified(name, false)?;
                }
                ModuleDirective::Exports { package, to }
                | ModuleDirective::Opens { package, to } => {
                    let opens = matches!(directive, ModuleDirective::Opens { .. });
                    if module.open && opens {
                        return Err(Self::invalid(
                            "an open module cannot contain opens directives",
                        ));
                    }
                    self.text(if opens { "opens " } else { "exports " });
                    self.qualified(package, false)?;
                    if !to.is_empty() {
                        self.text(" to ");
                        self.separated(to, ", ", |p, name| p.qualified(name, false))?;
                    }
                }
                ModuleDirective::Uses(service) => {
                    self.text("uses ");
                    self.qualified(service, true)?;
                }
                ModuleDirective::Provides {
                    service,
                    implementations,
                } => {
                    if implementations.is_empty() {
                        return Err(Self::invalid(
                            "provides requires at least one implementation",
                        ));
                    }
                    self.text("provides ");
                    self.qualified(service, true)?;
                    self.text(" with ");
                    self.separated(implementations, ", ", |p, name| p.qualified(name, true))?;
                }
            }
            self.text(";");
            self.newline();
        }
        self.close_block();
        self.newline();
        Ok(())
    }
}
