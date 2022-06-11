use errors::ConversionError;
use log::{error, info, trace, warn};
use num::Num;
use pattern::{
    CulturePattern, NumberCultureSettings, NumberType, ParsingPattern, Patterns, Separator,
};
use regex::Regex;
use std::fmt::Display;
use thousands::Separable;

mod errors;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number<T: Num + Display> {
    num: T,
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
    pub fn apply_decimal_format(
        decimal_part: i32,
        options: FormatOption,
    ) -> String {
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
    use crate::{pattern::NumberType, ConvertString, Culture, FormatOption, Number};
    use log::{info, warn};
    use regex::Regex;
    use thousands::Separable;

    // Run this function before each test
    #[ctor::ctor]
    fn init() {
        env_logger::init();
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

    #[test]
    pub fn test_apply_decimal_format() {
        let list = vec![
            (512, FormatOption::new(2, 4), "512"),
            (512, FormatOption::new(2, 2), "51"),
            (512, FormatOption::new(5, 5), "51200"),
        ];

        for (num, format, string_num) in list {
            assert_eq!(Number::<i32>::apply_decimal_format(
                num,
                format
            ), string_num);
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
                Number::new(number).to_format(&culture).unwrap(),
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
        ];

        for (number, culture, to_string_format) in floats {
            assert_eq!(
                Number::new(number).to_format(&culture).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    #[test]
    pub fn test_number_to_format_explicit_float() {
        let floats = vec![
            (2_000.98, Culture::English, "2,000", FormatOption::new(0, 0)),
            (-2_000.98, Culture::French, "-2 000,9", FormatOption::new(0, 1)),
            (2_000.98, Culture::Italian, "2.000,98", FormatOption::new(0, 5)),
            (2_000.98, Culture::Italian, "2.000,98000", FormatOption::new(5, 5)),
        ];

        for (number, culture, to_string_format, format) in floats {
            assert_eq!(
                Number::new(number).to_format_options(&culture, format).unwrap(),
                String::from(to_string_format)
            );
        }
    }

    #[test]
    fn test_test() {
        let int = 1000;
        let float = 1000.32;
        let zob = int.separate_with_commas();
        assert_eq!(int.separate_with_commas(), "1,000".to_owned());

        let val = "1000.32";
        assert_eq!(float.to_string(), val);

        let regex = Regex::new(r"([0-9]+)([\.])([0-9]+)").unwrap();
        let capture = regex.captures(val).unwrap();
        info!("Hehe {:?}", capture);

        assert_eq!(capture.get(3).unwrap().as_str(), "32");

        let decimal_len = "32".len();
        let whole_part = ConvertString::new(capture.get(1).unwrap().as_str(), None)
            .to_integer()
            .unwrap()
            .num;
        let decimal_part = ConvertString::new(capture.get(3).unwrap().as_str(), None)
            .to_integer()
            .unwrap()
            .num;
        assert_eq!(2, decimal_len);
        assert_eq!(1000, whole_part);
        assert_eq!(32, decimal_part);
        let to_float_decimal_part = decimal_part as f32 / 10_i32.pow(decimal_len as u32) as f32;
        assert_eq!(0.32, to_float_decimal_part);

        let to_final_string = format!(
            "{}{}{}",
            whole_part.separate_with_spaces(),
            ",",
            decimal_part
        );
        assert_eq!("1 000,32", to_final_string);

        // let x1 = 1000.32;
        // let x2 = 1000;
        // let x3 = x1 - x2 as f64;
        // assert_eq!(x3, 0.32);
        // assert_eq!(x3.to_string(), "0.32");
        // assert_eq!(1000.32.trunc() as i32, 1000);
        // assert_eq!(1000.99 as i32, 1000);
        // let mut writer = String::new();
        // let format = CustomFormat::builder()
        //     .grouping(Grouping::Standard)
        //     .separator(" ")
        //     .decimal(",")
        //     .build()?;
        // 10.to_formatted_string(&culture.to_local());
        // let mut buf = Buffer::new();
        // buf.write_formatted(&10, &format);
        // buf.write_formatted(&10.2, &format);
        // let f = 10.0;

        // f.0.to_formatted_string(&culture.to_local());
        // writer.write_formatted(&self.num, &culture.to_local()).map_err(|e| ConversionError::UnableToDisplayFormat)?;
        // Ok(writer)
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
