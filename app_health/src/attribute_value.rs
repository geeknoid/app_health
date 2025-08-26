use crate::attribute_string::AttributeString;
use core::fmt::Display;

/// The value portion of a signal attribute.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeValue {
    /// A signed integer value.
    Int(i64),

    /// A floating-point value.
    Double(f64),

    /// A string value.
    String(AttributeString),

    /// A boolean value.
    Boolean(bool),
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        Self::String(s.into())
    }
}

impl From<Box<str>> for AttributeValue {
    fn from(s: Box<str>) -> Self {
        Self::String(s.into())
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        Self::Double(f)
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Int(i) => Display::fmt(&i, f),
            Self::Double(d) => Display::fmt(&d, f),
            Self::String(s) => Display::fmt(&s, f),
            Self::Boolean(b) => Display::fmt(&b, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let value = AttributeValue::from("test".to_string());
        match value {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_from_box_str() {
        let boxed_str: Box<str> = "test".into();
        let value = AttributeValue::from(boxed_str);
        match value {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_from_str_ref() {
        let value = AttributeValue::from("test");
        match value {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_from_bool() {
        let value_true = AttributeValue::from(true);
        let value_false = AttributeValue::from(false);

        match value_true {
            AttributeValue::Boolean(b) => assert!(b),
            _ => panic!("Expected Boolean variant"),
        }

        match value_false {
            AttributeValue::Boolean(b) => assert!(!b),
            _ => panic!("Expected Boolean variant"),
        }
    }

    #[test]
    fn test_from_i64() {
        let value = AttributeValue::from(42i64);
        match value {
            AttributeValue::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int variant"),
        }
    }

    #[test]
    fn test_from_f64() {
        let value = AttributeValue::from(2.5f64);
        match value {
            AttributeValue::Double(d) => assert!((d - 2.5).abs() < f64::EPSILON),
            _ => panic!("Expected Double variant"),
        }
    }

    #[test]
    fn test_display_int() {
        let value = AttributeValue::Int(42);
        assert_eq!(format!("{value}"), "42");
    }

    #[test]
    fn test_display_double() {
        let value = AttributeValue::Double(2.5);
        assert_eq!(format!("{value}"), "2.5");
    }

    #[test]
    fn test_display_string() {
        let value = AttributeValue::String(AttributeString::new("test"));
        assert_eq!(format!("{value}"), "test");
    }

    #[test]
    fn test_display_boolean() {
        let value_true = AttributeValue::Boolean(true);
        let value_false = AttributeValue::Boolean(false);

        assert_eq!(format!("{value_true}"), "true");
        assert_eq!(format!("{value_false}"), "false");
    }

    #[test]
    fn test_clone() {
        let original = AttributeValue::String(AttributeString::new("test"));
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_debug() {
        let value = AttributeValue::Int(42);
        let debug_str = format!("{value:?}");
        assert!(debug_str.contains("Int"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_partial_eq() {
        let value1 = AttributeValue::Int(42);
        let value2 = AttributeValue::Int(42);
        let value3 = AttributeValue::Int(43);

        assert_eq!(value1, value2);
        assert_ne!(value1, value3);
    }

    #[test]
    fn test_different_types_not_equal() {
        let int_val = AttributeValue::Int(42);
        let double_val = AttributeValue::Double(42.0);
        let string_val = AttributeValue::String(AttributeString::new("42"));
        let bool_val = AttributeValue::Boolean(true);

        assert_ne!(int_val, double_val);
        assert_ne!(int_val, string_val);
        assert_ne!(int_val, bool_val);
        assert_ne!(double_val, string_val);
        assert_ne!(double_val, bool_val);
        assert_ne!(string_val, bool_val);
    }

    #[test]
    fn test_negative_values() {
        let negative_int = AttributeValue::Int(-42);
        let negative_double = AttributeValue::Double(-2.5);

        assert_eq!(format!("{negative_int}"), "-42");
        assert_eq!(format!("{negative_double}"), "-2.5");
    }

    #[test]
    fn test_zero_values() {
        let zero_int = AttributeValue::Int(0);
        let zero_double = AttributeValue::Double(0.0);

        assert_eq!(format!("{zero_int}"), "0");
        assert_eq!(format!("{zero_double}"), "0");
    }

    #[test]
    fn test_empty_string() {
        let empty_value = AttributeValue::from("");
        match &empty_value {
            AttributeValue::String(s) => assert_eq!(s.as_str(), ""),
            _ => panic!("Expected String variant"),
        }
        assert_eq!(format!("{empty_value}"), "");
    }

    #[test]
    fn test_large_values() {
        let large_int = AttributeValue::Int(i64::MAX);
        let large_double = AttributeValue::Double(f64::MAX);

        assert_eq!(format!("{large_int}"), i64::MAX.to_string());
        assert_eq!(format!("{large_double}"), f64::MAX.to_string());
    }

    #[test]
    fn test_special_float_values() {
        let infinity = AttributeValue::Double(f64::INFINITY);
        let neg_infinity = AttributeValue::Double(f64::NEG_INFINITY);
        let nan = AttributeValue::Double(f64::NAN);

        assert_eq!(format!("{infinity}"), "inf");
        assert_eq!(format!("{neg_infinity}"), "-inf");
        assert_eq!(format!("{nan}"), "NaN");
    }

    #[test]
    fn test_string_variants() {
        // Test static string
        let static_str = AttributeValue::String(AttributeString::new("static"));
        assert_eq!(format!("{static_str}"), "static");

        // Test boxed string
        let boxed_str = AttributeValue::from("boxed".to_string());
        assert_eq!(format!("{boxed_str}"), "boxed");
    }

    #[test]
    fn test_edge_case_numbers() {
        let min_int = AttributeValue::Int(i64::MIN);
        let max_int = AttributeValue::Int(i64::MAX);
        let min_float = AttributeValue::Double(f64::MIN);
        let max_float = AttributeValue::Double(f64::MAX);

        assert_eq!(format!("{min_int}"), i64::MIN.to_string());
        assert_eq!(format!("{max_int}"), i64::MAX.to_string());
        assert_eq!(format!("{min_float}"), f64::MIN.to_string());
        assert_eq!(format!("{max_float}"), f64::MAX.to_string());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_serialization() {
        let values = vec![
            AttributeValue::Int(42),
            AttributeValue::Double(2.5),
            AttributeValue::String(AttributeString::new("test")),
            AttributeValue::Boolean(true),
        ];

        for value in values {
            let serialized = serde_json::to_string(&value).expect("Failed to serialize");
            let deserialized: AttributeValue = serde_json::from_str(&serialized).expect("Failed to deserialize");
            assert_eq!(value, deserialized);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_edge_cases() {
        let edge_cases = vec![
            AttributeValue::Int(i64::MIN),
            AttributeValue::Int(i64::MAX),
            AttributeValue::Double(f64::INFINITY),
            AttributeValue::Double(f64::NEG_INFINITY),
            AttributeValue::String(AttributeString::new("")),
            AttributeValue::Boolean(false),
        ];

        for value in edge_cases {
            let serialized = serde_json::to_string(&value).expect("Failed to serialize");
            let deserialized: AttributeValue = serde_json::from_str(&serialized).expect("Failed to deserialize");

            match (&value, &deserialized) {
                (AttributeValue::Double(a), AttributeValue::Double(b)) if a.is_infinite() && b.is_infinite() => {
                    assert_eq!(a.is_sign_positive(), b.is_sign_positive());
                }
                _ => assert_eq!(value, deserialized),
            }
        }
    }
}
