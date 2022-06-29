use std::{fmt::Display};

/// The different kind of error which can happen during the conversion
#[derive(Debug, PartialEq)]
pub enum ConversionError {
    /// When trying to convert the string. This error happen when after cleaned the input the core::str::parse() function return a conversion error
    UnableToConvertStringToNumber,

    /// When the regex cannot parse the number
    UnableToConvertNumberToString,

    /// Error linked to UnableToConvertNumberToString, it happens when the number has been parsed but no match captures were found
    NotCaptureFoundWhenConvertNumberToString,

    /// The format (should be N0 / N2 / N9) is not well formatted
    UnableToDisplayFormat,

    /// The culture pattern has not been implemented
    PatternCultureNotFound,

    /// Try to create a separator from string but it does not exist in the enum
    SeparatorNotFound,

    /// When the dynamic regex generation fail (automatically build from culture and type parsing)
    RegexBuilder
}

impl ConversionError {
    pub fn message(&self) -> &str {
        match self {
            Self::UnableToConvertStringToNumber => "Error when trying to parse string number to number",
            Self::UnableToConvertNumberToString => "Error when trying to parse number to string number",
            Self::NotCaptureFoundWhenConvertNumberToString => "No capture found when trying to parse number to string number",
            Self::UnableToDisplayFormat => "Error when trying to display format number",
            Self::PatternCultureNotFound => "Unable to find pattern culture",
            Self::SeparatorNotFound => "Unable to find separator from string",
            Self::RegexBuilder => "Unable to create regex",
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}