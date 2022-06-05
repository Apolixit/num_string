use std::string;

use crate::number_conversion::FloatConversion;
use crate::number_conversion::IntegerConversion;
use crate::number_conversion::StringNumber;
use crate::Culture;
use crate::Number;
use num;
use regex::Regex;

/// Represent if the number is Whole (int), or Decimal (float)
#[derive(Debug, Clone, PartialEq)]
pub enum NumberType {
    WHOLE,
    DECIMAL,
}

pub type Convertable = dyn ToString;

/// Regex use to try to convert string to number
#[derive(Debug, Clone)]
pub struct RegexPattern {
    prefix: Regex,
    content: Regex,
    suffix: Regex,
}

impl RegexPattern {
    pub fn is_match(&self, text: &str) -> bool {
        let full_regex =
            Regex::new(format!("{}{}{}", self.prefix, self.content, self.suffix).as_str()).unwrap();
        full_regex.is_match(text)
    }
}

/// The parsing pattern wrapper
/// <I: num::Integer, F: num::Float>
#[derive(Debug, Clone)]
pub struct ParsingPattern {
    name: String,
    culture_settings: Option<NumberCultureSettings>,
    pub regex: RegexPattern,
    pub number_type: NumberType,
    additional_pattern: Option<String>,
    // to_integer: dyn Fn(&Self) -> Number<I>,
    // to_float: dyn Fn<X>(x: X) -> Number<F>,
}

impl ParsingPattern {
    pub fn to_integer(&self, string_number: String) -> Option<Number<i32>> {
        self.parsing(string_number).to_integer().ok()
    }

    pub fn to_float(&self, string_number: String) -> Option<Number<f32>> {
        self.parsing(string_number).to_float().ok()
    }

    fn parsing(&self, string_number: String) -> StringNumber {
        if self.culture_settings.is_none() {
            StringNumber::new(string_number)
        } else {
            StringNumber::new_with_settings(string_number, self.culture_settings.clone().unwrap())
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberCultureSettings {
    pub thousand_separator: String,
    pub decimal_separator: String,
}

impl NumberCultureSettings {
    pub fn new(thousand_separator: &str, decimal_separator: &str) -> NumberCultureSettings {
        NumberCultureSettings {
            thousand_separator: thousand_separator.to_owned(),
            decimal_separator: decimal_separator.to_owned(),
        }
    }
}

/// The pattern which is culture dependent. Allow us to try to parse multi culture string
#[derive(Debug, Clone)]
pub struct CulturePattern {
    name: String,
    value: Vec<Culture>,
    patterns: Vec<ParsingPattern>,
}

impl CulturePattern {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_cultures(&self) -> &Vec<Culture> {
        &self.value
    }

    pub fn get_patterns(&self) -> &Vec<ParsingPattern> {
        &self.patterns
    }
}

/// All pattern defined to try to convert string to number
pub struct Patterns {
    common_pattern: Vec<ParsingPattern>,
    culture_pattern: Vec<CulturePattern>,
    math_pattern: Vec<ParsingPattern>,
}

impl Patterns {
    pub fn new() -> Patterns {
        Patterns::default()
    }

    pub fn get_culture_pattern(&self) -> Vec<CulturePattern> {
        self.culture_pattern.to_vec()
    }

    pub fn add_culture_pattern(&mut self, pattern: CulturePattern) {
        self.culture_pattern.push(pattern);
    }

    pub fn get_common_pattern(&self) -> Vec<ParsingPattern> {
        self.common_pattern.to_vec()
    }

    pub fn add_common_pattern(&mut self, pattern: ParsingPattern) {
        self.common_pattern.push(pattern);
    }

    pub fn get_math_pattern(&self) -> Vec<ParsingPattern> {
        self.math_pattern.to_vec()
    }

    pub fn add_math_pattern(&mut self, pattern: ParsingPattern) {
        self.math_pattern.push(pattern);
    }
}

impl Default for Patterns {
    fn default() -> Self {
        Patterns {
            common_pattern: vec![
                ParsingPattern {
                    /*
                     * X / +X / -X
                     * Ex: 1000 / -1000 / +1000
                     */
                    name: String::from("Common_Simple_Whole"),
                    number_type: NumberType::WHOLE,
                    culture_settings: None,
                    additional_pattern: None,
                    regex: RegexPattern {
                        prefix: Regex::new(r"^").unwrap(),
                        content: Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap(),
                        suffix: Regex::new(r"$").unwrap(),
                    },
                },
                ParsingPattern {
                    /*
                     * .XX
                     * Ex: .25 / ,25
                     */
                    name: String::from("Common_Decimal_Without_Whole_Part"),
                    number_type: NumberType::DECIMAL,
                    culture_settings: None,
                    additional_pattern: None,
                    regex: RegexPattern {
                        prefix: Regex::new(r"^").unwrap(),
                        content: Regex::new(r"[\-\+]?[\.,][0-9]+").unwrap(),
                        suffix: Regex::new(r"$").unwrap(),
                    },
                },
            ],
            culture_pattern: vec![CulturePattern {
                name: String::from("fr"),
                value: vec![Culture::French],

                patterns: vec![ParsingPattern {
                    /*
                     * X,XX
                     * Ex: 1,2 / 0,35 / 1545456465000,25465
                     */
                    name: String::from("FR_Decimal_Simple"),
                    number_type: NumberType::DECIMAL,
                    culture_settings: Some(NumberCultureSettings {
                        decimal_separator: String::from(","),
                        thousand_separator: String::from(" "),
                    }),
                    additional_pattern: None,
                    regex: RegexPattern {
                        prefix: Regex::new(r"^").unwrap(),
                        content: Regex::new(r"[\-\+]?[0-9]+[\\,\\.][0-9]{1,}").unwrap(),
                        suffix: Regex::new(r"$").unwrap(),
                    },
                }],
            }],
            math_pattern: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    #[test]
    fn test_regex() {
        let r = Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap();
        assert!(r.is_match("10,2"));
    }
}
