use errors::ConversionError;
use log::warn;
use num::Num;
use pattern::{CulturePattern, NumberType, ParsingPattern, Patterns};

mod errors;
mod number_conversion;
mod pattern;

/// Represent the current "ConvertString" culture
#[derive(PartialEq, Debug, Clone)]
pub enum Culture {
    English,
    French,
}

/// Default culture = English
impl Default for &Culture {
    fn default() -> Self {
        &Culture::English
    }
}

/// Main struct
pub struct ConvertString {
    string_num: String,
    culture: Option<Culture>,
    all_patterns: Patterns,
}

impl ConvertString {
    /// Create a new ConvertString instance
    pub fn new(string_num: &str, culture: Option<Culture>) -> ConvertString {
        ConvertString {
            string_num: String::from(string_num),
            culture,
            all_patterns: ConvertString::load_patterns(),
        }
    }

    /// Load all patterns
    fn load_patterns() -> Patterns {
        Patterns::default()
    }

    /// Return the pattern selected for conversion
    fn get_current_pattern(&self) -> Option<ParsingPattern> {
        ConvertString::find_pattern(
            &self.string_num,
            self.culture.as_ref().unwrap_or_default(),
            &self.all_patterns,
        )
    }

    /// Get all parsing pattern from culture
    // pub fn find_culture_parsing_pattern(
    //     culture: &Culture,
    //     patterns: &Patterns,
    // ) -> Result<Vec<ParsingPattern>, ConversionError> {
    //     let culture_pattern = ConvertString::find_culture_pattern(&culture, &patterns)?;
    //     Ok(culture_pattern.get_patterns().to_vec())
    // }

    /// Get culture pattern from culture
    pub fn find_culture_pattern(culture: &Culture, patterns: &Patterns) -> Option<CulturePattern> {
        patterns
            .get_culture_pattern()
            .into_iter()
            .find(|c| c.get_cultures().iter().any(|cc| cc == culture))
    }

    pub fn find_pattern(
        string_num: &str,
        culture: &Culture,
        patterns: &Patterns,
    ) -> Option<ParsingPattern> {
        //First, we search in common pattern (not currency dependent) and currency pattern
        let mut all_patterns = patterns.get_common_pattern();

        let pattern_culture = ConvertString::find_culture_pattern(&culture, &patterns);

        if pattern_culture.is_none() {
            warn!("{}", ConversionError::PatternCultureNotFound.message());
        } else {
            all_patterns.extend(pattern_culture.unwrap().get_patterns().clone());
        }

        // Return the pattern which match
        all_patterns
            .into_iter()
            .find(|p| p.regex.is_match(string_num))
    }

    /// Return true is the string has been succesfully converted into number
    pub fn is_numeric(&self) -> bool {
        self.get_current_pattern().is_some()
    }

    /// Return true is the string has been succesfully converted into an integer
    pub fn is_integer(&self) -> bool {
        if let Some(pp) = self.get_current_pattern() {
            return pp.number_type == NumberType::WHOLE;
        }

        false
    }

    /// Return true is the string has been succesfully converted into a float
    pub fn is_float(&self) -> bool {
        if let Some(pp) = self.get_current_pattern() {
            return pp.number_type == NumberType::DECIMAL;
        }

        false
    }

    /// Convert the string into an integer
    pub fn to_integer(&self) -> Option<i32> {
        if let Some(pp) = self.get_current_pattern() {
            return pp.to_integer(String::from(&self.string_num)).map(|n| n.num);
        }

        None
    }

    /// Convert the string into an float
    pub fn to_float(&self) -> Option<f32> {
        if let Some(pp) = self.get_current_pattern() {
            return pp.to_float(String::from(&self.string_num)).map(|n| n.num);
        }

        None
    }
}

pub struct FormatOption {
    minimum_fraction_digit: u8,
    maximum_fraction_digit: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number<T: Num> {
    num: T,
}

impl<T: num::Num> Number<T> {
    pub fn new(num: T) -> Number<T> {
        Number { num }
    }

    pub fn to_format() {
        todo!()
    }

    pub fn to_format_options(options: FormatOption) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ConvertString;

    #[test]
    fn test_common_number() {
        let convert = ConvertString::new("10,2", None);
        // assert_eq!(convert.to_integer(), 10);
        assert!(convert.is_integer());
    }
}
