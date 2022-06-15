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
use num::Num;
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
    pub fn apply_decimal_format(decimal_part: i32, options: FormatOption) -> String {
        let decimal_len = decimal_part.to_string().len() as u8;

        if decimal_len < options.minimum_fraction_digit {
            return (decimal_part
                * 10_i32.pow((options.minimum_fraction_digit - decimal_len) as u32))
            .to_string();
        }

        if decimal_len > options.maximum_fraction_digit {
            return (decimal_part
                / 10i32.pow((decimal_len - options.minimum_fraction_digit) as u32))
            .to_string();
        }

        decimal_part.to_string()
    }

    pub fn to_format_options(
        &self,
        culture: &Culture,
        format: FormatOption,
    ) -> Result<String, ConversionError> {
        let (sign_string, whole_string, decimal_opt_string) = self.regex_read_number()?;

        let mut number_string = Number::<T>::apply_thousand_separator(
            ConvertString::new(format!("{}{}", sign_string, whole_string).as_str(), None)
                .to_integer()
                .unwrap()
                .num,
            culture,
        );

        if let Some(decimal_string) = decimal_opt_string {
            let decimal_part = ConvertString::new(decimal_string.as_str(), None)
                .to_integer()
                .unwrap()
                .num;

            number_string = format!(
                "{}{}{}",
                number_string,
                NumberCultureSettings::from(*culture).decimal_separator,
                Number::<T>::apply_decimal_format(decimal_part, format)
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
    use crate::number::ToFormat;

    #[test]
    pub fn str_to_format() {
        assert_eq!(10000_i32.to_format("N0", &crate::Culture::French).unwrap(), "10 000");
    }
}
