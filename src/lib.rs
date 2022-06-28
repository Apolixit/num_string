use regex::Regex;

mod errors;
mod number;
mod number_conversion;
mod pattern;

pub use number::ToFormat;
pub use number_conversion::NumberConversion;
pub use errors::ConversionError;
pub use pattern::{NumberCultureSettings, Separator};


/// This crate perform conversion between string and number
/// It allows to convert culture formated number to Rust number
/// And it allowed to display Rust numbers to culture formated string
/// 
/// Here some basic utilisation :
/// # Example string to number
/// 
/// ## Basic string to number
/// ```
/// use num_string::{Culture, ConversionError, NumberConversion};
/// 
///     // Basic string number
///     assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
///     assert_eq!("+1000".to_number::<i64>().unwrap(), 1000);
///     assert_eq!("-1000".to_number::<i64>().unwrap(), -1000);
///     assert_eq!("1000".to_number::<f32>().unwrap(), 1000.0);
///
///     // Fail because 1000 > i8 capacity
///     assert_eq!("1000".to_number::<i8>(), Err(ConversionError::UnableToConvertStringToNumber));
/// ```
/// 
/// ## For more advanced conversion you can specify culture
/// ```
/// use num_string::{Culture, NumberConversion};
///     assert_eq!("1,000.8888".to_number_culture::<f32>(Culture::English).unwrap(), 1000.8888);
///     assert_eq!("-10 564,10".to_number_culture::<f32>(Culture::French).unwrap(), -10564.10);
/// ```
/// 
/// ## You can also specify some custom (DOT as thousand separator and SPACE a decimal separator)
/// ```
/// use num_string::{NumberCultureSettings, Separator, NumberConversion};
///     assert_eq!("1.000 8888".to_number_separators::<f32>(NumberCultureSettings::new(Separator::DOT, Separator::SPACE)).unwrap(), 1000.8888);
/// ```
/// 
/// # Example number to string
/// ```
/// use num_string::{Culture, ToFormat};
///     // Some basic display
///     assert_eq!(1000.to_format("N0", &Culture::English).unwrap(), "1,000");
///     assert_eq!(1000.to_format("N2", &Culture::French).unwrap(), "1 000,00");
/// 
///     // Perform the round decimal
///     assert_eq!(10000.9999.to_format("N2", &Culture::French).unwrap(), "10 001,00");
/// ```
/// Please ref to other file for more advanced tests and explaination


/// Represent the current "ConvertString" culture
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Culture {
    English,
    French,
    Italian,
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
            _ => return Err(ConversionError::PatternCultureNotFound)
        })
    }
}

#[cfg(test)]
mod tests {

use crate::errors::ConversionError;
use crate::number_conversion::NumberConversion;
use crate::NumberCultureSettings;
use crate::{
        number::ToFormat,
        pattern::{NumberType, Separator}, Culture,
    };

    // Run this function before each test
    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }
    #[test]
    fn x() {
        assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
        assert_eq!(1000.to_format("N2", &Culture::French).unwrap(), "1 000,00");
        assert_eq!("1000".to_number::<i8>(), Err(ConversionError::UnableToConvertStringToNumber));
        assert_eq!("1000".to_number::<f32>().unwrap(), 1000.0);
        assert_eq!("1,000.8888".to_number_culture::<f32>(Culture::English).unwrap(), 1000.8888);
        
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
                number.to_format("N0", &culture).unwrap(),
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
        ];

        for (number, culture, to_string_format) in floats {
            assert_eq!(
                number.to_format("N2", &culture).unwrap(),
                String::from(to_string_format)
            );
        }
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
}