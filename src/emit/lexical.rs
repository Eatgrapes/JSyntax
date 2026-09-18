use super::{ErrorKind, Printer, Result};
use crate::ast::{Comment, JavaString, Literal};
use crate::version::Feature;
use std::fmt::Write;
use unicode_general_category::{GeneralCategory as Category, get_general_category};

impl Printer<'_> {
    pub(super) fn identifier(&mut self, value: &str) -> Result {
        let mut chars = value.chars();
        let valid = chars.next().is_some_and(identifier_start) && chars.all(identifier_part);
        let version = self.options.language.version.number();
        let reserved = matches!(
            value,
            "abstract"
                | "boolean"
                | "break"
                | "byte"
                | "case"
                | "catch"
                | "char"
                | "class"
                | "const"
                | "continue"
                | "default"
                | "do"
                | "double"
                | "else"
                | "extends"
                | "final"
                | "finally"
                | "float"
                | "for"
                | "goto"
                | "if"
                | "implements"
                | "import"
                | "instanceof"
                | "int"
                | "interface"
                | "long"
                | "native"
                | "new"
                | "package"
                | "private"
                | "protected"
                | "public"
                | "return"
                | "short"
                | "static"
                | "super"
                | "switch"
                | "synchronized"
                | "this"
                | "throw"
                | "throws"
                | "transient"
                | "try"
                | "void"
                | "volatile"
                | "while"
                | "true"
                | "false"
                | "null"
        ) || (value == "strictfp" && version >= 2)
            || (value == "assert" && version >= 4)
            || (value == "enum" && version >= 5)
            || (value == "_" && version >= 9);
        if !valid || reserved {
            return Err(Self::error(ErrorKind::InvalidIdentifier(value.into())));
        }
        self.text(value);
        Ok(())
    }

    pub(super) fn binding(&mut self, value: &str, unnamed: bool) -> Result {
        if value == "_" && unnamed && self.options.language.version.number() >= 9 {
            self.require(Feature::UnnamedVariables)?;
            self.text("_");
            Ok(())
        } else {
            self.identifier(value)
        }
    }

    pub(super) fn type_identifier(&mut self, value: &str) -> Result {
        let level = self.options.language;
        if (value == "var" && level.version.number() >= 10)
            || (value == "yield" && level.supports(Feature::Yield))
            || (value == "record" && level.supports(Feature::Records))
            || (matches!(value, "sealed" | "permits") && level.supports(Feature::SealedTypes))
        {
            return Err(Self::error(ErrorKind::InvalidIdentifier(value.into())));
        }
        self.identifier(value)
    }

    pub(super) fn qualified(&mut self, name: &str, type_name: bool) -> Result {
        let mut parts = name.split('.').peekable();
        while let Some(part) = parts.next() {
            if type_name && parts.peek().is_none() {
                self.type_identifier(part)?;
            } else {
                self.identifier(part)?;
            }
            if parts.peek().is_some() {
                self.text(".");
            }
        }
        Ok(())
    }

    pub(super) fn comments(&mut self, comments: &[Comment]) -> Result {
        for comment in comments {
            let (text, opening) = match comment {
                Comment::Line(text) => (text, "//"),
                Comment::Block(text) => (text, "/*"),
                Comment::Javadoc(text) => (text, "/**"),
            };
            if text.contains("\\u") || (opening != "//" && text.contains("*/")) {
                return Err(Self::error(ErrorKind::InvalidComment));
            }
            if opening == "//" {
                for line in text.split(['\n', '\r']) {
                    self.text("// ");
                    self.text(line);
                    self.newline();
                }
            } else {
                self.text(opening);
                self.newline();
                for line in text.split(['\n', '\r']) {
                    self.text(" * ");
                    self.text(line);
                    self.newline();
                }
                self.text(" */");
                self.newline();
            }
        }
        Ok(())
    }

    pub(super) fn literal(&mut self, literal: &Literal) -> Result {
        match literal {
            Literal::Null => self.text("null"),
            Literal::Boolean(value) => self.text(if *value { "true" } else { "false" }),
            Literal::Int(value) => self.text(&value.to_string()),
            Literal::Long(value) => self.text(&format!("{value}L")),
            Literal::Float(value) => self.floating(f64::from(*value), true, &value.to_string()),
            Literal::Double(value) => self.floating(*value, false, &value.to_string()),
            Literal::Char(value) => {
                self.text("'");
                self.escaped(&JavaString::from_utf16(vec![*value]), true, false);
                self.text("'");
            }
            Literal::String(value) => {
                self.text("\"");
                self.escaped(value, false, false);
                self.text("\"");
            }
            Literal::TextBlock(value) => {
                self.require(Feature::TextBlocks)?;
                self.text("\"\"\"");
                self.newline();
                self.escaped(value, false, true);
                self.text("\"\"\"");
            }
        }
        Ok(())
    }

    fn floating(&mut self, value: f64, single: bool, digits: &str) {
        let suffix = if single { "F" } else { "D" };
        if value.is_nan() {
            self.text(&format!("(0.0{suffix} / 0.0{suffix})"));
        } else if value.is_infinite() {
            self.text(&format!(
                "({}1.0{suffix} / 0.0{suffix})",
                if value.is_sign_negative() { "-" } else { "" }
            ));
        } else {
            self.text(digits);
            if !digits.contains(['.', 'e', 'E']) {
                self.text(".0");
            }
            self.text(suffix);
        }
    }

    pub(super) fn escaped(&mut self, value: &JavaString, character: bool, multiline: bool) {
        for &unit in value.as_utf16() {
            match unit {
                8 => self.text("\\b"),
                9 => self.text("\\t"),
                10 if multiline => self.newline(),
                10 => self.text("\\n"),
                12 => self.text("\\f"),
                13 => self.text("\\r"),
                34 => self.text("\\\""),
                39 if character => self.text("\\'"),
                92 => self.text("\\\\"),
                32 if multiline => self.text("\\040"),
                0..=31 | 127 => self.text(&format!("\\{unit:03o}")),
                32..=126 => self.text(&char::from(unit as u8).to_string()),
                _ => {
                    let mut escape = String::with_capacity(6);
                    let _ = write!(escape, "\\u{unit:04X}");
                    self.text(&escape);
                }
            }
        }
    }
}

fn identifier_start(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        Category::UppercaseLetter
            | Category::LowercaseLetter
            | Category::TitlecaseLetter
            | Category::ModifierLetter
            | Category::OtherLetter
            | Category::LetterNumber
            | Category::CurrencySymbol
            | Category::ConnectorPunctuation
    )
}

fn identifier_part(ch: char) -> bool {
    identifier_start(ch)
        || matches!(
            get_general_category(ch),
            Category::DecimalNumber
                | Category::NonspacingMark
                | Category::SpacingMark
                | Category::Format
        )
        || matches!(ch, '\u{0}'..='\u{8}' | '\u{e}'..='\u{1b}' | '\u{7f}'..='\u{9f}')
}
