use num::Num;
use pattern::{ParsingPattern, Patterns};

mod pattern;

pub enum Culture {
    English,
    French,
}

pub struct ConvertString {
    string_num: String,
    culture: Option<Culture>,
}

impl ConvertString {
    /// Return true is the string has been succesfully converted into number
    pub fn is_numeric(&self) -> bool {
        todo!()
    }

    /// Return true is the string has been succesfully converted into an integer
    pub fn is_integer() {
        todo!()
    }

    /// Return true is the string has been succesfully converted into a float
    pub fn is_float() {
        todo!()
    }

    /// Convert the string into an integer
    pub fn to_integer() -> i32 {
        todo!()
    }

    /// Convert the string into an float
    pub fn to_float() -> f32 {
        todo!()
    }

    pub fn get_pattern_from_culture(culture: Culture) -> Vec<ParsingPattern> {}

    fn get_current_pattern() -> Vec<ParsingPattern> {}

    pub fn find_pattern(culture: Culture) -> Option<ParsingPattern> {
        //First, we search in common pattern (not currency dependent) and currency pattern
        let patterns = Patterns::default();
        let pattern_culture = patterns
        .culture_pattern
        .into_iter()
        .filter(|c| c.value.iter().any(|cc| cc == &culture));

        let paterns_with_current_culture = patterns.common_pattern.concat(
            ,
        );
    }
}

pub struct FormatOption {
    minimum_fraction_digit: u8,
    maximum_fraction_digit: u8,
}
pub struct Number<T: Num> {
    num: T,
}

impl<T: num::Num> Number<T> {
    pub fn to_format() {
        todo!()
    }

    pub fn to_format_options(options: FormatOption) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
