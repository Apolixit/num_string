[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

## This crate perform conversion between string and number

### It allows to convert culture formated number to rust number

### It allows to display rust numbers to culture formated string

# Example string to number

## Basic string to number

``` rust
use num_string::{Culture, ConversionError, NumberConversion};
    assert_eq!("1000".to_number::<i32>().unwrap(), 1000);
    assert_eq!("+1000".to_number::<i64>().unwrap(), 1000);
    assert_eq!("-1000".to_number::<i64>().unwrap(), -1000);
    assert_eq!("1000".to_number::<f32>().unwrap(), 1000.0);
    assert_eq!("1000.5822".to_number::<f32>().unwrap(), 1000.5822);
    
    // Fail because 1000 > i8 max capacity
    assert_eq!("1000".to_number::<i8>(), Err(ConversionError::UnableToConvertStringToNumber));
```

## For more advanced conversion you can specify culture

``` rust
use num_string::{Culture, NumberConversion};     
    // Numbers with decimal separator
    assert_eq!("10.8888".to_number_culture::<f32>(Culture::English).unwrap(), 10.8888);
    assert_eq!("0,10".to_number_culture::<f32>(Culture::Italian).unwrap(), 0.1); 

    // Numbers with decimal separator and no whole part
    assert_eq!(",10".to_number_culture::<f32>(Culture::Italian).unwrap(), 0.1); 

    // Numbers with thousand separator
    assert_eq!("1,000".to_number_culture::<i32>(Culture::English).unwrap(), 1000);     

    // Numbers with thousand and decimal separator
    assert_eq!("1,000.8888".to_number_culture::<f32>(Culture::English).unwrap(), 1000.8888);
    assert_eq!("-10 564,10".to_number_culture::<f32>(Culture::French).unwrap(), -10564.10);
```

## Custom separator (DOT as thousand separator and SPACE a decimal separator)

``` rust
use num_string::{NumberCultureSettings, Separator, NumberConversion};

    assert_eq!(
            "1.000 8888"
                .to_number_separators::<f32>(NumberCultureSettings::new(
                    Separator::DOT,
                    Separator::SPACE
                ))
                .unwrap(),
            1000.8888
        );
```

# Example number to string

``` rust
use num_string::{Culture, ToFormat}; 
    // Some basic display (N0 = 0 digit, N2 = 2 digits etc)
    assert_eq!(1000.to_format("N0", Culture::English).unwrap(), "1,000");
    assert_eq!((-1000).to_format("N0", Culture::English).unwrap(), "-1,000");
    assert_eq!(1000.to_format("N2", Culture::French).unwrap(), "1 000,00");

    // Perform the round decimal
    assert_eq!(10_000.9999.to_format("N2", Culture::French).unwrap(), "10 001,00");
    assert_eq!((-10_000.999).to_format("N2", Culture::French).unwrap(), "-10 001,00");
```

# Example of number analysis

``` rust
use num_string::{ConvertString, Culture};
use num_string::pattern::TypeParsing; 
    let string_num = ConvertString::new("1,000.2", Some(Culture::English));
    assert!(string_num.is_numeric());
    assert!(string_num.is_float());
    assert!(!string_num.is_integer()); 

    // Convert to number
    assert_eq!(string_num.to_number::<f32>().unwrap(), 1000.2); 
    
    // If the conversion is ok (string_num.isNumeric() == true), you will have access to the matching pattern
    let matching_pattern = string_num.get_current_pattern().unwrap();
    assert_eq!(matching_pattern.get_regex().get_type_parsing(), &TypeParsing::DecimalThousandSeparator); 

    // If we try to convert a bad formatted number
    let string_error = ConvertString::new("NotANumber", Some(Culture::English));
    assert!(!string_error.is_numeric());
```


Feel free to fork or contact me if needed