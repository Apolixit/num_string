use crate::pattern::ConvertString;
use crate::number_conversion::NumberConversion;
use crate::pattern::Separator;
use crate::ConversionError;
use crate::Culture;
use crate::NumberCultureSettings;
use crate::Regex;
use log::error;
use log::trace;
use num::Num;
use std::fmt::Display;
use thousands::Separable;

/// Trait to display a number with 'to_format' function
/// The format parameter is like C# toString() function with N0 / N2 / N4 values
/// N0 display 0 digit, N2 two digit, N4 four digit etc.
/// The max is N9 digit
/// And the culture parameter is use to display with the selected culture (it automatically
/// apply the thousand and decimal separator of the given culture)
/// # Example
/// ```
/// use num_string::{Culture, ToFormat};
///     assert_eq!(1000.to_format("N0", Culture::English).unwrap(), "1,000");
///     assert_eq!(1000.to_format("N2", Culture::French).unwrap(), "1 000,00");
/// ```
pub trait ToFormat {
    fn to_format(self, format: &str, culture: Culture) -> Result<String, ConversionError>;
}

/// Implement the trait for all primitive (i8, i64, u32, f32 etc.), thanks to Num trait
impl<T> ToFormat for T
where
    T: Num + Display,
{
    fn to_format(self, digit: &str, culture: Culture) -> Result<String, ConversionError> {
        let nb_digit = Number::<T>::set_nb_digits(digit)?;
        Number::<T>::new(self).to_format_options(&culture, FormatOption::new(nb_digit, nb_digit))
    }
}

/// A wrapper structure to perform the 'to_format' trait
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number<T: Num + Display> {
    pub num: T,
}

impl<T: num::Num + Display> Number<T> {
    pub fn new(num: T) -> Number<T> {
        Number { num }
    }

    /// Split the current number into a string
    /// Return the Sign, the Whole part and the optional Decimal part
    /// For example :
    ///     10000.65    should return : ("+", "10000", Some("65"))
    ///     -10         should return : ("-", "10", None)
    /// See 'test_split_number' for example
    pub fn regex_read_number(&self) -> Result<(String, String, Option<String>), ConversionError> {
        let str = &self.num.to_string();

        // Regex to split the current number
        let regex = Regex::new(r"([\-\+]?)([0-9]+)([\.]?)([0-9]*)").map_err(|e| {
            error!("{:?}", e);
            return ConversionError::UnableToConvertNumberToString;
        })?;

        let capture = regex
            .captures(str)
            .ok_or(ConversionError::NotCaptureFoundWhenConvertNumberToString)?;
        trace!("Text : {} / {:?}", str, capture);

        let capt = |index: usize| -> Option<String> {
            if let Some(matched) = capture.get(index) {
                let match_str = matched.as_str();
                if match_str.is_empty() {
                    return None;
                } else {
                    return Some(String::from(match_str));
                }
            }
            None
        };

        // Respectively : Sign (+ / -) | Whole part | Decimal part
        Ok((
            capt(1).unwrap_or(String::from("+")),
            capt(2).ok_or(ConversionError::UnableToConvertNumberToString)?,
            capt(4),
        ))
    }

    /// Return the number of digit pass in str parameter.
    /// Split the 'Nx' from the to_format trait
    /// Allowed values : N0, N1, N2, N3, N4, N5, N6, N7, N8, N9
    /// Ref test_set_nb_digits
    fn set_nb_digits(digit: &str) -> Result<u8, ConversionError> {
        if digit.len() != 2 {
            return Err(ConversionError::UnableToDisplayFormat);
        }

        let chars: Vec<char> = digit.chars().collect();
        if chars[0] != "N".chars().next().unwrap() {
            return Err(ConversionError::UnableToDisplayFormat);
        }

        Ok(chars[1].to_string().as_str().to_number::<u8>()?)
    }

    /// Apply the thousand separator to the whole number given in parameter
    /// Thanks to thousands crate 
    /// Ref 'test_apply_thousand_separator'
    fn apply_thousand_separator(num: i32, culture: &Culture) -> String {
        let culture_settings = NumberCultureSettings::from(*culture);
        match culture_settings.to_thousand_separator() {
            Separator::COMMA => num.separate_with_commas(),
            Separator::DOT => num.separate_with_dots(),
            Separator::SPACE => num.separate_with_spaces(),
        }
    }

    /// Apply the format option to the decimal part (which is currently manipulated as a whole integer)
    /// This function sucks, todo refacto later
    /// Ref 'test_apply_decimal'
    pub fn apply_decimal_format(decimal_part: i32, options: FormatOption) -> Option<(String, bool)> {
        if options.minimum_fraction_digit == 0 {
            return None;
        }

        let decimal_string = decimal_part.to_string();
        let decimal_len = decimal_string.len() as u8;

        if decimal_len < options.minimum_fraction_digit {
            trace!(
                "The decimal part ({}) is smaller than the minimum_fraction_digit ({})",
                decimal_len,
                options.minimum_fraction_digit
            );
            return Some((format!(
                "{}{}",
                decimal_part,
                "0".repeat(options.minimum_fraction_digit as usize - decimal_len as usize)
            ), false));
        }

        if decimal_len > options.maximum_fraction_digit {
            trace!(
                "The decimal part ({}) is greater than the maximum_fraction_digit ({})",
                decimal_len,
                options.maximum_fraction_digit
            );
            //Check if we need to round the whole part
            let decimal_rounded = decimal_part as f64 / (10i32.pow(decimal_len as u32 - options.maximum_fraction_digit as u32) as f64); 
            if decimal_rounded.round() as u32 == 10u32.pow(options.maximum_fraction_digit as u32) {
                trace!("Need to round the whole part up");
                return Some(("0".repeat(options.maximum_fraction_digit as usize), true));
            }

            let exp = 10i32.pow((decimal_len - options.maximum_fraction_digit) as u32) as f64;
            let calc = ((decimal_part as f64) / exp).round() as u128;
            return Some((calc.to_string(), false));
        }

        trace!(
            "The decimal part ({}) is equal to the minimum/maximum_fraction_digit ({})",
            decimal_len,
            options.minimum_fraction_digit
        );
        Some((decimal_part.to_string(), false))
    }

    /// Main function
    /// Apply the format to the number
    pub fn to_format_options(
        &self,
        culture: &Culture,
        format: FormatOption,
    ) -> Result<String, ConversionError> {
        trace!("format = {:?}", format);
        let (sign_string, whole_string, decimal_opt_string) = self.regex_read_number()?;
        
        let calc_to_string = |sign_string, whole_string| -> String {
            Number::<T>::apply_thousand_separator(
                ConvertString::new(format!("{}{}", sign_string, whole_string).as_str(), None)
                    .to_number::<i32>()
                    .unwrap(),
                culture,
            )
        };
        let mut number_string;

        // the decimal read by the previous regex or "0" if None
        let decimal_string = decimal_opt_string.unwrap_or("0".to_owned());
        let decimal_part = ConvertString::new(decimal_string.as_str(), None)
            .to_number::<i32>()
            .unwrap();

        trace!("Decimal part : {}", decimal_part);
        let decimal_opt = Number::<T>::apply_decimal_format(decimal_part, format);
        if let Some((decimal_format, need_round_up_whole_part)) = decimal_opt {
            if need_round_up_whole_part {
                number_string = calc_to_string(
                    sign_string,
                    (whole_string.as_str().to_number::<u64>().unwrap() + 1).to_string(),
                );
            } else {
                number_string = calc_to_string(sign_string, whole_string);
            }

            number_string = format!(
                "{}{}{}",
                number_string,
                NumberCultureSettings::from(*culture).to_decimal_separator_string(),
                decimal_format
            );
        } else {
            // No decimal required but
            let whole_number = whole_string.as_str().to_number::<u64>().unwrap();

            let exp = 10i32.pow(decimal_part.to_string().len() as u32) as f64;

            number_string = calc_to_string(
                sign_string,
                (whole_number + (((decimal_part as f64) / exp).round() as u64)).to_string(),
            );
        }

        Ok(number_string)
    }
}

impl<T: num::Num + Display> PartialEq<T> for Number<T> {
    fn eq(&self, other: &T) -> bool {
        &self.num == other
    }
}

impl<T: num::Num + Display> Display for Number<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.num)
    }
}

/// Structure with the nb decimal required when display a number to string
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
    use crate::number::FormatOption;
use crate::{number::ToFormat, Culture, errors::ConversionError};
    use super::Number;

    /// Test of 'to_format' function to display number to string with integer values
    #[test]
    pub fn str_to_format_integer() {
        let vals_i32 = vec![
            (1000, "N0", Culture::French, "1 000"),
            (10000, "N2", Culture::French, "10 000,00"),
            (10000, "N4", Culture::English, "10,000.0000"),
            (-1000, "N0", Culture::Italian, "-1.000"),
            (-1000, "N3", Culture::Italian, "-1.000,000"),
            (1, "N1", Culture::English, "1.0"),
        ];

        for (val_i32, to_format, culture, string_result) in vals_i32 {
            assert_eq!(
                val_i32.to_format(to_format, culture).unwrap(),
                string_result
            );
        }

        let vals_i8 = vec![
            (100_i8, "N0", Culture::French, "100"),
            (100_i8, "N2", Culture::French, "100,00"),
            (100_i8, "N4", Culture::English, "100.0000"),
            (-10_i8, "N0", Culture::Italian, "-10"),
            (-10_i8, "N3", Culture::Italian, "-10,000"),
            (-1_i8, "N1", Culture::English, "-1.0"),
        ];

        for (val_i8, to_format, culture, string_result) in vals_i8 {
            assert_eq!(
                val_i8.to_format(to_format, culture).unwrap(),
                string_result
            );
        }

        let vals_u64 = vec![
            (10000000_u64, "N0", Culture::French, "10 000 000"),
            (10000000_u64, "N2", Culture::French, "10 000 000,00"),
            (10000000_u64, "N4", Culture::English, "10,000,000.0000"),
        ];

        for (val_u64, to_format, culture, string_result) in vals_u64 {
            assert_eq!(
                val_u64.to_format(to_format, culture).unwrap(),
                string_result
            );
        }
    }

    /// Test of 'to_format' function to display number to string with float values
    #[test]
    pub fn str_to_format_float() {
        let vals_f64 = vec![
            (1000.48f64, "N0", Culture::French, "1 000"),
            (10000.48, "N2", Culture::French, "10 000,48"),
            (10000.99, "N4", Culture::English, "10,000.9900"),
            (-1000.98, "N0", Culture::Italian, "-1.001"),
            (-1000.66666, "N3", Culture::Italian, "-1.000,667"),
            (1., "N2", Culture::English, "1.00"),
            (2_000.98, "N0",  Culture::Italian, "2.001"),
            (2_000.98, "N2",  Culture::Italian, "2.000,98"),
            (2_000.98, "N3",  Culture::Italian, "2.000,980"),
            (2_000.9998888, "N3",  Culture::Italian, "2.001,000"),
        ];

        for (val_f64, to_format, culture, string_result) in vals_f64 {
            assert_eq!(
                val_f64.to_format(to_format, culture).unwrap(),
                string_result
            );
        }   
    }

    #[test]
    pub fn test_round_format() {
        assert_eq!(1000.66666.to_format("N2", Culture::French).unwrap(), "1 000,67");
        assert_eq!((-1000.66666).to_format("N2", Culture::French).unwrap(), "-1 000,67");

        assert_eq!(1000.999.to_format("N2", Culture::French).unwrap(), "1 001,00");
        assert_eq!((-1000.999).to_format("N2", Culture::French).unwrap(), "-1 001,00");
    }

    /// Test of 'apply_decimal_format' function
    #[test]
    pub fn test_apply_decimal() {
        let list = vec![
            (2, FormatOption::new(4, 4), "2000"),
            (265556, FormatOption::new(2, 2), "27"),
            (512, FormatOption::new(2, 4), "512"),
            (512, FormatOption::new(2, 2), "51"),
            (512, FormatOption::new(5, 5), "51200"),
        ];

        for (decimal_value, format, decimal_string) in list {
            assert_eq!(
                Number::<i32>::apply_decimal_format(decimal_value, format).unwrap().0,
                decimal_string
            );
        }
    }

    /// Test of 'to_format_options' function with float number
    #[test]
    pub fn test_number_to_format_option_float() {
        let floats = vec![
            (2_000.98, Culture::English, "2,001", FormatOption::new(0, 2)),
            (-2_000.98, Culture::French, "-2 001", FormatOption::new(0, 0)),
            (2_000.98, Culture::Italian, "2.000,980", FormatOption::new(3, 5)),
            (2_000.98, Culture::Italian, "2.000,98000", FormatOption::new(5, 5)),
        ];

        for (number, culture, to_string_format, format) in floats {
            assert_eq!(
                Number::new(number).to_format_options(&culture, format).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    /// Test the 'regex_read_number' function
    #[test]
    fn test_split_number() {
        assert_eq!(
            Number::new(1_000.32f32).regex_read_number().unwrap(),
            ("+".to_owned(), "1000".to_owned(), Some("32".to_owned())),
            "Error when spliting 1_000.32f32"
        );

        assert_eq!(
            Number::new(-1_000_000.32f64).regex_read_number().unwrap(),
            ("-".to_owned(), "1000000".to_owned(), Some("32".to_owned())),
            "Error when spliting -1_000_000.32f64"
        );

        assert_eq!(
            Number::new(-1_000i32).regex_read_number().unwrap(),
            ("-".to_owned(), "1000".to_owned(), None),
            "Error when spliting -1_000i32"
        );

        assert_eq!(
            Number::new(2).regex_read_number().unwrap(),
            ("+".to_owned(), "2".to_owned(), None),
            "Error when spliting 2"
        );
    }

    /// The the 'set_nb_digits' function
    #[test]
    fn test_set_nb_digits() {
        let values_ok = vec![
            ("N0", 0),
            ("N2", 2),
            ("N4", 4),
            ("N9", 9),
        ];
        let values_error = vec![
            ("N10", ConversionError::UnableToDisplayFormat),
            ("N200", ConversionError::UnableToDisplayFormat),
            ("good morning", ConversionError::UnableToDisplayFormat),
            ("Polkadot", ConversionError::UnableToDisplayFormat),
        ];

        for (format_str, result) in values_ok {
            assert_eq!(Number::<i32>::set_nb_digits(format_str), Ok(result));
        }

        for (format_str, result) in values_error {
            assert_eq!(Number::<i32>::set_nb_digits(format_str), Err(result));
        }
    }

    /// The the 'apply_thousand_separator' function
    #[test]
    fn test_apply_thousand_separator() {
        let values = vec![
            (1000, Culture::French, "1 000"),
            (-1000000, Culture::French, "-1 000 000"),
            (1000, Culture::English, "1,000"),
            (-1000000, Culture::English, "-1,000,000"),
            (1000, Culture::Italian, "1.000"),
            (-1000000, Culture::Italian, "-1.000.000"),
        ];

        for (val_i32, culture, val_string) in values {
            assert_eq!(Number::<i32>::apply_thousand_separator(val_i32, &culture), val_string)
        }
    }
}
