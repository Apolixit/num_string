use std::{fmt::Display};

#[derive(Debug, PartialEq)]
pub enum ConversionError {
    UnableToConvert,
    PatternCultureNotFound
}

impl ConversionError {
    pub fn message(&self) -> &str {
        match self {
            Self::UnableToConvert => "Error when trying to parse string number to number",
            Self::PatternCultureNotFound => "Unable to find pattern culture"
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}