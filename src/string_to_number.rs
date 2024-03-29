use crate::Culture;
use std::{fmt::Display, str::FromStr};

use log::{trace, info, debug};
use regex::Regex;

use crate::{errors::ConversionError, pattern::NumberCultureSettings};

/// Trait implemented to convert a string number to Rust number
/// ``` rust
/// use num_string::{Culture, ConversionError, NumberConversion};
/// use num_string::{pattern::{NumberCultureSettings, ThousandGrouping}, Separator};
/// 
///     assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
///     assert_eq!("1000.5822".to_number::<f32>().unwrap(), 1000.5822);
///     assert_eq!("1,000.8888".to_number_culture::<f32>(Culture::English).unwrap(), 1000.8888);
///     assert_eq!("-5'000.66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::APOSTROPHE, Separator::DOT)).unwrap(), -5000.66);
// ```
pub trait NumberConversion {
    /// Try to convert a common string (not culture dependent)
    fn to_number<N: num::Num + Display + FromStr>(&self) -> Result<N, ConversionError>;

    /// Try to convert a string with given thousand and decimal separator
    fn to_number_separators<N: num::Num + Display + FromStr>(
        &self,
        separators: NumberCultureSettings,
    ) -> Result<N, ConversionError>;

    /// Try to convert a string with given culture
    fn to_number_culture<N: num::Num + Display + FromStr>(
        &self,
        culture: Culture,
    ) -> Result<N, ConversionError>;
}

/// Structure which represent a string number (can be either well formated or bad formated)
struct StringNumber {
    value: String,
    number_culture_settings: Option<NumberCultureSettings>,
}

impl StringNumber {
    /// Create a new instance with only the string number
    pub fn new(value: String) -> StringNumber {
        StringNumber {
            value,
            number_culture_settings: None,
        }
    }

    /// Create a new instance with the thousand and decimal separator
    pub fn new_with_settings(
        value: String,
        number_culture_settings: NumberCultureSettings,
    ) -> StringNumber {
        StringNumber {
            value,
            number_culture_settings: Some(number_culture_settings),
        }
    }

    /// Does number_culture_settings has been specified
    pub fn has_settings(&self) -> bool {
        self.number_culture_settings.is_some()
    }

    /// Get the decimal separator for float number in Rust
    pub fn string_decimal_replacement() -> String {
        String::from(".")
    }

    /// Return settings as option reference
    pub fn get_settings(&self) -> Option<&NumberCultureSettings> {
        self.number_culture_settings.as_ref()
    }

    /// Replace the string which match the regex by the replacement string
    fn replace_element(string_number: &str, string_regex: &str, replacement: &str) -> String {
        // let regex_space = Regex::new(format!(r"[\\{}]", string_regex).as_str()).unwrap();
        let regex_space = Regex::new(string_regex).unwrap();
        debug!(
            "Regex replace : {:?} / string_value = {} / string replacement = {}",
            regex_space,
            string_number,
            replacement
        );

        let cleaned_input = regex_space.replace_all(string_number, replacement);

        cleaned_input.to_string()
    }

    /// Create regex from struct to clean the string.
    ///
    /// Return the string cleaned.
    pub fn clean(&self) -> String {
        info!(
            "Clean with string input = {} and separators = {:?}",
            &self.value, &self.number_culture_settings
        );
        let mut string_value = self.value.clone();

        // Shortcut closure to call replace_element function
        let replace = |string_input: &str, separator: &str, replacement: &str| {
            StringNumber::replace_element(
                string_input,
                separator, //format!(r"{}", separator).as_str(),
                replacement,
            )
        };

        //Clean decimal and thousand separator if needed
        if self.has_settings() {
            debug!(
                "Decimal ({}) and thousand ({}) separator has been specified",
                &self.get_settings().unwrap().into_decimal_separator_string(),
                &self.get_settings().unwrap().into_thousand_separator_string()
            );

            trace!("Begin thousand separator replace");
            string_value = replace(
                &string_value,
                &self.get_settings().unwrap().into_thousand_separator_regex(),
                "",
            );
            trace!(
                "End thousand separator replace. string_value = {}",
                string_value
            );

            trace!("Begin decimal separator replace");
            string_value = replace(
                &string_value,
                &self.get_settings().unwrap().into_decimal_separator_regex(),
                StringNumber::string_decimal_replacement().as_str(),
            );
            trace!(
                "End decimal separator replace. string_value = {}",
                string_value
            );
        } else {
            string_value = replace(&string_value, r"\s", "");
        }

        debug!(
            "Input before clean = {} / after clean = {}",
            self.value,
            string_value
        );
        string_value
    }
}

impl NumberConversion for &str {
    fn to_number<N>(&self) -> Result<N, ConversionError>
    where
        N: num::Num,
        N: std::fmt::Display,
        N: std::str::FromStr,
    {
        StringNumber::new(String::from(*self)).to_number()
    }

    fn to_number_separators<N>(
        &self,
        pattern: NumberCultureSettings,
    ) -> std::result::Result<N, ConversionError>
    where
        N: num::Num,
        N: std::fmt::Display,
        N: std::str::FromStr,
    {
        StringNumber::new_with_settings(String::from(*self), pattern).to_number()
    }

    fn to_number_culture<N>(&self, culture: Culture) -> Result<N, ConversionError>
    where
        N: num::Num,
        N: std::fmt::Display,
        N: std::str::FromStr,
    {
        StringNumber::new_with_settings(String::from(*self), culture.into())
            .to_number()
    }
}

impl NumberConversion for StringNumber {
    fn to_number<N: num::Num + Display + FromStr>(&self) -> Result<N, ConversionError> {
        Ok(self
            .clean()
            .parse::<N>()
            .map_err(|_e| ConversionError::UnableToConvertStringToNumber)?)
    }

    fn to_number_separators<N>(
        &self,
        _pattern: NumberCultureSettings,
    ) -> std::result::Result<N, ConversionError>
    where
        N: num::Num,
        N: std::fmt::Display,
        N: std::str::FromStr,
    {
        self.to_number()
    }

    fn to_number_culture<N>(&self, _: Culture) -> std::result::Result<N, ConversionError>
    where
        N: num::Num,
        N: std::fmt::Display,
        N: std::str::FromStr,
    {
        self.to_number()
    }
}

#[cfg(test)]
mod tests {
    use regex::escape;

    use crate::{
        errors::ConversionError,
        string_to_number::{NumberConversion, StringNumber},
        pattern::{NumberCultureSettings, ThousandGrouping}, Separator,
    };

    fn dot_comma() -> NumberCultureSettings {
        NumberCultureSettings::from((".", ","))
    }
    fn comma_dot() -> NumberCultureSettings {
        NumberCultureSettings::from((",", "."))
    }
    fn comma_dot_grouping_two() -> NumberCultureSettings {
        NumberCultureSettings::from((",", ".")).with_grouping(ThousandGrouping::TwoBlock)
    }
    fn space_comma() -> NumberCultureSettings {
        NumberCultureSettings::from((" ", ","))
    }

    #[test]
    fn test_number_separator() {
        // Use enum when it's ok
        assert_eq!("-5'000.66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::APOSTROPHE, Separator::DOT)).unwrap(), -5000.66);
        // Use enum when it's not necessary, should work
        assert_eq!("1000.66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::APOSTROPHE, Separator::COMMA)).unwrap(), 1000.66);

        // Use text
        assert_eq!("-5{000.66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('{'), Separator::DOT)).unwrap(), -5000.66);
        
        // https://fr.piliapp.com/emoji/list/
        // Should work
        assert_eq!("-5🍓000🦀66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🍓'), Separator::CUSTOM('🦀'))).unwrap(), -5000.66);
        assert_eq!("-5🦀🦀🦀🦀🦀000🍓66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🦀'), Separator::CUSTOM('🍓'))).unwrap(), -5000.66);
        
        
        assert_eq!("-5🍓000🍓000🦀66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🍓'), Separator::CUSTOM('🦀'))).unwrap(), -5000000.66);
        
        // It works but it's close to be an invalid separator
        assert_eq!("-5🍓🍓🍓🍓🍓🍓000🦀66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🍓'), Separator::CUSTOM('🦀'))).unwrap(), -5000.66);
        assert_eq!("-5🍓🍓000🍓🍓000🦀66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🍓'), Separator::CUSTOM('🦀'))).unwrap(), -5000000.66);
    }

    #[test]
    #[should_panic]
    fn test_number_separator_same_separator() {
        assert_eq!("-5|000|66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('|'), Separator::CUSTOM('|'))).unwrap(), -5000.66);
    }

    #[test]
    #[should_panic]
    fn test_number_separator_multiple_decimal() {
        assert_eq!("-5🍓000🦀🦀🦀🦀🦀🦀🦀66".to_number_separators::<f32>(NumberCultureSettings::new(Separator::CUSTOM('🍓'), Separator::CUSTOM('🦀'))).unwrap(), -5000.66);
    }
    /// Simple integer conversion
    #[test]
    fn number_conversion_integer() {
        let list = vec![
            ("10", 10, 10.0),
            ("0", 0, 0.0),
            ("-10", -10, -10.0),
            ("1000", 1000, 1000.0),
            ("1 000", 1000, 1000.0),
        ];

        for (string_value, int_value, float_value) in list {
            assert_eq!(string_value.to_number::<i32>().unwrap(), int_value);
            assert_eq!(string_value.to_number::<f64>().unwrap(), float_value);
        }
    }

    /// Simple decimal conversion
    #[test]
    fn number_conversion_decimal() {
        let list = vec![
            ("10,0", 10.0),
            ("0,25", 0.25),
            ("-10,5", -10.5),
            ("1000,89", 1000.89),
            ("1 000,4564654654654", 1000.4564654654654),
            ("1000,4564654654654", 1000.4564654654654),
        ];

        for (string_value, float_value) in list {
            assert_eq!(
                string_value
                    .to_number_separators::<f64>(NumberCultureSettings::from((" ", ",")))
                    .unwrap(),
                float_value
            );
        }
    }

    /// Conversion with several thousand and decimal separator
    #[test]
    fn number_conversion_others() {
        assert_eq!(
            "10.000.000"
                .to_number_separators::<i32>(dot_comma())
                .unwrap(),
            10_000_000
        );

        assert_eq!(
            "10,000,000"
                .to_number_separators::<i32>(comma_dot())
                .unwrap(),
            10_000_000
        );

        assert_eq!(
            "10,00,00,00,000"
                .to_number_separators::<i64>(comma_dot_grouping_two())
                .unwrap(),
            10_000_000_000
        );

        assert_eq!(
            "1.000,45"
                .to_number_separators::<f64>(dot_comma())
                .unwrap(),
            1_000.45
        );

        assert_eq!(
            "1.000"
                .to_number_separators::<i32>(dot_comma())
                .unwrap(),
            1_000
        );
    }

    /// Conversion with i8 primitive
    #[test]
    fn number_conversion_primitive_dependent_i8() {
        /* Reminder :
        * i8 : [-128: 128]
        */

        let i8_ok = "120";
        assert_eq!(
            i8_ok
                .to_number::<i8>()
                .unwrap(),
            120
        );
        assert_eq!(
            i8_ok
                .to_number::<i64>()
                .unwrap(),
            120
        );
        assert_eq!(
            i8_ok
                .to_number::<u8>()
                .unwrap(),
            120
        );
    }

    /// Conversion with i16 primitive
    #[test]
    fn number_conversion_primitive_dependent_i16() {
        /* Reminder :
        * i16 : [-32768: 32768]
        */
        let i16_ok = "-10000";
        assert_eq!(
            i16_ok
                .to_number::<i16>()
                .unwrap(),
            -10_000
        );

        assert_eq!(
            i16_ok.to_number::<i8>(),
            Err(ConversionError::UnableToConvertStringToNumber)
        );
    }

    #[test]
    fn number_error_conversion() {
        assert_eq!(
            "10,000,000"
                .to_number_separators::<i32>(space_comma()),
            Err(ConversionError::UnableToConvertStringToNumber)
        );

        assert_eq!(
            "10,00,00,00"
                .to_number_separators::<i32>(space_comma()),
            Err(ConversionError::UnableToConvertStringToNumber)
        );
        assert_eq!(
            "10,00,00,00"
                .to_number::<i32>(),
            Err(ConversionError::UnableToConvertStringToNumber)
        );
    }
    #[test]
    fn number_conversion_not_allowed() {
        let list = vec!["x", "10*5", "2..500"];

        for string_value in list {
            let wn = StringNumber::new(String::from(string_value));

            assert_eq!(
                wn.to_number::<i32>(),
                Err(ConversionError::UnableToConvertStringToNumber)
            );
        }
    }

    #[test]
    fn escape_special_char_regex() {
        // escape
        assert_eq!("\'", escape("'"));
        assert_eq!("\\|AnyThousandSeparator\\|", escape("|AnyThousandSeparator|"));
        assert_eq!("🍓", escape("🍓"));
        assert_eq!("🦀🦀", escape("🦀🦀"));
    }   
}
