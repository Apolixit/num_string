use crate::errors::ConversionError;
use crate::Culture;
use regex::Regex;
use std::fmt::Display;

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
/// Can be thousand or decimal separator
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Separator {
    SPACE,
    DOT,
    COMMA,
}

impl Separator {
    fn to_string_regex(&self) -> String {
        match self {
            Separator::COMMA => String::from(r"[\\,]"),
            Separator::DOT => String::from(r"[\\.]"),
            Separator::SPACE => String::from(r"[\s]"),
        }
    }

    pub fn to_owned_string(&self) -> String {
        match self {
            Separator::COMMA => String::from(","),
            Separator::DOT => String::from("."),
            Separator::SPACE => String::from(" "),
        }
    }
}

/// Get string slice from Separator
impl From<Separator> for &str {
    fn from(e: Separator) -> Self {
        match e {
            Separator::COMMA => ",",
            Separator::DOT => ".",
            Separator::SPACE => " ",
        }
    }
}

/// Try get Separator from string slice
impl TryFrom<&str> for Separator {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "," => Ok(Separator::COMMA),
            "." => Ok(Separator::DOT),
            " " => Ok(Separator::SPACE),
            _ => Err(ConversionError::SeparatorNotFound),
        }
    }
}

/// The number type
#[derive(Debug, Clone, PartialEq)]
pub enum TypeParsing {
    /**
     * X / +X / -X
     */
    WholeSimple,
    /**
     * X[DecimalSeparator]XX / +X[DecimalSeparator]XX / -X[DecimalSeparator]XX
     */
    DecimalSimple,
    /**
     * [DecimalSeparator]XX / +[DecimalSeparator]XX / -[DecimalSeparator]XX
     */
    DecimalWithoutWholePart,
    /**
     * X[ThousandSeparator]XXX / +X[ThousandSeparator]XXX / -X[ThousandSeparator]XXX
     */
    WholeThousandSeparator,
    /**
     * X[ThousandSeparator]XXX[DecimalSeparator]XX / +X[ThousandSeparator]XXX[DecimalSeparator]XX / -X[ThousandSeparator]XXX[DecimalSeparator]XX
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
            TypeParsing::WholeThousandSeparator => Regex::new(
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
            ),
            TypeParsing::DecimalThousandSeparator => Regex::new(
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
            ),
        }
        .map_err(|e| ConversionError::RegexBuilder)?;

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
}

/// The parsing pattern wrapper
#[derive(Debug, Clone)]
pub struct ParsingPattern {
    name: String,
    culture_settings: Option<NumberCultureSettings>,
    regex: RegexPattern,
    number_type: NumberType,
    additional_pattern: Option<String>,
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
        // let name = format!("{}_{}", name.to_uppercase(), type_parsing);
        // let number_type = NumberType::from(type_parsing);
        // let regex_pattern = RegexPattern::new(type_parsing, culture_settings)?;

        Ok(ParsingPattern {
            name: format!("{}_{}", name.to_uppercase(), &type_parsing),
            culture_settings: culture_settings,
            regex: RegexPattern::new(&type_parsing, culture_settings)?,
            number_type: NumberType::from(&type_parsing),
            additional_pattern: None,
        })
        // let regex = RegexPattern

        // /**
        //              * X.XXX,XX
        //              * Ex: 1.000,02 / 1.025.359,0036262 / -1.000.000,230
        //              */
        //             name: String::from("IT_Thousand_Separator_Decimal"),
        //             number_type: NumberType::DECIMAL,
        //             culture_settings: Some(NumberCultureSettings::italian_culture()),
        //             additional_pattern: None,
        //             regex: RegexPattern {
        //                 prefix: Regex::new(r"^").unwrap(),
        //                 content: Regex::new(r"[\-\+]?[0-9]+([\.][0-9]{3})+[\\,][0-9]*")
        //                     .unwrap(),
        //                 suffix: Regex::new(r"$").unwrap(),
        //             },
    }

    pub fn get_regex(&self) -> &RegexPattern {
        &self.regex
    }

    pub fn get_number_type(&self) -> &NumberType {
        &self.number_type
    }
}

/// Represent the current thousand and decimal separator
#[derive(Debug, Clone, Copy)]
pub struct NumberCultureSettings {
    thousand_separator: Separator,
    decimal_separator: Separator,
}

impl NumberCultureSettings {
    /// Create a new instance
    pub fn new(
        thousand_separator: Separator,
        decimal_separator: Separator,
    ) -> NumberCultureSettings {
        NumberCultureSettings {
            thousand_separator: thousand_separator.to_owned(),
            decimal_separator: decimal_separator.to_owned(),
        }
    }

    /// Get English culture settings
    pub fn english_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::COMMA, Separator::DOT)
    }

    /// Get French culture settings
    pub fn french_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::SPACE, Separator::COMMA)
    }

    /// Get Italian culture settings
    pub fn italian_culture() -> NumberCultureSettings {
        NumberCultureSettings::new(Separator::DOT, Separator::COMMA)
    }

    pub fn to_thousand_separator(&self) -> &Separator {
        &self.thousand_separator
    }

    pub fn to_thousand_separator_string(&self) -> String {
        self.thousand_separator.to_owned_string()
    }

    pub fn to_decimal_separator(&self) -> &Separator {
        &self.decimal_separator
    }

    pub fn to_decimal_separator_string(&self) -> String {
        self.decimal_separator.to_owned_string()
    }
}

impl From<(&str, &str)> for NumberCultureSettings {
    fn from(val: (&str, &str)) -> Self {
        NumberCultureSettings::new(
            Separator::try_from(val.0).unwrap(),
            Separator::try_from(val.1).unwrap(),
        )
    }
}

/// Get the culture settings from current culture
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

        // French pattern
        patterns.add_culture_pattern(
            CulturePattern::new("fr", NumberCultureSettings::french_culture()).unwrap(),
        );

        // English pattern
        patterns.add_culture_pattern(
            CulturePattern::new("en", NumberCultureSettings::english_culture()).unwrap(),
        );

        // Italian pattern
        patterns.add_culture_pattern(
            CulturePattern::new("it", NumberCultureSettings::italian_culture()).unwrap(),
        );

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::NumberPatterns;
    use super::NumberType;
    use super::Separator;
    use crate::errors::ConversionError;
    use crate::pattern::TypeParsing;
    use crate::Culture;
    use crate::CulturePattern;
    use crate::NumberCultureSettings;
    use regex::Regex;

    #[test]
    fn test_number_type() {
        assert_eq!(NumberType::DECIMAL, NumberType::from(&TypeParsing::DecimalSimple));
        assert_eq!(NumberType::DECIMAL, NumberType::from(&TypeParsing::DecimalThousandSeparator));
        assert_eq!(NumberType::DECIMAL, NumberType::from(&TypeParsing::DecimalWithoutWholePart));
        assert_eq!(NumberType::WHOLE, NumberType::from(&TypeParsing::WholeSimple));
    }

    #[test]
    fn test_regex() {
        let r = Regex::new(r"[\-\+]?\d+([0-9]{3})*").unwrap();
        assert!(r.is_match("10,2"));
    }

    #[test]
    fn test_separator() {
        let comma_str: &str = Separator::COMMA.into();
        assert_eq!(",", comma_str);
        assert_eq!(Separator::SPACE,  Separator::try_from(" ").unwrap());
        assert_eq!(Err(ConversionError::SeparatorNotFound), Separator::try_from("i_am_not_well_formatted"));

        assert_eq!(Separator::DOT.to_owned_string(), String::from("."));

        assert_eq!(Separator::COMMA.to_string_regex(), String::from(r"[\\,]"));
        assert_eq!(Separator::DOT.to_string_regex(), String::from(r"[\\.]"));
        assert_eq!(Separator::SPACE.to_string_regex(), String::from(r"[\s]"));
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
    fn test_generated_regex() {
        let french_culture =
            CulturePattern::new("fr", NumberCultureSettings::french_culture()).unwrap();
        let english_culture =
            CulturePattern::new("en", NumberCultureSettings::english_culture()).unwrap();
        let italian_culture =
            CulturePattern::new("it", NumberCultureSettings::italian_culture()).unwrap();

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
            r"[\-\+]?[0-9]+[\\,][0-9]{1,}",
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
            r"[\-\+]?[\\,][0-9]+",
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
            r"[\-\+]?[0-9]+([\s][0-9]{3})+[\\,][0-9]*",
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
            r"[\-\+]?[0-9]+[\\.][0-9]{1,}",
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
            r"[\-\+]?[\\.][0-9]+",
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
            r"[\-\+]?[0-9]+([\\,][0-9]{3})+",
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
            r"[\-\+]?[0-9]+([\\,][0-9]{3})+[\\.][0-9]*",
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
            r"[\-\+]?[0-9]+[\\,][0-9]{1,}",
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
            r"[\-\+]?[\\,][0-9]+",
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
            r"[\-\+]?[0-9]+([\\.][0-9]{3})+",
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
            r"[\-\+]?[0-9]+([\\.][0-9]{3})+[\\,][0-9]*",
            "Error italian culture DecimalThousandSeparator"
        );
    }
}
