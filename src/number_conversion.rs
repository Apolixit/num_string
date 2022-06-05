use std::str::FromStr;

use log::{info, error};
use regex::Regex;

use crate::{
    errors::ConversionError,
    pattern::{NumberCultureSettings},
    Number,
};

pub trait IntegerConversion<I: num::Integer> {
    fn to_integer(&self) -> Result<Number<I>, ConversionError>;
}

pub trait FloatConversion<F: num::Float> {
    fn to_float(&self) -> Result<Number<F>, ConversionError>;
}

pub struct StringNumber {
    value: String,
    number_culture_settings: Option<NumberCultureSettings>,
}

impl StringNumber {
    pub fn new(value: String) -> StringNumber {
        StringNumber {
            value,
            number_culture_settings: None,
        }
    }

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

    /// Return settings as option reference
    pub fn get_settings(&self) -> Option<&NumberCultureSettings> {
        self.number_culture_settings.as_ref()
    }

    /// Replace the string which match the regex by the replacement string
    fn replace_element(string_number: &str, string_regex: &str, replacement: &str) -> String {
        let regex_space = Regex::new(string_regex).unwrap();
        let cleaned_input = regex_space.replace_all(string_number, replacement);

        cleaned_input.to_string()
    }

    /// Create regex from struct to clean the string
    pub fn clean(&self) -> String {
        let mut string_value = self.value.clone();

        // Shortcut closure to call replace_element function
        let replace = |string_input: &str, separator: &str, replacement: &str| {
            StringNumber::replace_element(
                string_input,
                format!(r"{}", separator).as_str(),
                replacement,
            )
        };

        //Clean decimal and thousand separator if needed
        if self.has_settings() {
            string_value = replace(
                &string_value,
                &self.get_settings().unwrap().decimal_separator.as_str(),
                ".",
            );
            string_value = replace(
                &string_value,
                &self.get_settings().unwrap().thousand_separator.as_str(),
                "",
            );
        } else {
            string_value = replace(&string_value, r"\s", "");
        }

        info!("Input before clean = {} / after clean = {}", self.value, string_value);
        string_value
    }
}

/// Convert the string number to integer
impl IntegerConversion<i32> for StringNumber {
    fn to_integer(&self) -> Result<Number<i32>, ConversionError> {
        Ok(Number::new(
            self.clean()
                .parse::<f32>()
                .map_err(|e| {
                    error!("{}", e.to_string());
                    ConversionError::UnableToConvert
                })? as i32,
        ))
    }
}

/// Convert the string number to float
impl FloatConversion<f32> for StringNumber {
    fn to_float(&self) -> Result<Number<f32>, ConversionError> {
        Ok(Number::new(
            self.clean()
                .parse::<f32>()
                .map_err(|e| {
                    error!("{}", e.to_string());
                    ConversionError::UnableToConvert
                })?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        number_conversion::{FloatConversion, IntegerConversion, StringNumber},
        pattern::NumberCultureSettings, errors::ConversionError,
    };

    // Run this function before each test
    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn number_conversion_whole() {
        let list = vec![
            ("10", 10, 10.0),
            ("0", 0, 0.0),
            ("-10", -10, -10.0),
            ("1000", 1000, 1000.0),
            ("1 000", 1000, 1000.0),
        ];

        for (string_value, int_value, float_value) in list {
            let wn = StringNumber::new(String::from(string_value));

            let int_conversion = wn.to_integer().expect(
                format!(
                    "{} string hasn't been convert to integer",
                    wn.value.to_string()
                )
                .as_str(),
            );
            assert_eq!(int_conversion.num, int_value);

            let float_conversion = wn.to_float().expect(
                format!(
                    "{} string hasn't been convert to float",
                    wn.value.to_string()
                )
                .as_str(),
            );
            assert_eq!(float_conversion.num, float_value);
        }
    }

    #[test]
    fn number_conversion_decimal() {
        let list = vec![
            ("10,0", 10, 10.0),
            ("0,25", 0, 0.25),
            ("-10,5", -10, -10.5),
            ("1000,89", 1000, 1000.89),
            ("1 000,4564654654654", 1000, 1000.4564654654654),
            ("1000,4564654654654", 1000, 1000.4564654654654),
        ];

        for (string_value, int_value, float_value) in list {
            let wn = StringNumber::new_with_settings(
                String::from(string_value),
                NumberCultureSettings::new(" ", ","),
            );

            let int_conversion = wn.to_integer().expect(
                format!(
                    "{} string couldn't been converted to integer",
                    wn.value.to_string()
                )
                .as_str(),
            );
            assert_eq!(int_conversion.num, int_value);

            let float_conversion = wn.to_float().expect(
                format!(
                    "{} string couldn't been converted to float",
                    wn.value.to_string()
                )
                .as_str(),
            );
            assert_eq!(float_conversion.num, float_value);
        }
    }

    #[test]
    fn number_conversion_not_allowed() {
        let list = vec!["x", "10*5"];

        for (string_value) in list {
            let wn = StringNumber::new(
                String::from(string_value),
            );

            assert_eq!(wn.to_integer(), Err(ConversionError::UnableToConvert));
        }
    }
}