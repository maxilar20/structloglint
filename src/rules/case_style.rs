use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    CamelCase,
    ConstantCase,
    KebabCase,
    LowerCase,
    PascalCase,
    SentenceCase,
    SnakeCase,
    TitleCase,
    TrainCase,
    UpperCase,
}

impl CaseStyle {
    pub const ALL: &[CaseStyle] = &[
        Self::CamelCase,
        Self::ConstantCase,
        Self::KebabCase,
        Self::LowerCase,
        Self::PascalCase,
        Self::SentenceCase,
        Self::SnakeCase,
        Self::TitleCase,
        Self::TrainCase,
        Self::UpperCase,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CamelCase => "camelCase",
            Self::ConstantCase => "CONSTANT_CASE",
            Self::KebabCase => "kebab-case",
            Self::LowerCase => "lower case",
            Self::PascalCase => "PascalCase",
            Self::SentenceCase => "sentence case",
            Self::SnakeCase => "snake_case",
            Self::TitleCase => "Title Case",
            Self::TrainCase => "Train-Case",
            Self::UpperCase => "UPPER CASE",
        }
    }

    pub fn convert(&self, s: &str) -> String {
        use inflections::case;
        match self {
            Self::CamelCase => case::to_camel_case(s),
            Self::ConstantCase => case::to_constant_case(s),
            Self::KebabCase => case::to_kebab_case(s),
            Self::LowerCase => case::to_lower_case(s),
            Self::PascalCase => case::to_pascal_case(s),
            Self::SentenceCase => case::to_sentence_case(s),
            Self::SnakeCase => case::to_snake_case(s),
            Self::TitleCase => case::to_title_case(s),
            Self::TrainCase => case::to_train_case(s),
            Self::UpperCase => case::to_upper_case(s),
        }
    }

    pub fn is_match(&self, s: &str) -> bool {
        use inflections::case;
        match self {
            Self::CamelCase => case::is_camel_case(s),
            Self::ConstantCase => case::is_constant_case(s),
            Self::KebabCase => case::is_kebab_case(s),
            Self::LowerCase => case::is_lower_case(s),
            Self::PascalCase => case::is_pascal_case(s),
            Self::SentenceCase => case::is_sentence_case(s),
            Self::SnakeCase => case::is_snake_case(s),
            Self::TitleCase => case::is_title_case(s),
            Self::TrainCase => case::is_train_case(s),
            Self::UpperCase => case::is_upper_case(s),
        }
    }
}

impl FromStr for CaseStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.iter().find(|c| c.as_str() == s).copied().ok_or(())
    }
}

impl fmt::Display for CaseStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
