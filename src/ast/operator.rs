#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitAnd,
    BitXor,
    BitOr,
    And,
    Or,
}

impl BinaryOp {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::UnsignedShiftRight => ">>>",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::BitAnd => "&",
            Self::BitXor => "^",
            Self::BitOr => "|",
            Self::And => "&&",
            Self::Or => "||",
        }
    }
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Or => 3,
            Self::And => 4,
            Self::BitOr => 5,
            Self::BitXor => 6,
            Self::BitAnd => 7,
            Self::Equal | Self::NotEqual => 8,
            Self::Less | Self::LessEqual | Self::Greater | Self::GreaterEqual => 9,
            Self::ShiftLeft | Self::ShiftRight | Self::UnsignedShiftRight => 10,
            Self::Add | Self::Subtract => 11,
            Self::Multiply | Self::Divide | Self::Remainder => 12,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    Complement,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

impl UnaryOp {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
            Self::Complement => "~",
            Self::PreIncrement | Self::PostIncrement => "++",
            Self::PreDecrement | Self::PostDecrement => "--",
        }
    }
    pub const fn is_postfix(self) -> bool {
        matches!(self, Self::PostIncrement | Self::PostDecrement)
    }
    pub const fn is_update(self) -> bool {
        matches!(
            self,
            Self::PreIncrement | Self::PreDecrement | Self::PostIncrement | Self::PostDecrement
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignOp {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    BitAnd,
    BitXor,
    BitOr,
}

impl AssignOp {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Add => "+=",
            Self::Subtract => "-=",
            Self::Multiply => "*=",
            Self::Divide => "/=",
            Self::Remainder => "%=",
            Self::ShiftLeft => "<<=",
            Self::ShiftRight => ">>=",
            Self::UnsignedShiftRight => ">>>=",
            Self::BitAnd => "&=",
            Self::BitXor => "^=",
            Self::BitOr => "|=",
        }
    }
}
