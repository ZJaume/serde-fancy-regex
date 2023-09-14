
//! # Serde FancyRegex
//!
//! [Documentation](https://docs.rs/serde_fancy_regex) |
//! [Github](https://github.com/ZJaume/serde-fancy-regex) |
//! [Crate](https://crates.io/crates/serde_regex)
//!
//! A (de)serializer for `fancy_regex::Regex` forked from `serde_regex`.
//! Note that this fork does not implement bytes Regex
//! nor RegexSet, as `fancy_regex` does not implement them.
//!
//! # Example
//!
//! ```rust
//!
//! use fancy_regex::Regex;
//! use serde::{Deserialize, Serialize};
//! use serde_derive::{Serialize, Deserialize};
//!
//!
//! #[derive(Serialize, Deserialize)]
//! struct Timestamps {
//!     #[serde(with = "serde_fancy_regex")]
//!     pattern: Regex,
//! }
//!
//! #
//! # fn main() {}
//! ```
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use fancy_regex;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    ops::{Deref, DerefMut}
};

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{Error, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq}
};

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `fancy_regex::Regex`
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(pub T);

struct FancyRegexVecVisitor;

impl<'a> Visitor<'a> for FancyRegexVecVisitor {
    type Value = Serde<Vec<fancy_regex::Regex>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid sequence")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'a>,
    {
        let mut vec = match seq.size_hint() {
            Some(size) => Vec::with_capacity(size),
            None => Vec::new(),
        };
        while let Some(Serde(el)) = seq.next_element()? {
            vec.push(el);
        }
        return Ok(Serde(vec));
    }
}


struct FancyRegexHashMapVisitor<K, S>(PhantomData<(K, S)>);

impl<K, S> Default for FancyRegexHashMapVisitor<K, S> {
    fn default() -> Self {
        Self(Default::default())
    }
}


impl<'a, K, S> Visitor<'a> for FancyRegexHashMapVisitor<K, S>
where
    K: Hash + Eq + Deserialize<'a>,
    S: BuildHasher + Default,
{
    type Value = Serde<HashMap<K, fancy_regex::Regex, S>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid map")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'a>
    {
        let mut hashmap = match map.size_hint() {
            Some(size) => HashMap::with_capacity_and_hasher(size, S::default()),
            None => HashMap::with_hasher(S::default()),
        };
        while let Some((key, Serde(value))) = map.next_entry()? {
            hashmap.insert(key, value);
        }
        return Ok(Serde(hashmap));
    }
}

impl<'de> Deserialize<'de> for Serde<Option<fancy_regex::Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<fancy_regex::Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<fancy_regex::Regex>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<fancy_regex::Regex> {
    fn deserialize<D>(d: D) -> Result<Serde<fancy_regex::Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <Cow<str>>::deserialize(d)?;

        match s.parse() {
            Ok(regex) => Ok(Serde(regex)),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Vec<fancy_regex::Regex>> {
    fn deserialize<D>(d: D) -> Result<Serde<Vec<fancy_regex::Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_seq(FancyRegexVecVisitor)
    }
}

impl<'de, K, S> Deserialize<'de> for Serde<HashMap<K, fancy_regex::Regex, S>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_map(FancyRegexHashMapVisitor::default())
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Vec<fancy_regex::Regex>>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Vec<fancy_regex::Regex>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<Vec<fancy_regex::Regex>>>::deserialize(d)? {
            Some(Serde(regex)) => Ok(Serde(Some(regex))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de, K, S> Deserialize<'de> for Serde<Option<HashMap<K, fancy_regex::Regex, S>>>
where
    K: Hash + Eq + Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
         match Option::<Serde<HashMap<K, fancy_regex::Regex, S>>>::deserialize(d)? {
            Some(Serde(map)) => Ok(Serde(Some(map))),
            None => Ok(Serde(None)),
        }
    }
}

/// Deserialize function, see crate docs to see how to use it
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    Serde<T>: Deserialize<'de>,
{
    Serde::deserialize(deserializer).map(|x| x.0)
}

/// Serialize function, see crate docs to see how to use it
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    for<'a> Serde<&'a T>: Serialize,
{
    Serde(value).serialize(serializer)
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Serde<T> {
    /// Consumes the `Serde`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Serde<T> {
    fn from(val: T) -> Serde<T> {
        Serde(val)
    }
}

impl<'a> Serialize for Serde<&'a fancy_regex::Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl Serialize for Serde<fancy_regex::Regex> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Option<fancy_regex::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            &Some(ref value) => serializer.serialize_some(&Serde(value)),
            &None => serializer.serialize_none(),
        }
    }
}

impl Serialize for Serde<Option<fancy_regex::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl Serialize for Serde<Vec<fancy_regex::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<K, S> Serialize for Serde<HashMap<K, fancy_regex::Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<'a> Serialize for Serde<&'a Vec<fancy_regex::Regex>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for element in self.0 {
            seq.serialize_element(&Serde(element))?;
        }
        seq.end()
    }
}

impl<'a> Serialize for Serde<&'a Option<Vec<fancy_regex::Regex>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            &Some(ref value) => serializer.serialize_some(&Serde(value)),
            &None => serializer.serialize_none(),
        }
    }
}

impl<'a, K, S> Serialize for Serde<&'a HashMap<K, fancy_regex::Regex, S>>
where
    K: Hash + Eq + Serialize,
    S: BuildHasher + Default,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.0.iter() {
            map.serialize_entry(key, &Serde(value))?;
        }
        map.end()
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde_json::{json, from_value, from_str, to_string};
    use fancy_regex;
    use crate::Serde;

    const SAMPLE: &str = r#"[a-z"\]]+\d{1,10}""#;
    const SAMPLE_JSON: &str = r#""[a-z\"\\]]+\\d{1,10}\"""#;

    #[test]
    fn test_vec() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Vec<fancy_regex::Regex>> = from_value(json)?;
        assert!(vec.0[0].as_str() == "a.*b");
        assert!(vec.0[1].as_str() == "c?d");
        assert!(vec.len() == 2);
        Ok(())
    }

    #[test]
    fn test_hashmap() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({"a": "a.*b", "b": "c?d"});
        let map: Serde<HashMap<String, fancy_regex::Regex>> = from_value(json)?;
        assert!(map.0["a"].as_str() == "a.*b");
        assert!(map.0["b"].as_str() == "c?d");
        assert!(map.len() == 2);
        Ok(())
    }

    #[test]
    fn test_simple() {
        let re: Serde<fancy_regex::Regex> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_str(), SAMPLE);
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_option_some() {
        let re: Serde<Option<fancy_regex::Regex>> = from_str(SAMPLE_JSON).unwrap();
        assert_eq!(re.as_ref().map(|regex| regex.as_str()), Some(SAMPLE));
        assert_eq!(to_string(&re).unwrap(), SAMPLE_JSON);
    }

    #[test]
    fn test_option_none() {
        let re: Serde<Option<fancy_regex::Regex>> = from_str("null").unwrap();
        assert!(re.is_none());
        assert_eq!(to_string(&re).unwrap(), "null");
    }

    #[test]
    fn test_option_vec() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!(["a.*b", "c?d"]);
        let vec: Serde<Option<Vec<fancy_regex::Regex>>> = from_value(json)?;
        assert!(vec.is_some());
        let v = vec.0.unwrap();
        assert!(v[0].as_str() == "a.*b");
        assert!(v[1].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
    #[test]
    fn test_option_hashmap() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({"a": "a.*b", "b": "c?d"});
        let map: Serde<Option<HashMap<String, fancy_regex::Regex>>> = from_value(json)?;
        assert!(map.is_some());
        let v = map.0.unwrap();
        assert!(v["a"].as_str() == "a.*b");
        assert!(v["b"].as_str() == "c?d");
        assert!(v.len() == 2);
        Ok(())
    }
 }

