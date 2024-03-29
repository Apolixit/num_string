use crate::errors::ConversionError;
use crate::string_to_number::NumberConversion;
use crate::Culture;
use log::{info, warn};
use regex::{Regex, escape};
use std::fmt::Display;
use std::str::FromStr;

/// Represent if the number is Whole (int), or Decimal (float)
#[derive(Debug, Clone, PartialEq)]
pub enum NumberType {
    WHOLE,
    DECIMAL,
}

impl From<&TypeParsing> for NumberType {
    fn from(type_parsing: &TypeParsing) -> Self {
        match type_parsing {
            TypeParsing::WholeSimple | TypeParsing::WholeThousandSeparator => NumberType::WHOLE,
            TypeParsing::DecimalSimple
            | TypeParsing::DecimalThousandSeparator
            | TypeParsing::DecimalWithoutWholePart => NumberType::DECIMAL,
        }
    }
}

/// Represent commons separators.
///
/// Can be thousand or decimal separator.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Separator {
    SPACE,
    DOT,
    COMMA,
    APOSTROPHE,
    CUSTOM(char)
}

impl Separator {
    fn to_string_regex(&self) -> String {
        format!("[{}]", match self {
            Separator::COMMA => escape(","),
            Separator::DOT => escape("."),
            Separator::SPACE => r"\s".to_string(),
            Separator::APOSTROPHE => escape("'"),
            Separator::CUSTOM(c) => escape(c.to_string().as_str())
        })
    }

    pub fn to_owned_string(&self) -> String {
        (*self).into()
    }
}

// /// Get string slice from Separator
// impl From<Separator> for &str {
//     fn from(e: Separator) -> Self {
//         let s: String = e.into();
//         s.as_str().c
//         // match e {
//         //     Separator::COMMA => Cow::Borrowed(",").borrow(),
//         //     Separator::DOT => ".",
//         //     Separator::SPACE => " ",
//         //     Separator::APOSTROPHE => "'",
//         //     Separator::CUSTOM(c) => &c.to_string(),
//         // }
//     }
// }

/// Get owned string from Separator
impl From<Separator> for String {
    fn from(e: Separator) -> Self {
        match e {
            Separator::COMMA => ",".to_owned(),
            Separator::DOT => ".".to_owned(),
            Separator::SPACE => " ".to_owned(),
            Separator::APOSTROPHE => "'".to_owned(),
            Separator::CUSTOM(c) => c.to_string(),
        }
    }
}

/// Get char from separator
impl From<Separator> for char {
    fn from(e: Separator) -> Self {
        match e {
            Separator::COMMA => ',',
            Separator::DOT => '.',
            Separator::SPACE => ' ',
            Separator::APOSTROPHE => '\'',
            Separator::CUSTOM(c) => c,
        }
    }
}

/// Try get Separator from string slice
impl TryFrom<&'static str> for Separator {
    type Error = ConversionError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        match value {
            "," => Ok(Separator::COMMA),
            "." => Ok(Separator::DOT),
            " " => Ok(Separator::SPACE),
            // I'm pretty sure we can have a huge better syntax here...
            s if s.len() == 1 => Ok(Separator::CUSTOM(s.to_string().chars().collect::<Vec<char>>()[0])),
            _ => Err(ConversionError::SeparatorNotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThousandGrouping {
    /// The standard grouping is the most common thousand split. We group the number by blocks of 3
    /// Ex : X XXX XXX XXX
    ThreeBlock,
    /// Indian thousand split
    TwoBlock
}

/// To be compatible with thousands crate
impl From<ThousandGrouping> for &[u8] {
    fn from(val: ThousandGrouping) -> Self {
        match val {
            ThousandGrouping::ThreeBlock => &[3],
            ThousandGrouping::TwoBlock => &[3, 2],
        }
    }
}

/// The type of parsing. Represent all kind of basic number format
#[derive(Debug, Clone, PartialEq)]
pub enum TypeParsing {
    /**
     * X / +X / -X
     */
    WholeSimple,
    /**
     * X|DecimalSeparator|XX / +X|DecimalSeparator|XX / -X|DecimalSeparator|XX
     */
    DecimalSimple,
    /**
     * |DecimalSeparator|XX / +|DecimalSeparator|XX / -|DecimalSeparator|XX
     */
    DecimalWithoutWholePart,
    /**
     * X|ThousandSeparator|XXX / +X|ThousandSeparator|XXX / -X|ThousandSeparator|XXX
     */
    WholeThousandSeparator,
    /**
     * X|ThousandSeparator|XXX|DecimalSeparator|XX / +X|ThousandSeparator|XXX|DecimalSeparator|XX / -X|ThousandSeparator|XXX|DecimalSeparator|XX
     */
    DecimalThousandSeparator,
}

impl Display for TypeParsing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let name = match self {
            Self::WholeSimple => "Whole_Simple",
            Self::DecimalSimple => "Decimal_Simple",
            Self::DecimalWithoutWholePart => "Decimal_Without_Whole_Part",
            Self::WholeThousandSeparator => "Whole_Thousand_Separator",
            Self::DecimalThousandSeparator => "Decimal_Thousand_Separator",
        };

        write!(f, "{}", name)
    }
}

/// Regex use to try to convert string to number
#[derive(Debug, Clone)]
pub struct RegexPattern {
    type_parsing: TypeParsing,
    prefix: Regex,
    content: Regex,
    suffix: Regex,
}

impl RegexPattern {
    pub fn new(
        type_parsing: &TypeParsing,
        culture_settings: Option<NumberCultureSettings>,
    ) -> Result<RegexPattern, ConversionError> {
        if type_parsing != &TypeParsing::WholeSimple && culture_settings.is_none() {
            panic!("The regex pattern need to have culture settings set");
        }

        //Indian
        // ^[\-\+]?([0-9]{0,3})([,][0-9]{2})*([,][0-9]{3}){1}

        let regex_content = match type_parsing {
            TypeParsing::WholeSimple => Regex::new(r"[\-\+]?\d+([0-9]{3})*"),
            TypeParsing::DecimalSimple => Regex::new(
                format!(
                    "{}{}{}",
                    r"[\-\+]?[0-9]+",
                    culture_settings
                        .unwrap()
                        .decimal_separator
                        .to_string_regex(),
                    r"[0-9]{1,}"
                )
                .as_str(),
            ),
            TypeParsing::DecimalWithoutWholePart => Regex::new(
                format!(
                    "{}{}{}",
                    r"[\-\+]?",
                    culture_settings
                        .unwrap()
                        .decimal_separator
                        .to_string_regex(),
                    "[0-9]+"
                )
                .as_str(),
            ),
            TypeParsing::WholeThousandSeparator => {
                match culture_settings.unwrap().thousand_grouping {
                    ThousandGrouping::ThreeBlock => {
                        Regex::new(
                            format!(
                                "{}({}{})+",
                                r"[\-\+]?[0-9]+",
                                culture_settings
                                    .unwrap()
                                    .thousand_separator
                                    .to_string_regex(),
                                r"[0-9]{3}"
                            )
                            .as_str(),
                        )
                    },
                    ThousandGrouping::TwoBlock => {
                        Regex::new(
                            format!("{}{}{}{}{}", r"[\-\+]?([0-9]{0,3})(", culture_settings
                            .unwrap()
                            .thousand_separator
                            .to_string_regex(), r"[0-9]{2})*(", culture_settings
                            .unwrap()
                            .thousand_separator
                            .to_string_regex(), r"[0-9]{3}){1}")
                            .as_str(),
                        )
                    },
                }
            },
            TypeParsing::DecimalThousandSeparator => {
                // [\-\+]?([0-9]{0,3})([,][0-9]{2})*([,][0-9]{3}){1}
                match culture_settings.unwrap().thousand_grouping {
                    ThousandGrouping::ThreeBlock => {
                        Regex::new(
                            format!(
                                "{}({}{})+{}[0-9]*",
                                r"[\-\+]?[0-9]+",
                                culture_settings
                                    .unwrap()
                                    .thousand_separator
                                    .to_string_regex(),
                                r"[0-9]{3}",
                                culture_settings
                                    .unwrap()
                                    .decimal_separator
                                    .to_string_regex()
                            )
                            .as_str(),
                        )
                    },
                    ThousandGrouping::TwoBlock => {
                        Regex::new(
                            format!("{}{}{}{}{}{}[0-9]*", r"[\-\+]?([0-9]{0,3})(", culture_settings
                            .unwrap()
                            .thousand_separator
                            .to_string_regex(), r"[0-9]{2})*(", culture_settings
                            .unwrap()
                            .thousand_separator
                            .to_string_regex(), r"[0-9]{3}){1}", culture_settings
                            .unwrap()
                            .decimal_separator
                            .to_string_regex())
                            .as_str(),
                        )
                    },
                }

            },
        }
        .map_err(|_| ConversionError::RegexBuilder)?;

        Ok(RegexPattern {
            type_parsing: type_parsing.to_owned(),
            prefix: Regex::new(r"^").unwrap(),
            content: regex_content,
            suffix: Regex::new(r"$").unwrap(),
        })
    }

    /// Return if the string number has been matched by the regex
    pub fn is_match(&self, text: &str) -> bool {
        let full_regex =
            Regex::new(format!("{}{}{}", self.prefix, self.content, self.suffix).as_str()).unwrap();
        full_regex.is_match(text)
    }

    pub fn get_type_parsing(&self) -> &TypeParsing {
        &self.type_parsing
    }

    pub fn get_regex(&self) -> Regex {
        Regex::new(format!("{}{}{}", self.prefix, self.content, self.suffix).as_str()).unwrap()
    }
}


/// The parsing pattern wrapper
#[derive(Debug, Clone)]
pub struct ParsingPattern {
    name: String,
    regex: RegexPattern,
    number_type: NumberType,
}

impl Display for ParsingPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", &self.name)
    }
}

impl ParsingPattern {
    pub fn build(
        name: String,
        type_parsing: TypeParsing,
        culture_settings: Option<NumberCultureSettings>,
    ) -> Result<ParsingPattern, ConversionError> {
        Ok(ParsingPattern {
            name: format!("{}_{}", name.to_uppercase(), &type_parsing),
            regex: RegexPattern::new(&type_parsing, culture_settings)?,
            number_type: NumberType::from(&type_parsing),
        })
    }

    pub fn get_regex(&self) -> &RegexPattern {
        &self.regex
    }

    pub fn get_number_type(&self) -> &NumberType {
        &self.number_type
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

/// Represent the current thousand and decimal separator
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberCultureSettings {
    thousand_separator: Separator,
    decimal_separator: Separator,
    thousand_grouping: ThousandGrouping,
}

impl NumberCultureSettings {
    /// Create a new instance
    pub fn new(
        thousand_separator: Separator,
        decimal_separator: Separator,
    ) -> NumberCultureSettings {
        assert!(thousand_separator != decimal_separator);

        NumberCultureSettings {
            thousand_separator,
            decimal_separator,
            thousand_grouping: ThousandGrouping::ThreeBlock,
        }
    }

    /// Set the thousand grouping value (didn't want to expose it in the constructor)
    pub fn with_grouping(mut self, thousand_grouping: ThousandGrouping) -> Self {
        self.thousand_grouping = thousand_grouping;
        self
    }

    pub fn thousand_separator(&self) -> Separator {
        self.thousand_separator
    }

    pub fn into_thousand_separator_string(&self) -> String {
        self.thousand_separator.to_owned_string()
    }

    pub fn into_thousand_separator_regex(&self) -> String {
        self.thousand_separator.to_string_regex()
    }

    pub fn decimal_separator(&self) -> Separator {
        self.decimal_separator
    }

    pub fn into_decimal_separator_string(&self) -> String {
        self.decimal_separator.to_owned_string()
    }

    pub fn into_decimal_separator_regex(&self) -> String {
        self.decimal_separator.to_string_regex()
    }

    pub fn thousand_grouping(&self) -> ThousandGrouping {
        self.thousand_grouping
    }
}


impl From<(&'static str, &'static str)> for NumberCultureSettings {
    fn from(val: (&'static str, &'static str)) -> Self {
        NumberCultureSettings::new(
            Separator::try_from(val.0).unwrap(),
            Separator::try_from(val.1).unwrap())
    }
}

/// Get the culture settings from current culture
impl From<Culture> for NumberCultureSettings {
    fn from(culture: Culture) -> Self {
        match culture {
            Culture::English => NumberCultureSettings::new(Separator::COMMA, Separator::DOT),
            Culture::French => NumberCultureSettings::new(Separator::SPACE, Separator::COMMA),
            Culture::Italian => NumberCultureSettings::new(Separator::DOT, Separator::COMMA),
            Culture::Indian => NumberCultureSettings::new(Separator::COMMA, Separator::DOT).with_grouping(ThousandGrouping::TwoBlock),
        }
    }
}

/// The pattern which is culture dependent. Allow us to try to parse multi culture string
#[derive(Debug, Clone)]
pub struct CulturePattern {
    name: String,
    value: Culture,
    patterns: Vec<ParsingPattern>,
}

impl CulturePattern {
    /// Create a new language pattern
    /// This struct is use to parse a string number from the given culture
    pub fn new(
        name: &str,
        culture_settings: NumberCultureSettings,
    ) -> Result<CulturePattern, ConversionError> {
        Ok(CulturePattern {
            name: String::from(name),
            value: name.try_into().unwrap(),
            patterns: vec![
                ParsingPattern::build(
                    String::from(name),
                    TypeParsing::DecimalSimple,
                    Some(culture_settings),
                )
                .unwrap(),
                ParsingPattern::build(
                    String::from(name),
                    TypeParsing::DecimalWithoutWholePart,
                    Some(culture_settings),
                )
                .unwrap(),
                ParsingPattern::build(
                    String::from(name),
                    TypeParsing::WholeThousandSeparator,
                    Some(culture_settings),
                )
                .unwrap(),
                ParsingPattern::build(
                    String::from(name),
                    TypeParsing::DecimalThousandSeparator,
                    Some(culture_settings),
                )
                .unwrap(),
            ],
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_culture(&self) -> &Culture {
        &self.value
    }

    pub fn get_patterns(&self) -> &Vec<ParsingPattern> {
        &self.patterns
    }
}

/// All pattern defined to try to convert string to number
pub struct NumberPatterns {
    common_pattern: Vec<ParsingPattern>,
    culture_pattern: Vec<CulturePattern>,
    math_pattern: Vec<ParsingPattern>,
}

impl NumberPatterns {
    pub fn new() -> NumberPatterns {
        NumberPatterns::default()
    }

    /// Return all culture pattern
    pub fn get_all_culture_pattern(&self) -> Vec<CulturePattern> {
        self.culture_pattern.to_vec()
    }

    /// Try to return the culture pattern from the following culture
    pub fn get_culture_pattern(&self, culture: &Culture) -> Option<CulturePattern> {
        self.get_all_culture_pattern()
            .into_iter()
            .find(|c| c.get_culture() == culture)
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

impl Default for NumberPatterns {
    fn default() -> Self {
        let mut patterns = NumberPatterns {
            common_pattern: vec![],
            culture_pattern: vec![],
            math_pattern: vec![],
        };

        // Common pattern which is not culture dependent
        patterns.add_common_pattern(
            ParsingPattern::build(String::from("Common"), TypeParsing::WholeSimple, None).unwrap(),
        );

        // Loop over culture enum
        for culture in enum_iterator::all::<Culture>().collect::<Vec<Culture>>().into_iter() {
            patterns.add_culture_pattern(CulturePattern::new(culture.into(), culture.into()).unwrap())
        }

        patterns
    }
}

/// Structure to convert a string to number
pub struct ConvertString {
    string_num: String,
    culture: Option<Culture>,
    all_patterns: NumberPatterns,
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
    fn load_patterns() -> NumberPatterns {
        NumberPatterns::default()
    }

    /// Return the pattern selected for conversion
    pub fn get_current_pattern(&self) -> Option<ParsingPattern> {
        ConvertString::find_pattern(
            &self.string_num,
            &self.culture.unwrap_or_default(),
            &self.all_patterns,
        )
    }

    /// Get culture pattern from culture
    pub fn find_culture_pattern(
        culture: &Culture,
        patterns: &NumberPatterns,
    ) -> Option<CulturePattern> {
        patterns
            .get_all_culture_pattern()
            .into_iter()
            .find(|c| c.get_culture() == culture)
    }

    /// Find a matching pattern for the given string num
    pub fn find_pattern(
        string_num: &str,
        culture: &Culture,
        patterns: &NumberPatterns,
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
            .find(|p| p.get_regex().is_match(string_num))
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
            return pp.get_number_type() == &NumberType::WHOLE;
        }

        false
    }

    /// Return true is the string has been succesfully converted into a float
    pub fn is_float(&self) -> bool {
        if let Some(pp) = self.get_current_pattern() {
            return pp.get_number_type() == &NumberType::DECIMAL;
        }

        false
    }

    pub fn to_number<N: num::Num + Display + FromStr>(&self) -> Result<N, ConversionError> {
        if let Some(culture) = self.culture {
            self.string_num.as_str().to_number_culture::<N>(culture)
        } else {
            self.string_num.as_str().to_number::<N>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NumberPatterns;
    use super::NumberType;
    use super::Separator;
    use crate::errors::ConversionError;
    use crate::pattern::ConvertString;
    use crate::pattern::CulturePattern;
    use crate::pattern::TypeParsing;
    use crate::Culture;
    use crate::NumberCultureSettings;
    use regex::Regex;

    #[test]
    fn test_number_type() {
        assert_eq!(
            NumberType::DECIMAL,
            NumberType::from(&TypeParsing::DecimalSimple)
        );
        assert_eq!(
            NumberType::DECIMAL,
            NumberType::from(&TypeParsing::DecimalThousandSeparator)
        );
        assert_eq!(
            NumberType::DECIMAL,
            NumberType::from(&TypeParsing::DecimalWithoutWholePart)
        );
        assert_eq!(
            NumberType::WHOLE,
            NumberType::from(&TypeParsing::WholeSimple)
        );
    }

    #[test]
    fn test_regex() {
        let r = Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap();
        assert!(r.is_match("10,2"));
    }

    #[test]
    fn test_separator() {
        let comma_str: char = Separator::COMMA.into();
        assert_eq!(',', comma_str);
        assert_eq!(Separator::SPACE, " ".try_into().unwrap());
        assert_eq!(
            Err(ConversionError::SeparatorNotFound),
            Separator::try_from("i_am_not_well_formatted")
        );

        assert_eq!(Separator::DOT.to_owned_string(), String::from("."));

        assert_eq!(Separator::COMMA.to_string_regex(), String::from("[,]"));
        assert_eq!(Separator::DOT.to_string_regex(), String::from("[\\.]"));
        assert_eq!(Separator::SPACE.to_string_regex(), String::from(r"[\s]"));
        assert_eq!(Separator::SPACE.to_string_regex(), String::from("[\\s]"));
    }

    #[test]
    fn test_parsing_pattern_fr() {
        let optionnal_fr_pattern = NumberPatterns::default().get_culture_pattern(&Culture::French);

        //We need to have an fr pattern
        assert!(optionnal_fr_pattern.is_some());
        let fr_pattern = optionnal_fr_pattern.unwrap();
        assert_eq!(fr_pattern.get_name(), "fr");
        assert!(fr_pattern.get_patterns().len() > 0);
    }

    #[test]
    fn test_parsing_pattern_en() {
        let optionnal_en_pattern = NumberPatterns::default().get_culture_pattern(&Culture::English);

        //We need to have an en pattern
        assert!(optionnal_en_pattern.is_some());
        let en_pattern = optionnal_en_pattern.unwrap();
        assert_eq!(en_pattern.get_name(), "en");
        assert!(en_pattern.get_patterns().len() > 0);
    }

    #[test]
    fn test_parsing_pattern_it() {
        let optionnal_en_pattern = NumberPatterns::default().get_culture_pattern(&Culture::Italian);

        //We need to have an it pattern
        assert!(optionnal_en_pattern.is_some());
        let en_pattern = optionnal_en_pattern.unwrap();
        assert_eq!(en_pattern.get_name(), "it");
        assert!(en_pattern.get_patterns().len() > 0);
    }

    #[test]
    fn test_generated_regex_culture() {
        let french_culture =
            CulturePattern::new("fr", Culture::French.into()).unwrap();
        let english_culture =
            CulturePattern::new("en", Culture::English.into()).unwrap();
        let italian_culture =
            CulturePattern::new("it", Culture::Italian.into()).unwrap();

        assert_eq!(french_culture.get_name(), "fr");
        assert_eq!(english_culture.get_name(), "en");
        assert_eq!(italian_culture.get_name(), "it");

        assert_eq!(french_culture.get_culture(), &Culture::French);
        assert_eq!(english_culture.get_culture(), &Culture::English);
        assert_eq!(italian_culture.get_culture(), &Culture::Italian);

        let fr_decimal_simple = french_culture
            .get_patterns()
            .into_iter()
            .find(|f| f.regex.type_parsing == TypeParsing::DecimalSimple)
            .unwrap();
        assert_eq!(fr_decimal_simple.name, String::from("FR_Decimal_Simple"));
        assert_eq!(
            fr_decimal_simple.regex.content.as_str(),
            r"[\-\+]?[0-9]+[,][0-9]{1,}",
            "Error french culture DecimalSimple"
        );

        assert_eq!(
            french_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalWithoutWholePart)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[,][0-9]+",
            "Error french culture DecimalWithoutWholePart"
        );
        assert_eq!(
            french_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::WholeThousandSeparator)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+([\s][0-9]{3})+",
            "Error french culture WholeThousandSeparator"
        );
        assert_eq!(
            french_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalThousandSeparator)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+([\s][0-9]{3})+[,][0-9]*",
            "Error french culture DecimalThousandSeparator"
        );

        assert_eq!(
            english_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalSimple)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+[\.][0-9]{1,}",
            "Error english culture DecimalSimple"
        );
        assert_eq!(
            english_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalWithoutWholePart)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[\.][0-9]+",
            "Error english culture DecimalWithoutWholePart"
        );

        let en_whole_thousand_separator = english_culture
            .get_patterns()
            .into_iter()
            .find(|f| f.regex.type_parsing == TypeParsing::WholeThousandSeparator)
            .unwrap();
        assert_eq!(
            en_whole_thousand_separator.name,
            String::from("EN_Whole_Thousand_Separator")
        );
        assert_eq!(
            en_whole_thousand_separator.regex.content.as_str(),
            r"[\-\+]?[0-9]+([,][0-9]{3})+",
            "Error english culture WholeThousandSeparator"
        );
        assert_eq!(
            english_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalThousandSeparator)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+([,][0-9]{3})+[\.][0-9]*",
            "Error english culture DecimalThousandSeparator"
        );

        assert_eq!(
            italian_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalSimple)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+[,][0-9]{1,}",
            "Error italian culture DecimalSimple"
        );
        assert_eq!(
            italian_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::DecimalWithoutWholePart)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[,][0-9]+",
            "Error italian culture DecimalWithoutWholePart"
        );
        assert_eq!(
            italian_culture
                .get_patterns()
                .into_iter()
                .find(|f| f.regex.type_parsing == TypeParsing::WholeThousandSeparator)
                .unwrap()
                .regex
                .content
                .as_str(),
            r"[\-\+]?[0-9]+([\.][0-9]{3})+",
            "Error italian culture WholeThousandSeparator"
        );

        let it_decimal_thousand_separator = italian_culture
            .get_patterns()
            .into_iter()
            .find(|f| f.regex.type_parsing == TypeParsing::DecimalThousandSeparator)
            .unwrap();
        assert_eq!(
            it_decimal_thousand_separator.name,
            String::from("IT_Decimal_Thousand_Separator")
        );
        assert_eq!(
            it_decimal_thousand_separator.regex.content.as_str(),
            r"[\-\+]?[0-9]+([\.][0-9]{3})+[,][0-9]*",
            "Error italian culture DecimalThousandSeparator"
        );
    }

    // #[test]
    // fn test_number_culture_settings() {
    //     assert_eq!(
    //         NumberCultureSettings::from((" ", ",", ThousandGrouping::ThreeBlock)),
    //         Culture::french_culture()
    //     );
    //     assert_eq!(
    //         NumberCultureSettings::from(Culture::English),
    //         Culture::english_culture()
    //     );
    // }

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

            let to_integer = convert.to_number::<i32>();
            if number_type == NumberType::WHOLE {
                assert!(to_integer.is_ok(), "to_number() return Err instead of Ok");
                assert_eq!(
                    convert.to_number::<i32>().unwrap(),
                    int_value,
                    "to_integer() conversion failed for {}",
                    string_num
                );
            } else {
                assert!(to_integer.is_err(), "to_number() return Ok instead of Err");
                assert_eq!(
                    convert.to_number::<i32>(),
                    Err(ConversionError::UnableToConvertStringToNumber)
                );
            }

            let to_float = convert.to_number::<f32>();
            assert!(to_float.is_ok(), "to_float() return Err instead of Ok");
            assert_eq!(
                convert.to_number::<f32>().unwrap(),
                float_value,
                "to_float() conversion failed for {}",
                string_num
            );
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
    fn number_culture_settings_regex() {
        // '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' | '#' | '&' | '-' | '~'
        let basic1 = NumberCultureSettings::new(Separator::CUSTOM('|'), Separator::DOT);

        // assert_eq!(String::from("$"), basic1.into_thousand_separator_regex());
        log::info!("{}", basic1.into_thousand_separator_regex());
    }
}
