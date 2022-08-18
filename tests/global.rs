//!  Global test case
//! An overview of the main functionalities of the crate

use num_string::{ConvertString, NumberConversion, NumberCultureSettings, ToFormat};

#[test]
fn convert_string_number_with_separator_should_work() {
    assert_eq!("100".to_number::<i32>().unwrap(), 100);
    assert_eq!("10000".to_number::<i32>().unwrap(), 10000);

    assert_eq!(
        "10 000"
            .to_number_separators::<i32>(NumberCultureSettings::new(
                num_string::Separator::SPACE,
                num_string::Separator::DOT
            ))
            .unwrap(),
        10000
    );
    assert_eq!(
        "10,000,000"
            .to_number_separators::<i32>(NumberCultureSettings::new(
                num_string::Separator::COMMA,
                num_string::Separator::DOT
            ))
            .unwrap(),
        10000000
    );

    assert_eq!(
        "10,000,000.80"
            .to_number_separators::<f32>(NumberCultureSettings::new(
                num_string::Separator::COMMA,
                num_string::Separator::DOT
            ))
            .unwrap(),
        10000000.80
    );
    assert_eq!(
        "10🥦000🥦000🦀80"
            .to_number_separators::<f32>(NumberCultureSettings::new(
                num_string::Separator::CUSTOM('🥦'),
                num_string::Separator::CUSTOM('🦀')
            ))
            .unwrap(),
        10000000.80
    );

    assert_eq!(
        "1 00 00 000.50"
            .to_number_separators::<f32>(
                NumberCultureSettings::new(
                    num_string::Separator::SPACE,
                    num_string::Separator::DOT
                )
                .with_grouping(num_string::ThousandGrouping::TwoBlock)
            )
            .unwrap(),
        10000000.5
    );
}

#[test]
fn convert_string_number_with_culture_should_work() {
    assert_eq!(
        "10 000"
            .to_number_culture::<i32>(num_string::Culture::French)
            .unwrap(),
        10000
    );
    assert_eq!(
        "10,000"
            .to_number_culture::<i32>(num_string::Culture::English)
            .unwrap(),
        10000
    );

    assert_eq!(
        "-18.888,88"
            .to_number_culture::<f32>(num_string::Culture::Italian)
            .unwrap(),
        -18888.88
    );
    assert_eq!(
        "-10,000.80"
            .to_number_culture::<f32>(num_string::Culture::English)
            .unwrap(),
        -10000.8
    );

    assert_eq!(
        "-1,00,00,000.50"
            .to_number_culture::<f32>(num_string::Culture::Indian)
            .unwrap(),
        -10000000.5
    );
    assert_eq!(
        "10,00,000"
            .to_number_culture::<i32>(num_string::Culture::Indian)
            .unwrap(),
        1000000
    );
    assert_eq!(
        "-10,00,00,000.50"
            .to_number_culture::<f32>(num_string::Culture::Indian)
            .unwrap(),
        -100000000.5
    );
    assert_eq!(
        "10,00,00,00,000"
            .to_number_culture::<i64>(num_string::Culture::Indian)
            .unwrap(),
        10_000_000_000_i64
    );
}

#[test]
fn display_number_to_string_with_separator_should_work() {
    assert_eq!(
        "10",
        10.to_format_separators(
            "N0",
            NumberCultureSettings::new(num_string::Separator::SPACE, num_string::Separator::COMMA)
        )
        .unwrap()
    );
    assert_eq!(
        "10,00",
        10.to_format_separators(
            "N2",
            NumberCultureSettings::new(num_string::Separator::SPACE, num_string::Separator::COMMA)
        )
        .unwrap()
    );
    assert_eq!(
        "1,000.00",
        1000.to_format_separators(
            "N2",
            NumberCultureSettings::new(num_string::Separator::COMMA, num_string::Separator::DOT)
        )
        .unwrap()
    );
    assert_eq!(
        "1'000.00",
        1000.to_format_separators(
            "N2",
            NumberCultureSettings::new(
                num_string::Separator::APOSTROPHE,
                num_string::Separator::DOT
            )
        )
        .unwrap()
    );

    assert_eq!(
        "10🦀000🦀001,00",
        10_000_000.9999
            .to_format_separators(
                "N2",
                NumberCultureSettings::new(
                    num_string::Separator::CUSTOM('🦀'),
                    num_string::Separator::COMMA
                )
            )
            .unwrap()
    );

    assert_eq!(
        "10,001.00",
        10_000.9999
            .to_format_separators(
                "N2",
                NumberCultureSettings::new(
                    num_string::Separator::COMMA,
                    num_string::Separator::DOT
                )
                .with_grouping(num_string::ThousandGrouping::TwoBlock)
            )
            .unwrap()
    );
    assert_eq!(
        "10,00,001.00",
        1_000_000.9999
            .to_format_separators(
                "N2",
                NumberCultureSettings::new(
                    num_string::Separator::COMMA,
                    num_string::Separator::DOT
                )
                .with_grouping(num_string::ThousandGrouping::TwoBlock)
            )
            .unwrap()
    );

    assert_eq!(
        "10,00,001.00",
        1_000_000.9999
            .to_format_separators(
                "N2",
                NumberCultureSettings::new(
                    num_string::Separator::COMMA,
                    num_string::Separator::DOT
                )
                .with_grouping(num_string::ThousandGrouping::TwoBlock)
            )
            .unwrap()
    );
    assert_eq!(
        "10,00,00,000.00",
        100_000_000_f64
            .to_format_separators(
                "N2",
                NumberCultureSettings::new(
                    num_string::Separator::COMMA,
                    num_string::Separator::DOT
                )
                .with_grouping(num_string::ThousandGrouping::TwoBlock)
            )
            .unwrap()
    );
}

#[test]
fn display_number_to_string_with_culture_should_work() {
    assert_eq!(
        "10",
        10.to_format("N0", num_string::Culture::French).unwrap()
    );
    assert_eq!(
        "10,00",
        10.to_format("N2", num_string::Culture::French).unwrap()
    );
    assert_eq!(
        "1,000.00",
        1000.to_format("N2", num_string::Culture::English).unwrap()
    );
    assert_eq!(
        "1.000,0000",
        1000.to_format("N4", num_string::Culture::Italian).unwrap()
    );

    assert_eq!(
        "10,00,00,000",
        100_000_000_i64
            .to_format("N0", num_string::Culture::Indian)
            .unwrap()
    );
    assert_eq!(
        "10,00,00,001.00",
        100_000_000.9999
            .to_format("N2", num_string::Culture::Indian)
            .unwrap()
    );

    assert_eq!(
        10_000.9999
            .to_format("N2", num_string::Culture::Indian)
            .unwrap(),
        "10,001.00"
    );

    assert_eq!(
        1000.to_format("N0", num_string::Culture::English).unwrap(),
        "1,000"
    );
}

#[test]
fn convert_number_with_given_culture_and_display_info_should_work() {
    let string_num = ConvertString::new("10,000", Some(num_string::Culture::English));

    // A parsing pattern has been found for this number
    assert!(string_num.get_current_pattern().is_some());
    assert!(string_num.is_numeric());
    assert!(string_num.is_integer());
    assert!(!string_num.is_float());
    assert_eq!(
        string_num.get_current_pattern().unwrap().name(),
        "EN_Whole_Thousand_Separator"
    );
}
