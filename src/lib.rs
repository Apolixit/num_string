use errors::ConversionError;
use log::{info, warn};
use num::Num;
use num_format::{WriteFormatted, Locale, ToFormattedString, CustomFormat, Grouping, Buffer};
use pattern::{CulturePattern, NumberType, ParsingPattern, Patterns};
use std::fmt::{Display, write};

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

impl Culture {
    pub fn to_local(&self) -> Locale {
        Culture::to_num_format_local(self)
    }

    pub fn to_num_format_local(culture: &Culture) -> Locale {
        match culture {
            Culture::English => Locale::en,
            Culture::French => Locale::fr,
            Culture::Italian => Locale::it
        }
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

impl FormatOption {
    pub fn new(minimum_fraction_digit: u8, maximum_fraction_digit: u8) -> FormatOption {
        FormatOption { minimum_fraction_digit, maximum_fraction_digit }
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

    pub fn to_format(&self, culture: &Culture) -> Result<String, ConversionError> {
        self.to_format_options(culture, FormatOption::new(2, 2))
    }

    pub fn to_format_options(&self, culture: &Culture, options: FormatOption) -> Result<String, ConversionError> {
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
        todo!()
    }
}

// impl<T: num::Num + Display> ToFormattedString for Number<T> {
//     fn read_to_fmt_writer<F, W>(&self, w: W, format: &F) -> Result<usize, std::fmt::Error>
//     where
//         F: num_format::Format,
//         W: std::fmt::Write {
//         write!(w, "{}", &self.to_string())
//     }

//     fn read_to_io_writer<F, W>(&self, w: W, format: &F) -> Result<usize, std::io::Error>
//     where
//         F: num_format::Format,
//         W: std::io::Write {
//         todo!()
//     }
// }

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
    use log::{info};
    use num::Float;
    use regex::Regex;
    use crate::{pattern::NumberType, ConvertString, Culture};
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

    #[test]
    fn test_test() {
        let int = 1000;
        let float = 1000.32;
        let zob = int.separate_with_commas();
        assert_eq!(int.separate_with_commas(), "1,000".to_owned());

        assert_eq!(float.to_string(), "1000.32");

        let regex = Regex::new(r"[0-9]+([\.])([0-9]+)").unwrap();
        let capture = regex.captures("1000.32").unwrap();
        info!("Hehe {:?}", capture);

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