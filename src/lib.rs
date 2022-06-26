use crate::number_conversion::NumberConversion;
use errors::ConversionError;
use log::{error, info, trace, warn};
use num::Num;
use pattern::{
    CulturePattern, NumberCultureSettings, NumberType, ParsingPattern, Patterns, Separator,
};
use regex::Regex;
use std::fmt::Display;
use std::str::FromStr;

mod errors;
mod number;
mod number_conversion;
mod pattern;

/// Represent the current "ConvertString" culture
#[derive(PartialEq, Debug, Clone, Copy)]
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

/// Structure to convert a string to number
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

    /// Get culture pattern from culture
    pub fn find_culture_pattern(culture: &Culture, patterns: &Patterns) -> Option<CulturePattern> {
        patterns
            .get_all_culture_pattern()
            .into_iter()
            .find(|c| c.get_cultures().iter().any(|cc| cc == culture))
    }

    /// Find a matching pattern for the given string num
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
            .find(|p| p.get_regex().is_match(string_num))
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
            return pp.get_number_type() == &NumberType::WHOLE;
        }

        false
    }

    /// Return true is the string has been succesfully converted into a float
    pub fn is_float(&self) -> bool {
        if let Some(pp) = self.get_current_pattern() {
            return pp.get_number_type() == &NumberType::DECIMAL;
        }

        false
    }

    pub fn to_number<N: num::Num + Display + FromStr>(&self) -> Result<N, ConversionError> {
        if let Some(culture) = self.culture {
            self.string_num.as_str().to_number_culture::<N>(culture)
        } else {
            self.string_num.as_str().to_number::<N>()
        }
    }
}

#[derive(Debug)]
pub struct FormatOption {
    minimum_fraction_digit: u8,
    maximum_fraction_digit: u8,
}

impl FormatOption {
    pub fn new(minimum_fraction_digit: u8, maximum_fraction_digit: u8) -> FormatOption {
        FormatOption {
            minimum_fraction_digit,
            maximum_fraction_digit,
        }
    }
}

impl Default for FormatOption {
    fn default() -> Self {
        Self {
            minimum_fraction_digit: 2,
            maximum_fraction_digit: 2,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        errors::ConversionError, number::ToFormat, number_conversion::NumberConversion,
        pattern::NumberType, ConvertString, Culture, FormatOption,
    };
    use log::{info, warn};

    // Run this function before each test
    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_reverse_mapping_number() {
        let values_int = vec![(1, "1", Culture::French), (1000, "1 000", Culture::French)];

        for (val_i32, val_str, culture) in values_int {
            assert_eq!(val_i32.to_format("N0", &culture).unwrap(), val_str);
            assert_eq!(val_str.to_number_culture::<i32>(culture).unwrap(), val_i32);
        }

        let values_float = vec![
            (1.0, "1,00", Culture::French),
            (1000.88, "1 000,88", Culture::French),
            (-1582.99, "-1,582.99", Culture::English),
            (1.0, "1.00", Culture::English),
            (100000000.10, "100.000.000,10", Culture::Italian),
            (-50.50, "-50,50", Culture::Italian),
        ];

        for (val_f64, val_str, culture) in values_float {
            assert_eq!(val_f64.to_format("N2", &culture).unwrap(), val_str);
            assert_eq!(val_str.to_number_culture::<f64>(culture).unwrap(), val_f64);
        }
    }

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

    #[test]
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
                "1.000,4564654654654",
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

            let to_integer = convert.to_number::<i32>();
            if number_type == NumberType::WHOLE {
                assert!(to_integer.is_ok(), "to_number() return Err instead of Ok");
                assert_eq!(
                    convert.to_number::<i32>().unwrap(),
                    int_value,
                    "to_integer() conversion failed for {}",
                    string_num
                );
            } else {
                assert!(to_integer.is_err(), "to_number() return Ok instead of Err");
                assert_eq!(
                    convert.to_number::<i32>(),
                    Err(ConversionError::UnableToConvertStringToNumber)
                );
            }

            let to_float = convert.to_number::<f32>();
            assert!(to_float.is_ok(), "to_float() return Err instead of Ok");
            assert_eq!(
                convert.to_number::<f32>().unwrap(),
                float_value,
                "to_float() conversion failed for {}",
                string_num
            );
        }
    }

    #[test]
    fn test_number_unauthorized() {
        let list = vec!["1..0", "1.,0", ",1.0", "+-0.2", "20 00", "-0,2245,45"];
        let cultures = &vec![
            None,
            Some(Culture::English),
            Some(Culture::French),
            Some(Culture::Italian),
        ];

        for string_num in list {
            for culture in cultures.into_iter() {
                let convert = ConvertString::new(string_num, culture.to_owned());
                assert_eq!(convert.is_numeric(), false, "Numeric shouldn't be parsed");
            }
        }
    }

    #[test]
    pub fn test_number_to_format_integer() {
        let integers = vec![
            (2000i64, Culture::English, "2,000"),
            (2000, Culture::French, "2 000"),
            (2000, Culture::Italian, "2.000"),
            (-2000, Culture::English, "-2,000"),
            (-2000, Culture::French, "-2 000"),
            (-2000, Culture::Italian, "-2.000"),
        ];

        for (number, culture, to_string_format) in integers {
            assert_eq!(
                number.to_format("N0", &culture).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    #[test]
    pub fn test_number_to_format_float() {
        let floats = vec![
            (2_000.98, Culture::English, "2,000.98"),
            (-2_000.98, Culture::French, "-2 000,98"),
            (2_000.98, Culture::Italian, "2.000,98"),
            (049_490.8257, Culture::English, "49,490.83"),
        ];

        for (number, culture, to_string_format) in floats {
            assert_eq!(
                number.to_format("N2", &culture).unwrap(),
                String::from(to_string_format)
            );
        }
    }
}

// macro_rules! impl_from {
//     ($T:ty, $from_ty:path) => {
//         impl From<$T> for Decimal {
//             #[inline]
//             fn from(t: $T) -> Decimal {
//                 $from_ty(t).unwrap()
//             }
//         }
//     }
// }

// impl_from!(isize, FromPrimitive::from_isize);
// impl_from!(i8, FromPrimitive::from_i8);
// impl_from!(i16, FromPrimitive::from_i16);
// impl_from!(i32, FromPrimitive::from_i32);
// impl_from!(i64, FromPrimitive::from_i64);
// impl_from!(usize, FromPrimitive::from_usize);
// impl_from!(u8, FromPrimitive::from_u8);
// impl_from!(u16, FromPrimitive::from_u16);
// impl_from!(u32, FromPrimitive::from_u32);
// impl_from!(u64, FromPrimitive::from_u64);
