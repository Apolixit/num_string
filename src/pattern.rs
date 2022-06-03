use crate::Culture;
use crate::Number;
use num;
use regex::Regex;

/// Represent if the number is Whole (int), or Decimal (float)
pub enum NumberType {
    WHOLE,
    DECIMAL,
}

pub trait NumberConversion<I: num::Integer, F: num::Float> {
    fn to_integer(&self) -> Number<I>;
    fn to_float(&self) -> Number<F>;
}

/// Regex use to try to convert string to number
pub struct RegexPattern {
    prefix: Regex,
    content: Regex,
    suffix: Regex,
}

/// The parsing pattern wrapper
/// <I: num::Integer, F: num::Float>
pub struct ParsingPattern {
    name: String,
    regex: RegexPattern,
    number_type: NumberType,
    additional_pattern: Option<String>,
    // to_integer: dyn Fn(&Self) -> Number<I>,
    // to_float: dyn Fn<X>(x: X) -> Number<F>,
}

pub struct NumberCultureSettings {
    ThousandSeparator: String,
    DecimalSeparator: String
}

/// The pattern which is culture dependent. Allow us to try to parse multi culture string
pub struct CulturePattern {
    name: String,
    value: Vec<Culture>,
    culture_settings: NumberCultureSettings,
    pattern: Vec<ParsingPattern>,
}

/// All pattern defined to try to convert string to number
pub struct Patterns {
    pub common_pattern: Vec<ParsingPattern>,
    pub culture_pattern: Vec<CulturePattern>,
    pub math_pattern: Vec<ParsingPattern>,
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
                    additional_pattern: None,
                    regex: RegexPattern {
                        prefix: Regex::new(r"^").unwrap(),
                        content: Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap(),
                        suffix: Regex::new(r"$").unwrap(),
                    }
                },
                ParsingPattern {
                    /*
                    * .XX
                    * Ex: .25 / ,25
                    */
                    name: String::from("Common_Decimal_Without_Whole_Part"),
                    number_type: NumberType::DECIMAL,
                    additional_pattern: None,
                    regex: RegexPattern {
                        prefix: Regex::new(r"^").unwrap(),
                        content: Regex::new(r"[\-\+]?[\.,][0-9]+").unwrap(),
                        suffix: Regex::new(r"$").unwrap(),
                    }
                }
            ],
            culture_pattern: vec![
                CulturePattern {
                    name: String::from("fr"),
                    value: vec![Culture::French],
                    culture_settings: NumberCultureSettings {
                        DecimalSeparator: String::from(","),
                        ThousandSeparator: String::from(" "),
                    },
                    pattern: vec![
                        ParsingPattern {
                            /*
                            * X,XX
                            * Ex: 1,2 / 0,35 / 1545456465000,25465
                            */
                            name: String::from("FR_Decimal_Simple"),
                            number_type: NumberType::DECIMAL,
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+[\,\.][0-9]{1,}").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            }
                        }
                    ]
                }
            ],
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
