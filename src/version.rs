//! Syntax availability, including preview features that were subsequently withdrawn.
//! The modeled releases are Java 1.0 through Java 27. Future releases are not inferred.
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum JavaVersion {
    Java1_0 = 0,
    Java1_1 = 1,
    Java1_2 = 2,
    Java1_3 = 3,
    Java1_4 = 4,
    Java5 = 5,
    Java6 = 6,
    Java7 = 7,
    Java8 = 8,
    Java9 = 9,
    Java10 = 10,
    Java11 = 11,
    Java12 = 12,
    Java13 = 13,
    Java14 = 14,
    Java15 = 15,
    Java16 = 16,
    Java17 = 17,
    Java18 = 18,
    Java19 = 19,
    Java20 = 20,
    Java21 = 21,
    Java22 = 22,
    Java23 = 23,
    Java24 = 24,
    Java25 = 25,
    Java26 = 26,
    Java27 = 27,
}
impl JavaVersion {
    pub const LATEST: Self = Self::Java27;
    pub const ALL: &'static [Self] = &[
        Self::Java1_0,
        Self::Java1_1,
        Self::Java1_2,
        Self::Java1_3,
        Self::Java1_4,
        Self::Java5,
        Self::Java6,
        Self::Java7,
        Self::Java8,
        Self::Java9,
        Self::Java10,
        Self::Java11,
        Self::Java12,
        Self::Java13,
        Self::Java14,
        Self::Java15,
        Self::Java16,
        Self::Java17,
        Self::Java18,
        Self::Java19,
        Self::Java20,
        Self::Java21,
        Self::Java22,
        Self::Java23,
        Self::Java24,
        Self::Java25,
        Self::Java26,
        Self::Java27,
    ];
    pub const fn number(self) -> u8 {
        self as u8
    }
}
impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.number() < 5 {
            write!(f, "1.{}", self.number())
        } else {
            write!(f, "{}", self.number())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageLevel {
    pub version: JavaVersion,
    pub preview: bool,
}
impl LanguageLevel {
    pub const fn new(version: JavaVersion) -> Self {
        Self {
            version,
            preview: false,
        }
    }
    pub const fn preview(version: JavaVersion) -> Self {
        Self {
            version,
            preview: true,
        }
    }
    pub const fn supports(self, feature: Feature) -> bool {
        matches!(feature.status(self.version), FeatureStatus::Stable)
            || (self.preview && matches!(feature.status(self.version), FeatureStatus::Preview))
    }
}
impl Default for LanguageLevel {
    fn default() -> Self {
        Self::new(JavaVersion::LATEST)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureStatus {
    Unavailable,
    Preview,
    Stable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    InnerClasses,
    ClassLiterals,
    Strictfp,
    Assert,
    Generics,
    Annotations,
    Enums,
    Varargs,
    EnhancedFor,
    StaticImports,
    TryResources,
    MultiCatch,
    Diamond,
    Lambda,
    TypeAnnotations,
    IntersectionCast,
    InterfaceMethods,
    PrivateInterfaceMethods,
    Modules,
    ResourceReferences,
    AnonymousDiamond,
    Var,
    LambdaVar,
    SwitchExpressions,
    SwitchRules,
    Yield,
    BreakValue,
    TextBlocks,
    Records,
    InstanceOfPatterns,
    SealedTypes,
    PatternSwitch,
    WhenGuards,
    LegacyPatterns,
    ParenthesizedPatterns,
    RecordPatterns,
    RecordPatternBindings,
    RecordFor,
    UnnamedVariables,
    StringTemplates,
    CompactUnits,
    ModuleImports,
    FlexibleConstructors,
    PrimitivePatterns,
}
impl Feature {
    pub const ALL: &'static [Self] = &[
        Self::InnerClasses,
        Self::ClassLiterals,
        Self::Strictfp,
        Self::Assert,
        Self::Generics,
        Self::Annotations,
        Self::Enums,
        Self::Varargs,
        Self::EnhancedFor,
        Self::StaticImports,
        Self::TryResources,
        Self::MultiCatch,
        Self::Diamond,
        Self::Lambda,
        Self::TypeAnnotations,
        Self::IntersectionCast,
        Self::InterfaceMethods,
        Self::PrivateInterfaceMethods,
        Self::Modules,
        Self::ResourceReferences,
        Self::AnonymousDiamond,
        Self::Var,
        Self::LambdaVar,
        Self::SwitchExpressions,
        Self::SwitchRules,
        Self::Yield,
        Self::BreakValue,
        Self::TextBlocks,
        Self::Records,
        Self::InstanceOfPatterns,
        Self::SealedTypes,
        Self::PatternSwitch,
        Self::WhenGuards,
        Self::LegacyPatterns,
        Self::ParenthesizedPatterns,
        Self::RecordPatterns,
        Self::RecordPatternBindings,
        Self::RecordFor,
        Self::UnnamedVariables,
        Self::StringTemplates,
        Self::CompactUnits,
        Self::ModuleImports,
        Self::FlexibleConstructors,
        Self::PrimitivePatterns,
    ];
    pub const fn status(self, version: JavaVersion) -> FeatureStatus {
        let (stable, preview): (Option<u8>, Option<(u8, u8)>) = match self {
            Self::InnerClasses => (Some(1), None),
            Self::ClassLiterals => (Some(1), None),
            Self::Strictfp => (Some(2), None),
            Self::Assert => (Some(4), None),
            Self::Generics => (Some(5), None),
            Self::Annotations => (Some(5), None),
            Self::Enums => (Some(5), None),
            Self::Varargs => (Some(5), None),
            Self::EnhancedFor => (Some(5), None),
            Self::StaticImports => (Some(5), None),
            Self::TryResources => (Some(7), None),
            Self::MultiCatch => (Some(7), None),
            Self::Diamond => (Some(7), None),
            Self::Lambda => (Some(8), None),
            Self::TypeAnnotations => (Some(8), None),
            Self::IntersectionCast => (Some(8), None),
            Self::InterfaceMethods => (Some(8), None),
            Self::PrivateInterfaceMethods => (Some(9), None),
            Self::Modules => (Some(9), None),
            Self::ResourceReferences => (Some(9), None),
            Self::AnonymousDiamond => (Some(9), None),
            Self::Var => (Some(10), None),
            Self::LambdaVar => (Some(11), None),
            Self::SwitchExpressions => (Some(14), Some((12, 13))),
            Self::SwitchRules => (Some(14), Some((12, 13))),
            Self::Yield => (Some(14), Some((13, 13))),
            Self::BreakValue => (None, Some((12, 12))),
            Self::TextBlocks => (Some(15), Some((13, 14))),
            Self::Records => (Some(16), Some((14, 15))),
            Self::InstanceOfPatterns => (Some(16), Some((14, 15))),
            Self::SealedTypes => (Some(17), Some((15, 16))),
            Self::PatternSwitch => (Some(21), Some((17, 20))),
            Self::WhenGuards => (Some(21), Some((19, 20))),
            Self::LegacyPatterns => (None, Some((17, 18))),
            Self::ParenthesizedPatterns => (None, Some((17, 20))),
            Self::RecordPatterns => (Some(21), Some((19, 20))),
            Self::RecordPatternBindings => (None, Some((19, 19))),
            Self::RecordFor => (None, Some((20, 20))),
            Self::UnnamedVariables => (Some(22), Some((21, 21))),
            Self::StringTemplates => (None, Some((21, 22))),
            Self::CompactUnits => (Some(25), Some((21, 24))),
            Self::ModuleImports => (Some(25), Some((23, 24))),
            Self::FlexibleConstructors => (Some(25), Some((22, 24))),
            Self::PrimitivePatterns => (None, Some((23, 27))),
        };
        let release = version.number();
        if let Some(first) = stable
            && release >= first
        {
            return FeatureStatus::Stable;
        }
        if let Some((first, last)) = preview
            && release >= first
            && release <= last
        {
            return FeatureStatus::Preview;
        }
        FeatureStatus::Unavailable
    }
}
