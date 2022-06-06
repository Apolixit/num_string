use errors::ConversionError;
use log::{info, warn};
use num::Num;
use pattern::{CulturePattern, NumberType, ParsingPattern, Patterns};
use std::fmt::Display;

mod errors;
mod number_conversion;
mod pattern;

/// Represent the current "ConvertString" culture
#[derive(PartialEq, Debug, Clone)]
pub enum Culture {
    English,
    French,
    Italian,
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
            .get_all_culture_pattern()
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
        match all_patterns
            .into_iter()
            .find(|p| p.regex.is_match(string_num))
        {
            Some(pp) => {
                info!("Input = {} / Pattern found = {}", &string_num, &pp);
                return Some(pp);
            }
            None => {
                info!("No Pattern found for '{}'", &string_num);
                return None;
            }
        }
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
    pub fn to_integer(&self) -> Option<Number<i32>> {
        if let Some(pp) = self.get_current_pattern() {
            return pp.to_integer(String::from(&self.string_num));
        }

        None
    }

    /// Convert the string into an float
    pub fn to_float(&self) -> Option<Number<f32>> {
        if let Some(pp) = self.get_current_pattern() {
            return pp.to_float(String::from(&self.string_num));
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

impl<T: num::Num> PartialEq<T> for Number<T> {
    fn eq(&self, other: &T) -> bool {
        &self.num == other
    }
}

impl<T: num::Num + Display> Display for Number<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.num)
    }
}

#[cfg(test)]
mod tests {
    use log::{info, warn};

    use crate::{pattern::NumberType, ConvertString, Culture};

    #[test]
    fn test_common_number() {
        let convert = ConvertString::new("10,2", Some(Culture::French));
        assert!(convert.is_float());
    }

    #[test]
    fn test_number_french() {
        let list = vec![
            ("10", 10, 10.0, NumberType::WHOLE),
            ("+10", 10, 10.0, NumberType::WHOLE),
            ("-102", -102, -102., NumberType::WHOLE),
            ("+1 000", 1000, 1000.0, NumberType::WHOLE),
            ("2 500 563", 2500563, 2_500_563.0, NumberType::WHOLE),
            ("-200000", -200000, -200000.0, NumberType::WHOLE),
            (",25", 0, 0.25, NumberType::DECIMAL),
            ("10,2", 10, 10.2, NumberType::DECIMAL),
            ("0,25", 0, 0.25, NumberType::DECIMAL),
            ("-10,5", -10, -10.5, NumberType::DECIMAL),
            ("1000,89", 1000, 1000.89, NumberType::DECIMAL),
            (
                "+1 000,4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
            (
                "1000,4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
        ];
        test_number(Some(Culture::French), list);
    }

    #[test]
    fn test_number_english() {
        let list = vec![
            ("10", 10, 10.0, NumberType::WHOLE),
            ("-102", -102, -102., NumberType::WHOLE),
            ("1,000", 1000, 1000.0, NumberType::WHOLE),
            ("-200000", -200000, -200000.0, NumberType::WHOLE),
            ("2,500,563", 2500563, 2_500_563.0, NumberType::WHOLE),
            ("10.2", 10, 10.2, NumberType::DECIMAL),
            ("0.4", 0, 0.4, NumberType::DECIMAL),
            ("0.25", 0, 0.25, NumberType::DECIMAL),
            ("-10.5", -10, -10.5, NumberType::DECIMAL),
            ("1000.89", 1000, 1000.89, NumberType::DECIMAL),
            (
                "1,000.4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
            (
                "1000.4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
        ];
        test_number(Some(Culture::English), list);
    }

    fn test_number_italian() {
        let list = vec![
            ("10", 10, 10.0, NumberType::WHOLE),
            ("-102", -102, -102., NumberType::WHOLE),
            ("1.000", 1000, 1000.0, NumberType::WHOLE),
            ("-200000", -200000, -200000.0, NumberType::WHOLE),
            ("2.500.563", 2500563, 2_500_563.0, NumberType::WHOLE),
            ("10,2", 10, 10.2, NumberType::DECIMAL),
            ("0,4", 0, 0.4, NumberType::DECIMAL),
            ("0,25", 0, 0.25, NumberType::DECIMAL),
            ("-10,5", -10, -10.5, NumberType::DECIMAL),
            ("1000,89", 1000, 1000.89, NumberType::DECIMAL),
            (
                "1.000,4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
            (
                "1000,4564654654654",
                1000,
                1000.4564654654654,
                NumberType::DECIMAL,
            ),
        ];
        test_number(Some(Culture::Italian), list);
    }

    fn test_number(culture: Option<Culture>, list: Vec<(&str, i32, f32, NumberType)>) {
        for (string_num, int_value, float_value, number_type) in list {
            let convert = ConvertString::new(string_num, culture.to_owned());
            //All input are valid number
            assert_eq!(convert.is_numeric(), true, "Numeric number expected");
            assert_eq!(
                convert.is_integer(),
                number_type == NumberType::WHOLE,
                "Number should be a {}",
                if number_type == NumberType::WHOLE {
                    "integer"
                } else {
                    "decimal"
                }
            );
            assert_eq!(
                convert.is_float(),
                number_type == NumberType::DECIMAL,
                "Number should be a {}",
                if number_type == NumberType::WHOLE {
                    "integer"
                } else {
                    "float"
                }
            );
            let to_integer = convert.to_integer();
            assert!(
                to_integer.is_some(),
                "to_integer() return none instead of some"
            );

            let to_float = convert.to_float();
            assert!(to_float.is_some(), "to_float() return none instead of some");

            assert_eq!(
                convert.to_integer().unwrap(),
                int_value,
                "to_integer() conversion failed for {}",
                string_num
            );
            assert_eq!(
                convert.to_float().unwrap(),
                float_value,
                "to_float() conversion failed for {}",
                string_num
            );

            info!(
                "String input = {} / Cleaned input = {}",
                string_num,
                convert.to_float().unwrap()
            )
        }
    }

    #[test]
    fn test_number_unauthorized() {
        let list = vec![
            "1..0", "1.,0", ",1.0", "+-0.2", "20 00", "-0,2245,45"
        ];
        let cultures = &vec![None, Some(Culture::English), Some(Culture::French), Some(Culture::Italian)];

        for string_num in list {
            for culture in cultures.into_iter() {
                let convert = ConvertString::new(string_num, culture.to_owned());
                assert_eq!(convert.is_numeric(), false, "Numeric shouldn't be parsed");
            }
        }
    }
}
