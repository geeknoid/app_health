use crate::{AttributeString, AttributeValue};
use core::cmp::Ordering;
use core::fmt::Display;
use core::hash::{Hash, Hasher};

/// A name/value pair used to provide context about a signal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    name: AttributeString,
    value: AttributeValue,
}

impl Attribute {
    /// Create a new attribute with the given name and value.
    #[must_use]
    pub const fn new(name: AttributeString, value: AttributeValue) -> Self {
        Self { name, value }
    }

    /// Get the name of the attribute.
    #[must_use]
    pub const fn name(&self) -> &AttributeString {
        &self.name
    }

    /// Get the value of the attribute.
    #[must_use]
    pub const fn value(&self) -> &AttributeValue {
        &self.value
    }
}

impl<T, U> From<(T, U)> for Attribute
where
    T: Into<AttributeString>,
    U: Into<AttributeValue>,
{
    fn from((k, v): (T, U)) -> Self {
        Self::new(k.into(), v.into())
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({:?}: {:?})", self.name.as_str(), self.value)
    }
}

impl PartialOrd for Attribute {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_new() {
        let attr = Attribute::new(
            AttributeString::new("test_key"),
            AttributeValue::String(AttributeString::new("test_value")),
        );
        assert_eq!(attr.name().as_str(), "test_key");
        match attr.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test_value"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_name() {
        let attr = Attribute::new(AttributeString::new("key"), AttributeValue::Int(42));
        assert_eq!(attr.name().as_str(), "key");
    }

    #[test]
    fn test_value() {
        let attr = Attribute::new(AttributeString::new("key"), AttributeValue::Int(42));
        match attr.value() {
            AttributeValue::Int(42) => (),
            _ => panic!("Expected Int(42)"),
        }
    }

    #[test]
    fn test_from_tuple_string_int() {
        let attr = Attribute::from(("key", 42i64));
        assert_eq!(attr.name().as_str(), "key");
        match attr.value() {
            AttributeValue::Int(42) => (),
            _ => panic!("Expected Int(42)"),
        }
    }

    #[test]
    fn test_from_tuple_string_double() {
        let attr = Attribute::from(("key", 5.24f64));
        assert_eq!(attr.name().as_str(), "key");
        match attr.value() {
            AttributeValue::Double(f) if (*f - 5.24).abs() < f64::EPSILON => (),
            _ => panic!("Expected Double(5.24)"),
        }
    }

    #[test]
    fn test_from_tuple_string_bool() {
        let attr = Attribute::from(("key", true));
        assert_eq!(attr.name().as_str(), "key");
        match attr.value() {
            AttributeValue::Boolean(true) => (),
            _ => panic!("Expected Boolean(true)"),
        }
    }

    #[test]
    fn test_from_tuple_string_string() {
        let attr = Attribute::from(("key", "value"));
        assert_eq!(attr.name().as_str(), "key");
        match attr.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "value"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_from_tuple_owned_string() {
        let attr = Attribute::from(("key".to_string(), "value".to_string()));
        assert_eq!(attr.name().as_str(), "key");
        match attr.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "value"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_display() {
        let attr = Attribute::from(("test_key", "test_value"));
        let display_str = format!("{attr}");
        assert!(display_str.contains("test_key"));
        assert!(display_str.contains("test_value"));
    }

    #[test]
    fn test_display_different_types() {
        let attr_int = Attribute::from(("key", 42i64));
        let attr_bool = Attribute::from(("key", true));
        let attr_double = Attribute::from(("key", 5.24f64));

        let display_int = format!("{attr_int}");
        let display_bool = format!("{attr_bool}");
        let display_double = format!("{attr_double}");

        assert!(display_int.contains("42"));
        assert!(display_bool.contains("true"));
        assert!(display_double.contains("5.24"));
    }

    #[test]
    fn test_partial_eq() {
        let attr1 = Attribute::from(("key", "value"));
        let attr2 = Attribute::from(("key", "value"));
        let attr3 = Attribute::from(("key", "different"));
        let attr4 = Attribute::from(("different", "value"));

        assert_eq!(attr1, attr2);
        assert_ne!(attr1, attr3);
        assert_ne!(attr1, attr4);
    }

    #[test]
    fn test_partial_ord() {
        let attr_a = Attribute::from(("a", "value"));
        let attr_b = Attribute::from(("b", "value"));
        let attr_c = Attribute::from(("c", "value"));

        assert!(attr_a < attr_b);
        assert!(attr_b < attr_c);
        assert!(attr_a < attr_c);

        // Test with same keys but different values - should be equal since ordering is by name only
        let attr_same1 = Attribute::from(("same", "value1"));
        let attr_same2 = Attribute::from(("same", "value2"));
        assert_eq!(attr_same1.partial_cmp(&attr_same2), Some(Ordering::Equal));
    }

    #[test]
    fn test_hash() {
        let attr1 = Attribute::from(("key", "value1"));
        let attr2 = Attribute::from(("key", "value2"));
        let attr3 = Attribute::from(("different", "value"));

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        attr1.hash(&mut hasher1);
        attr2.hash(&mut hasher2);
        attr3.hash(&mut hasher3);

        // Same key should produce same hash regardless of value
        assert_eq!(hasher1.finish(), hasher2.finish());
        // Different key should produce different hash
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_debug() {
        let attr = Attribute::from(("debug_key", 123i64));
        let debug_str = format!("{attr:?}");
        assert!(debug_str.contains("debug_key"));
        assert!(debug_str.contains("123"));
    }

    #[test]
    fn test_const_new() {
        // Test that new() is const
        const ATTR: Attribute = Attribute::new(AttributeString::new("const_key"), AttributeValue::Int(42));
        assert_eq!(ATTR.name().as_str(), "const_key");
    }

    #[test]
    fn test_clone() {
        let attr1 = Attribute::from(("key", "value"));
        let attr2 = attr1.clone();
        assert_eq!(attr1, attr2);
        assert_eq!(attr1.name().as_str(), attr2.name().as_str());
    }

    #[test]
    fn test_different_value_types() {
        let attr_int = Attribute::from(("int_key", 42i64));
        let attr_float = Attribute::from(("float_key", 5.24f64));
        let attr_bool = Attribute::from(("bool_key", true));
        let attr_string = Attribute::from(("string_key", "test"));

        // Verify each type is stored correctly
        match attr_int.value() {
            AttributeValue::Int(42) => (),
            _ => panic!("Expected Int(42)"),
        }

        match attr_float.value() {
            AttributeValue::Double(f) if (*f - 5.24).abs() < f64::EPSILON => (),
            _ => panic!("Expected Double(5.24)"),
        }

        match attr_bool.value() {
            AttributeValue::Boolean(true) => (),
            _ => panic!("Expected Boolean(true)"),
        }

        match attr_string.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test with empty string
        let attr_empty = Attribute::from(("", ""));
        assert_eq!(attr_empty.name().as_str(), "");
        match attr_empty.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), ""),
            _ => panic!("Expected String variant"),
        }

        // Test with zero values
        let attr_zero_int = Attribute::from(("zero", 0i64));
        let attr_zero_float = Attribute::from(("zero", 0.0f64));
        let attr_false = Attribute::from(("false", false));

        match attr_zero_int.value() {
            AttributeValue::Int(0) => (),
            _ => panic!("Expected Int(0)"),
        }

        match attr_zero_float.value() {
            AttributeValue::Double(f) if f.abs() < f64::EPSILON => (),
            _ => panic!("Expected Double(0.0)"),
        }

        match attr_false.value() {
            AttributeValue::Boolean(false) => (),
            _ => panic!("Expected Boolean(false)"),
        }
    }

    #[test]
    fn test_ordering_consistency() {
        let mut attrs = [
            Attribute::from(("z", 1)),
            Attribute::from(("a", 2)),
            Attribute::from(("m", 3)),
            Attribute::from(("b", 4)),
        ];

        attrs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(attrs[0].name().as_str(), "a");
        assert_eq!(attrs[1].name().as_str(), "b");
        assert_eq!(attrs[2].name().as_str(), "m");
        assert_eq!(attrs[3].name().as_str(), "z");
    }

    #[test]
    fn test_negative_and_large_values() {
        let attr_negative = Attribute::from(("negative", -42i64));
        let attr_large = Attribute::from(("large", i64::MAX));

        match attr_negative.value() {
            AttributeValue::Int(-42) => (),
            _ => panic!("Expected Int(-42)"),
        }

        match attr_large.value() {
            AttributeValue::Int(v) if *v == i64::MAX => (),
            _ => panic!("Expected Int(i64::MAX)"),
        }
    }

    #[test]
    fn test_special_float_values() {
        let attr_nan = Attribute::from(("nan", f64::NAN));
        let attr_inf = Attribute::from(("inf", f64::INFINITY));
        let attr_neg_inf = Attribute::from(("neg_inf", f64::NEG_INFINITY));

        match attr_nan.value() {
            AttributeValue::Double(f) if f.is_nan() => (),
            _ => panic!("Expected NaN"),
        }

        match attr_inf.value() {
            AttributeValue::Double(f) if f.is_infinite() && f.is_sign_positive() => (),
            _ => panic!("Expected positive infinity"),
        }

        match attr_neg_inf.value() {
            AttributeValue::Double(f) if f.is_infinite() && f.is_sign_negative() => (),
            _ => panic!("Expected negative infinity"),
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_serialization() {
        let attr = Attribute::from(("test_key", "test_value"));
        let serialized = serde_json::to_string(&attr).expect("Failed to serialize");
        assert!(serialized.contains("test_key"));
        assert!(serialized.contains("test_value"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_deserialization() {
        let json = r#"{"name":"test_key","value":{"String":"test_value"}}"#;
        let attr: Attribute = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(attr.name().as_str(), "test_key");
        match attr.value() {
            AttributeValue::String(s) => assert_eq!(s.as_str(), "test_value"),
            _ => panic!("Expected String variant"),
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let original = Attribute::from(("roundtrip_key", 42i64));
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: Attribute = serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(original, deserialized);
    }
}
