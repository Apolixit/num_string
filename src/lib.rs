//! Perform conversion between string and number
//!
//! It allows to convert culture formated number to rust number
//!
//! It allows to display rust numbers to culture formated string
//!
//! ## Example string to number
//!
//! ### Basic string to number
//!
//! ``` rust
//! use num_string::{Culture, ConversionError, NumberConversion};
//!     assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
//!     assert_eq!("+1000".to_number::<i64>().unwrap(), 1000);
//!     assert_eq!("-1000".to_number::<i64>().unwrap(), -1000);
//!     assert_eq!("1000".to_number::<f32>().unwrap(), 1000.0);
//!     assert_eq!("1000.5822".to_number::<f32>().unwrap(), 1000.5822);
//!
//!     // Fail because 1000 > i8 max capacity
//!     assert_eq!("1000".to_number::<i8>(), Err(ConversionError::UnableToConvertStringToNumber));
//! ```
//!
//! ### For more advanced conversion you can specify culture
//!
//! ``` rust
//! use num_string::{Culture, NumberConversion};
//!     // Numbers with decimal separator
//!     assert_eq!("10.8888".to_number_culture::<f32>(Culture::English).unwrap(), 10.8888);
//!     assert_eq!("0,10".to_number_culture::<f32>(Culture::Italian).unwrap(), 0.1);
//!
//!     // Numbers with decimal separator and no whole part
//!     assert_eq!(",10".to_number_culture::<f32>(Culture::Italian).unwrap(), 0.1);
//!
//!     // Numbers with thousand separator
//!     assert_eq!("1,000".to_number_culture::<i32>(Culture::English).unwrap(), 1000);
//!
//!     // Numbers with thousand and decimal separator
//!     assert_eq!("1,000.8888".to_number_culture::<f32>(Culture::English).unwrap(), 1000.8888);
//!     assert_eq!("-10 564,10".to_number_culture::<f32>(Culture::French).unwrap(), -10564.10);
//! ```
//!
//! ### Custom separator (DOT as thousand separator and SPACE a decimal separator)
//!
//! ``` rust
//! use num_string::{NumberCultureSettings, Separator, NumberConversion, ThousandGrouping};
//!
//!     assert_eq!(
//!             "1.000 8888"
//!                 .to_number_separators::<f32>(NumberCultureSettings::new(
//!                     Separator::DOT,
//!                     Separator::SPACE,
//!                     ThousandGrouping::ThreeBlock
//!                 ))
//!                 .unwrap(),
//!             1000.8888
//!         );
//! ```
//!
//! ## Example number to string
//!
//! ``` rust
//! use num_string::{Culture, ToFormat};
//!     // Some basic display (N0 = 0 digit, N2 = 2 digits etc)
//!     assert_eq!(1000.to_format("N0", Culture::English).unwrap(), "1,000");
//!     assert_eq!((-1000).to_format("N0", Culture::English).unwrap(), "-1,000");
//!     assert_eq!(1000.to_format("N2", Culture::French).unwrap(), "1 000,00");
//!
//!     // Perform the round decimal
//!     assert_eq!(10_000.9999.to_format("N2", Culture::French).unwrap(), "10 001,00");
//!     assert_eq!((-10_000.999).to_format("N2", Culture::French).unwrap(), "-10 001,00");
//! ```
//!
//! ## Example of number analysis
//!
//! ``` rust
//! use num_string::{ConvertString, Culture};
//! use num_string::pattern::TypeParsing;
//!     let string_num = ConvertString::new("1,000.2", Some(Culture::English));
//!     assert!(string_num.is_numeric());
//!     assert!(string_num.is_float());
//!     assert!(!string_num.is_integer());
//!
//!     // Convert to number
//!     assert_eq!(string_num.to_number::<f32>().unwrap(), 1000.2);
//!
//!     // If the conversion is ok (string_num.isNumeric() == true), you will have access to the matching pattern
//!     let matching_pattern = string_num.get_current_pattern().unwrap();
//!     assert_eq!(matching_pattern.get_regex().get_type_parsing(), &TypeParsing::DecimalThousandSeparator);
//!
//!     // If we try to convert a bad formatted number
//!     let string_error = ConvertString::new("NotANumber", Some(Culture::English));
//!     assert!(!string_error.is_numeric());
//! ```

use regex::Regex;

pub mod errors;
pub mod number;
pub mod conversion;
pub mod pattern;

pub use errors::ConversionError;
pub use number::ToFormat;
pub use conversion::NumberConversion;
pub use pattern::{ConvertString, NumberCultureSettings, Separator, ThousandGrouping};

/// Represent the current "ConvertString" culture
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Culture {
    English,
    French,
    Italian,
    Indian
}

/// Default culture = English
impl Default for Culture {
    fn default() -> Self {
        Culture::English
    }
}

impl TryFrom<&str> for Culture {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "en" => Culture::English,
            "fr" => Culture::French,
            "it" => Culture::Italian,
            _ => return Err(ConversionError::PatternCultureNotFound),
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::errors::ConversionError;
    use crate::conversion::NumberConversion;
    use crate::{Culture, ToFormat};

    // Run this function before each test
    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_number_parsing_simple() {
        assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
        assert_eq!(1000.to_format("N2", Culture::French).unwrap(), "1 000,00");
        assert_eq!(
            "1000".to_number::<i8>(),
            Err(ConversionError::UnableToConvertStringToNumber)
        );
        assert_eq!("1000".to_number::<f32>().unwrap(), 1000.0);
        assert_eq!(
            "1,000.8888"
                .to_number_culture::<f32>(Culture::English)
                .unwrap(),
            1000.8888
        );
    }

    #[test]
    fn test_number_to_format_integer() {
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
                number.to_format("N0", culture).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    #[test]
    fn test_number_to_format_float() {
        let floats = vec![
            (2_000.98, Culture::English, "2,000.98"),
            (-2_000.98, Culture::French, "-2 000,98"),
            (2_000.98, Culture::Italian, "2.000,98"),
            (049_490.8257, Culture::English, "49,490.83"),
            (10_000.9999, Culture::French, "10 001,00"),
            (-10_000.999, Culture::French, "-10 001,00"),
        ];
        for (number, culture, to_string_format) in floats {
            assert_eq!(
                number.to_format("N2", culture).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    #[test]
    fn test_reverse_mapping_number() {
        let values_int = vec![(1, "1", Culture::French), (1000, "1 000", Culture::French)];

        for (val_i32, val_str, culture) in values_int {
            assert_eq!(val_i32.to_format("N0", culture).unwrap(), val_str);
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
            assert_eq!(val_f64.to_format("N2", culture).unwrap(), val_str);
            assert_eq!(val_str.to_number_culture::<f64>(culture).unwrap(), val_f64);
        }
    }
}
