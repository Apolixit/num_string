use std::{fmt::Display};

/// The different kind of error which can happen during the conversion
#[derive(Debug, PartialEq)]
pub enum ConversionError {
    UnableToConvertStringToNumber,
    UnableToConvertNumberToString,
    NotCaptureFoundWhenConvertNumberToString,
    UnableToDisplayFormat,
    PatternCultureNotFound,
    SeparatorNotFound,
}

impl ConversionError {
    pub fn message(&self) -> &str {
        match self {
            Self::UnableToConvertStringToNumber => "Error when trying to parse string number to number",
            Self::UnableToConvertNumberToString => "Error when trying to parse number to string number",
            Self::NotCaptureFoundWhenConvertNumberToString => "No capture found when trying to parse number to string number",
            Self::UnableToDisplayFormat => "Error when trying to display format number",
            Self::PatternCultureNotFound => "Unable to find pattern culture",
            Self::SeparatorNotFound => "Unable to find separator from string"
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}