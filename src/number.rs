use crate::number_conversion::NumberConversion;
use crate::pattern::Separator;
use crate::ConversionError;
use crate::ConvertString;
use crate::Culture;
use crate::FormatOption;
use crate::NumberCultureSettings;
use crate::Regex;
use log::error;
use log::trace;
use log::warn;
use num::Num;
use std::fmt::format;
use std::fmt::Display;
use thousands::Separable;

pub trait ToFormat {
    fn to_format(self, format: &str, culture: &Culture) -> Result<String, ConversionError>;
}

impl ToFormat for i32 {
    fn to_format(self, digit: &str, culture: &Culture) -> Result<String, ConversionError> {
        let nb_digit = Number::<i32>::set_nb_digits(digit)?;
        Number::<i32>::new(self).to_format_options(culture, FormatOption::new(nb_digit, nb_digit))
    }
}

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

    /// Return number to the current default culture format
    pub fn to_format(&self, culture: &Culture) -> Result<String, ConversionError> {
        self.to_format_options(culture, FormatOption::new(2, 2))
    }

    fn apply_thousand_separator(num: i32, culture: &Culture) -> String {
        let culture_settings = NumberCultureSettings::from(*culture);
        match culture_settings.to_thousand_separator() {
            Separator::COMMA => num.separate_with_commas(),
            Separator::DOT => num.separate_with_dots(),
            Separator::SPACE => num.separate_with_spaces(),
        }
    }

    /// Apply the format option to the decimal part (which is currently manipulated as a whole integer)
    pub fn apply_decimal_format(decimal_part: i32, options: FormatOption) -> Option<String> {
        if options.minimum_fraction_digit == 0 { return None; }

        let decimal_len = decimal_part.to_string().len() as u8;

        if decimal_len < options.minimum_fraction_digit {
            trace!(
                "The decimal part ({}) is smaller than the minimum_fraction_digit ({})",
                decimal_len,
                options.minimum_fraction_digit
            );
            return Some(format!(
                "{}{}",
                decimal_part,
                "0".repeat(options.minimum_fraction_digit as usize - decimal_len as usize)
            ));
        }

        if decimal_len > options.maximum_fraction_digit {
            trace!(
                "The decimal part ({}) is greater than the maximum_fraction_digit ({})",
                decimal_len,
                options.maximum_fraction_digit
            );
            let exp = 10i32.pow((decimal_len - options.minimum_fraction_digit) as u32) as f64;
            // let d = ((decimal_part as f64) / exp).round() as u128;
            // warn!("decimal_part = {} / {} = {}", decimal_part, exp, d);
            return Some((((decimal_part as f64) / exp).round() as u128).to_string());
        }

        trace!(
            "The decimal part ({}) is equal to the minimum/maximum_fraction_digit ({})",
            decimal_len,
            options.minimum_fraction_digit
        );
        Some(decimal_part.to_string())
    }

    pub fn to_format_options(
        &self,
        culture: &Culture,
        format: FormatOption,
    ) -> Result<String, ConversionError> {
        trace!("format = {:?}", format);
        let (sign_string, whole_string, decimal_opt_string) = self.regex_read_number()?;

        let mut number_string = Number::<T>::apply_thousand_separator(
            ConvertString::new(format!("{}{}", sign_string, whole_string).as_str(), None)
                .to_integer()
                .unwrap()
                .num,
            culture,
        );

        // the decimal read by the previous regex or "0" if None
        let decimal_string = decimal_opt_string.unwrap_or("0".to_owned());
        let decimal_part = ConvertString::new(decimal_string.as_str(), None)
            .to_integer()
            .unwrap()
            .num;

        trace!("Decimal part : {}", decimal_part);
        if let Some(decimal_format) = Number::<T>::apply_decimal_format(decimal_part, format) {
            number_string = format!(
                "{}{}{}",
                number_string,
                NumberCultureSettings::from(*culture).decimal_separator,
                decimal_format
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

#[cfg(test)]
mod tests {
    use log::trace;

    use crate::{number::ToFormat, FormatOption, Culture};

    use super::Number;

    #[test]
    pub fn str_to_format() {
        let vals = vec![
            (1000, "N0", Culture::French, "1 000"),
            (10000, "N2", Culture::French, "10 000,00"),
            (10000, "N4", Culture::English, "10,000.0000"),
            (-1000, "N0", Culture::Italian, "-1.000"),
            (-1000, "N3", Culture::Italian, "-1.000,000"),
            (1, "N1", Culture::English, "1.0"),
        ];

        for (val_i32, to_format, culture, string_result) in vals {
            assert_eq!(
                val_i32.to_format(to_format, &culture).unwrap(),
                string_result
            );
        }
    }

    #[test]
    pub fn test_apply_decimal() {
        let list = vec![
            (2, FormatOption::new(4, 4), "2000"),
            (265556, FormatOption::new(2, 2), "27"),
            (512, FormatOption::new(2, 4), "512"),
            (512, FormatOption::new(2, 2), "51"),
            (512, FormatOption::new(5, 5), "51200"),
        ];

        for (num, format, string_num) in list {
            assert_eq!(Number::<i32>::apply_decimal_format(
                num,
                format
            ).unwrap(), string_num);
        }
    }
}
