#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
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
