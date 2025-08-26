use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};
use std::fmt;

/// A string used in a signal's attributes.
///
/// # Example
///
/// ```
/// use app_health::AttributeString;
///
/// let s1 = AttributeString::new("static string");
/// let s2 = AttributeString::from("string".to_string());
///
/// assert_eq!(s1.as_str(), "static string");
/// assert_eq!(s2.as_str(), "string");
/// ```
#[derive(Clone, Debug, Eq)]
pub enum AttributeString {
    /// A string with static lifetime.
    Static(&'static str),

    /// A boxed string.
    Boxed(Box<str>),
}

impl AttributeString {
    /// Create a new `AttributeString` from a static string.
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self::Static(value)
    }

    /// Get the attribute string as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Boxed(s) => s.as_ref(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AttributeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AttributeString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::Boxed(Box::from(s)))
    }
}

impl PartialOrd for AttributeString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AttributeString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq for AttributeString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl Hash for AttributeString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl From<String> for AttributeString {
    fn from(s: String) -> Self {
        Self::Boxed(Box::from(s))
    }
}

impl From<Box<str>> for AttributeString {
    fn from(s: Box<str>) -> Self {
        Self::Boxed(s)
    }
}

impl From<&str> for AttributeString {
    fn from(s: &str) -> Self {
        Self::Boxed(Box::from(s))
    }
}

impl Display for AttributeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Static(s) => Display::fmt(&s, f),
            Self::Boxed(s) => Display::fmt(&s, f),
        }
    }
}

impl Borrow<str> for AttributeString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for AttributeString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new_static() {
        let attr = AttributeString::new("test");
        assert_eq!(attr.as_str(), "test");
        match attr {
            AttributeString::Static(_) => (),
            AttributeString::Boxed(_) => panic!("Expected Static variant"),
        }
    }

    #[test]
    fn test_as_str() {
        let static_attr = AttributeString::new("static");
        let boxed_attr = AttributeString::from("boxed");

        assert_eq!(static_attr.as_str(), "static");
        assert_eq!(boxed_attr.as_str(), "boxed");
    }

    #[test]
    fn test_from_string() {
        let s = String::from("test_string");
        let attr = AttributeString::from(s);
        assert_eq!(attr.as_str(), "test_string");
        match attr {
            AttributeString::Boxed(_) => (),
            AttributeString::Static(_) => panic!("Expected Boxed variant"),
        }
    }

    #[test]
    fn test_from_box_str() {
        let boxed_str: Box<str> = Box::from("test_box");
        let attr = AttributeString::from(boxed_str);
        assert_eq!(attr.as_str(), "test_box");
        match attr {
            AttributeString::Boxed(_) => (),
            AttributeString::Static(_) => panic!("Expected Boxed variant"),
        }
    }

    #[test]
    fn test_from_str_ref() {
        let attr = AttributeString::from("test_ref");
        assert_eq!(attr.as_str(), "test_ref");
        match attr {
            AttributeString::Boxed(_) => (),
            AttributeString::Static(_) => panic!("Expected Boxed variant"),
        }
    }

    #[test]
    fn test_equality() {
        let static1 = AttributeString::new("same");
        let static2 = AttributeString::new("same");
        let boxed1 = AttributeString::from("same");
        let boxed2 = AttributeString::from("same");
        let different = AttributeString::from("different");

        assert_eq!(static1, static2);
        assert_eq!(static1, boxed1);
        assert_eq!(boxed1, boxed2);
        assert_ne!(static1, different);
        assert_ne!(boxed1, different);
    }

    #[test]
    fn test_ordering() {
        let attr_a = AttributeString::from("a");
        let attr_b = AttributeString::from("b");
        let attr_a_static = AttributeString::new("a");

        assert!(attr_a < attr_b);
        assert!(attr_b > attr_a);
        assert_eq!(attr_a.cmp(&attr_a_static), Ordering::Equal);
        assert_eq!(attr_a.partial_cmp(&attr_b), Some(Ordering::Less));
    }

    #[test]
    fn test_hash() {
        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let static_attr = AttributeString::new("test");
        let boxed_attr = AttributeString::from("test");

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        static_attr.hash(&mut hasher1);
        boxed_attr.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_map_usage() {
        let mut map = HashMap::new();
        let key1 = AttributeString::new("key");
        let key2 = AttributeString::from("key");

        let _ = map.insert(key1, "value1");
        assert_eq!(map.get(&key2), Some(&"value1"));
    }

    #[test]
    fn test_display() {
        let static_attr = AttributeString::new("static_display");
        let boxed_attr = AttributeString::from("boxed_display");

        assert_eq!(format!("{static_attr}"), "static_display");
        assert_eq!(format!("{boxed_attr}"), "boxed_display");
    }

    #[test]
    fn test_debug() {
        let static_attr = AttributeString::new("debug");
        let boxed_attr = AttributeString::from("debug");

        let static_debug = format!("{static_attr:?}");
        let boxed_debug = format!("{boxed_attr:?}");

        assert!(static_debug.contains("Static"));
        assert!(static_debug.contains("debug"));
        assert!(boxed_debug.contains("Boxed"));
        assert!(boxed_debug.contains("debug"));
    }

    #[test]
    fn test_borrow() {
        let attr = AttributeString::from("borrow_test");
        let borrowed: &str = attr.borrow();
        assert_eq!(borrowed, "borrow_test");
    }

    #[test]
    fn test_as_ref() {
        let attr = AttributeString::from("as_ref_test");
        let as_ref: &str = attr.as_ref();
        assert_eq!(as_ref, "as_ref_test");
    }

    #[test]
    fn test_clone() {
        let original = AttributeString::from("clone_test");
        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.as_str(), cloned.as_str());
    }

    #[test]
    fn test_empty_string() {
        let empty_static = AttributeString::new("");
        let empty_boxed = AttributeString::from("");

        assert_eq!(empty_static.as_str(), "");
        assert_eq!(empty_boxed.as_str(), "");
        assert_eq!(empty_static, empty_boxed);
    }

    #[test]
    fn test_unicode_string() {
        let unicode = AttributeString::from("Hello, 世界! 🦀");
        assert_eq!(unicode.as_str(), "Hello, 世界! 🦀");
        assert_eq!(format!("{unicode}"), "Hello, 世界! 🦀");
    }

    #[test]
    fn test_long_string() {
        let long_str = "a".repeat(1000);
        let attr = AttributeString::from(long_str.as_str());
        assert_eq!(attr.as_str().len(), 1000);
        assert!(attr.as_str().chars().all(|c| c == 'a'));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_serialize() {
        let attr = AttributeString::from("serialize_test");
        let serialized = serde_json::to_string(&attr).unwrap();
        assert_eq!(serialized, "\"serialize_test\"");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_deserialize() {
        let json = "\"deserialize_test\"";
        let attr: AttributeString = serde_json::from_str(json).unwrap();
        assert_eq!(attr.as_str(), "deserialize_test");
        match attr {
            AttributeString::Boxed(_) => (),
            AttributeString::Static(_) => panic!("Deserialized AttributeString should be Boxed variant"),
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let original = AttributeString::from("roundtrip_test");
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: AttributeString = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip_comprehensive() {
        // Test Boxed variant
        let boxed_attr = AttributeString::from("boxed_roundtrip");
        let serialized_boxed = serde_json::to_string(&boxed_attr).unwrap();
        let deserialized_boxed: AttributeString = serde_json::from_str(&serialized_boxed).unwrap();
        assert_eq!(boxed_attr, deserialized_boxed);
        assert_eq!(boxed_attr.as_str(), deserialized_boxed.as_str());

        // Test Static variant
        let static_attr = AttributeString::new("static_roundtrip");
        let serialized_static = serde_json::to_string(&static_attr).unwrap();
        let deserialized_static: AttributeString = serde_json::from_str(&serialized_static).unwrap();
        assert_eq!(static_attr, deserialized_static);
        assert_eq!(static_attr.as_str(), deserialized_static.as_str());

        // Test empty string
        let empty_attr = AttributeString::from("");
        let serialized_empty = serde_json::to_string(&empty_attr).unwrap();
        let deserialized_empty: AttributeString = serde_json::from_str(&serialized_empty).unwrap();
        assert_eq!(empty_attr, deserialized_empty);
        assert_eq!(empty_attr.as_str(), deserialized_empty.as_str());

        // Test unicode string
        let unicode_attr = AttributeString::from("Hello, 世界! 🦀");
        let serialized_unicode = serde_json::to_string(&unicode_attr).unwrap();
        let deserialized_unicode: AttributeString = serde_json::from_str(&serialized_unicode).unwrap();
        assert_eq!(unicode_attr, deserialized_unicode);
        assert_eq!(unicode_attr.as_str(), deserialized_unicode.as_str());

        // Test string with special characters
        let special_attr = AttributeString::from("\"newline\n\\tab\t\\quote\"");
        let serialized_special = serde_json::to_string(&special_attr).unwrap();
        let deserialized_special: AttributeString = serde_json::from_str(&serialized_special).unwrap();
        assert_eq!(special_attr, deserialized_special);
        assert_eq!(special_attr.as_str(), deserialized_special.as_str());

        // Verify that all serialized forms are plain strings (no enum structure)
        assert_eq!(serialized_boxed, "\"boxed_roundtrip\"");
        assert_eq!(serialized_static, "\"static_roundtrip\"");
        assert_eq!(serialized_empty, "\"\"");
        assert!(serialized_unicode.contains("世界"));
        assert!(serialized_special.contains("\\n"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip_different_formats() {
        let attr = AttributeString::from("format_test");

        // Test with different serde formats
        // JSON
        let json_serialized = serde_json::to_string(&attr).unwrap();
        let json_deserialized: AttributeString = serde_json::from_str(&json_serialized).unwrap();
        assert_eq!(attr, json_deserialized);

        // Verify the JSON is a plain string
        assert_eq!(json_serialized, "\"format_test\"");
    }
}
