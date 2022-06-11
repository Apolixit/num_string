use crate::errors::ConversionError;
use crate::number_conversion::FloatConversion;
use crate::number_conversion::IntegerConversion;
use crate::number_conversion::StringNumber;
use crate::Culture;
use crate::Number;
use regex::Regex;
use std::fmt::Display;

/// Represent if the number is Whole (int), or Decimal (float)
#[derive(Debug, Clone, PartialEq)]
pub enum NumberType {
    WHOLE,
    DECIMAL,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Separator {
    SPACE,
    DOT,
    COMMA,
}

impl From<Separator> for String {
    fn from(e: Separator) -> Self {
        match e {
            Separator::COMMA => String::from(","),
            Separator::DOT => String::from("."),
            Separator::SPACE => String::from(" "),
        }
    }
}

impl TryFrom<&str> for Separator {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "," => Ok(Separator::COMMA),
            "." => Ok(Separator::DOT),
            " " => Ok(Separator::SPACE),
            _ => Err(ConversionError::SeparatorNotFound)
        }
    }
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
}

impl Display for ParsingPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", &self.name)
    }
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
    pub fn new(thousand_separator: String, decimal_separator: String) -> NumberCultureSettings {
        //TODO : Check separator here

        NumberCultureSettings {
            thousand_separator: thousand_separator.to_owned(),
            decimal_separator: decimal_separator.to_owned(),
        }
    }

    pub fn english_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::COMMA.into(), Separator::DOT.into())
    }

    pub fn french_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::SPACE.into(), Separator::COMMA.into())
    }

    pub fn italian_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::DOT.into(), Separator::COMMA.into())
    }

    pub fn to_thousand_separator(&self) -> Separator {
        self.thousand_separator.as_str().try_into().unwrap()
    }

    pub fn to_decimal_separator(&self) -> Separator {
        self.decimal_separator.as_str().try_into().unwrap()
    }
}

impl From<(&str, &str)> for NumberCultureSettings {
    fn from(val: (&str, &str)) -> Self {
        NumberCultureSettings::new(String::from(val.0), String::from(val.1))
    }
}

impl From<Culture> for NumberCultureSettings {
    fn from(culture: Culture) -> Self {
        match culture {
            Culture::English => NumberCultureSettings::english_culture(),
            Culture::French => NumberCultureSettings::french_culture(),
            Culture::Italian => NumberCultureSettings::italian_culture(),
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

    pub fn get_all_culture_pattern(&self) -> Vec<CulturePattern> {
        self.culture_pattern.to_vec()
    }

    pub fn get_culture_pattern(&self, culture: &Culture) -> Option<CulturePattern> {
        self.get_all_culture_pattern().into_iter().find(|c| {
            c.get_cultures()
                .into_iter()
                .any(|sub_culture| sub_culture == culture)
        })
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
            common_pattern: vec![ParsingPattern {
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
            }],
            culture_pattern: vec![
                CulturePattern {
                    name: String::from("fr"),
                    value: vec![Culture::French],

                    patterns: vec![
                        // French parser
                        ParsingPattern {
                            /*
                             * X,XX
                             * Ex: 1,2 / 0,35 / 1545456465000,25465
                             */
                            name: String::from("FR_Decimal_Simple"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::french_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+[\\,][0-9]{1,}").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /*
                             * .XX
                             * Ex: .25 / ,25
                             */
                            name: String::from("FR_Decimal_Without_Whole_Part"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::french_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[\\,][0-9]+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X XXX
                             *Ex : 1 000 / 1 025 359 / -1 000 / +1 000
                             */
                            name: String::from("FR_Thousand_Separator"),
                            number_type: NumberType::WHOLE,
                            culture_settings: Some(NumberCultureSettings::french_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\s][0-9]{3})+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X XXX,XX et X XXX.XX
                             *Ex : 1 000,02 / 1 025 359,00 / 1 000,00066564564654654 / +1 000.20
                             */
                            name: String::from("FR_Thousand_Separator_Decimal"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::french_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\s][0-9]{3})+[\\,][0-9]*")
                                    .unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                    ],
                },
                // English parser
                CulturePattern {
                    name: String::from("en"),
                    value: vec![Culture::English],
                    patterns: vec![
                        ParsingPattern {
                            /**
                             * X.XX (culture fr-FR + it-IT)
                             * Ex: 1.2 / 0.35 / 1545456465000.25465
                             */
                            name: String::from("EN_Decimal_Simple"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::english_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+\.[0-9]{1,}").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * .XX
                             * Ex: .25
                             */
                            name: String::from("EN_Decimal_Without_Whole_Part"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::english_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[\.][0-9]+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X,XXX
                             *Ex : 1,000 / 1,025,359 / -1,252
                             */
                            name: String::from("EN_Thousand_Separator"),
                            number_type: NumberType::WHOLE,
                            culture_settings: Some(NumberCultureSettings::english_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\\,][0-9]{3})+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X,XXX.XX (culture en-EN)
                             * Ex: 1,000.00 / +1,000.00
                             */
                            name: String::from("EN_Thousand_Separator_Decimal"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::english_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\\,][0-9]{3})+\.[0-9]*")
                                    .unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                    ],
                },
                // Italian parser
                CulturePattern {
                    name: String::from("it"),
                    value: vec![Culture::Italian],

                    patterns: vec![
                        ParsingPattern {
                            /**
                             * X,XX et X.XX
                             * Ex: 1,2 / 0,35 / 1545456465000,25465
                             */
                            name: String::from("IT_Decimal_Simple"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::italian_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+[\\,][0-9]{1,}").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /*
                             * .XX
                             * Ex: ,25
                             */
                            name: String::from("IT_Decimal_Without_Whole_Part"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::italian_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[\\,][0-9]+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X.XXX
                             * Ex: 1.009 / +1.000.000
                             */
                            name: String::from("IT_Thousand_Separator"),
                            number_type: NumberType::WHOLE,
                            culture_settings: Some(NumberCultureSettings::italian_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\.][0-9]{3})+").unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                        ParsingPattern {
                            /**
                             * X.XXX,XX
                             * Ex: 1.000,02 / 1.025.359,0036262 / -1.000.000,230
                             */
                            name: String::from("IT_Thousand_Separator_Decimal"),
                            number_type: NumberType::DECIMAL,
                            culture_settings: Some(NumberCultureSettings::italian_culture()),
                            additional_pattern: None,
                            regex: RegexPattern {
                                prefix: Regex::new(r"^").unwrap(),
                                content: Regex::new(r"[\-\+]?[0-9]+([\.][0-9]{3})+[\\,][0-9]*")
                                    .unwrap(),
                                suffix: Regex::new(r"$").unwrap(),
                            },
                        },
                    ],
                },
            ],
            math_pattern: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Patterns;
    use crate::Culture;
    use regex::Regex;

    #[test]
    fn test_regex() {
        let r = Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap();
        assert!(r.is_match("10,2"));
    }

    #[test]
    fn test_parsing_pattern_fr() {
        let optionnal_fr_pattern = Patterns::default().get_culture_pattern(&Culture::French);

        //We need to have an fr pattern
        assert!(optionnal_fr_pattern.is_some());
        let fr_pattern = optionnal_fr_pattern.unwrap();
        assert_eq!(fr_pattern.get_name(), "fr");
        assert!(fr_pattern.get_patterns().len() > 0);
    }

    #[test]
    fn test_parsing_pattern_en() {
        let optionnal_en_pattern = Patterns::default().get_culture_pattern(&Culture::English);

        //We need to have an en pattern
        assert!(optionnal_en_pattern.is_some());
        let en_pattern = optionnal_en_pattern.unwrap();
        assert_eq!(en_pattern.get_name(), "en");
        assert!(en_pattern.get_patterns().len() > 0);
    }

    #[test]
    fn test_parsing_pattern_it() {
        let optionnal_en_pattern = Patterns::default().get_culture_pattern(&Culture::Italian);

        //We need to have an it pattern
        assert!(optionnal_en_pattern.is_some());
        let en_pattern = optionnal_en_pattern.unwrap();
        assert_eq!(en_pattern.get_name(), "it");
        assert!(en_pattern.get_patterns().len() > 0);
    }
}
