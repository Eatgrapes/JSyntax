/// Java strings contain UTF-16 code units, including isolated surrogates.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct JavaString(Vec<u16>);

impl JavaString {
    pub fn from_utf16(units: impl Into<Vec<u16>>) -> Self {
        Self(units.into())
    }
    pub fn as_utf16(&self) -> &[u16] {
        &self.0
    }
    pub fn into_utf16(self) -> Vec<u16> {
        self.0
    }
}

impl From<&str> for JavaString {
    fn from(value: &str) -> Self {
        Self(value.encode_utf16().collect())
    }
}

impl From<String> for JavaString {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Char(u16),
    String(JavaString),
    /// Logical value, not raw source; indentation and escapes are handled by the emitter.
    TextBlock(JavaString),
}
