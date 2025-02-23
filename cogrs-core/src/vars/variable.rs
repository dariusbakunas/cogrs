use anyhow::bail;
use anyhow::Result;
use indexmap::IndexMap;
use serde::Serialize;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub type Sequence = Vec<Variable>;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number::Int(i) => Some(*i),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Mapping {
    pub(crate) map: IndexMap<String, Variable>,
}

impl From<IndexMap<String, Variable>> for Mapping {
    fn from(map: IndexMap<String, Variable>) -> Self {
        Mapping { map }
    }
}

impl From<IndexMap<String, Vec<String>>> for Mapping {
    fn from(map: IndexMap<String, Vec<String>>) -> Self {
        let mut result = Mapping::new();
        for (key, value) in map {
            let sequence: Vec<Variable> = value
                .iter()
                .map(|s| Variable::String(s.to_string()))
                .collect();
            result.insert(key, Variable::Sequence(sequence));
        }
        result
    }
}

impl Mapping {
    pub fn iter(&self) -> MappingIter<'_> {
        MappingIter {
            inner: self.map.iter(),
        }
    }

    pub fn new() -> Self {
        Mapping {
            map: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Variable) {
        self.map.insert(key, value);
    }
}

pub struct MappingIter<'a> {
    inner: indexmap::map::Iter<'a, String, Variable>,
}

impl<'a> Iterator for MappingIter<'a> {
    type Item = (&'a String, &'a Variable);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a String, &'a Variable);
    type IntoIter = MappingIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum Variable {
    Null,
    Bool(bool),
    Number(Number),
    Sequence(Sequence),
    Mapping(Mapping),
    String(String),
    Path(PathBuf),
}

pub fn get_vars_from_path(path: &Path) -> Result<IndexMap<String, Variable>> {
    todo!()
}

pub fn get_vars_from_inventory_sources(
    sources: Option<&[String]>,
) -> Result<IndexMap<String, Variable>> {
    let mut vars = IndexMap::new();

    if let Some(sources) = sources {
        for source in sources {
            // TODO: revisit this logic, what if its real path and host separated by comma?
            if source.contains(",") {
                continue;
            }

            let mut path = Path::new(source);

            if !path.is_dir() {
                path = path.parent().ok_or(anyhow::format_err!(
                    "Invalid inventory source path: {}",
                    source
                ))?;
            }

            //vars = combine_variables(&vars, &get_vars_from_path(path)?);
        }
    }

    Ok(vars)
}

pub enum ConflictResolution {
    Replace,
    Merge,
}

pub fn combine_variables(
    a: &IndexMap<String, Variable>,
    b: &IndexMap<String, Variable>,
    strategy: &ConflictResolution,
) -> IndexMap<String, Variable> {
    let mut result = a.clone();

    for (key, value) in b {
        match strategy {
            ConflictResolution::Replace => {
                result.insert(key.clone(), value.clone());
            }
            ConflictResolution::Merge => {
                if let (Some(Variable::Mapping(a_map)), Variable::Mapping(b_map)) =
                    (result.get(key), value)
                {
                    let merged = combine_variables(&a_map.map, &b_map.map, strategy);
                    result.insert(key.clone(), Variable::Mapping(Mapping { map: merged }));
                } else {
                    result.insert(key.clone(), value.clone());
                }
            }
        }
    }

    result
}

impl TryFrom<&serde_yaml::Value> for Variable {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(Variable::Null),
            Value::Bool(b) => Ok(Variable::Bool(b.to_owned())),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Variable::Number(Number::Int(i)))
                } else if let Some(f) = n.as_f64() {
                    Ok(Variable::Number(Number::Float(f)))
                } else {
                    bail!("Invalid number format: {:?}", n);
                }
            }
            Value::String(s) => Ok(Variable::String(s.to_string())),
            Value::Sequence(s) => {
                let sequence: Result<Sequence, _> = s.iter().map(Variable::try_from).collect();
                sequence.map(Variable::Sequence)
            }
            Value::Mapping(m) => {
                let map: Result<IndexMap<String, Variable>, _> = m
                    .into_iter()
                    .map(|(k, v)| {
                        if let Value::String(key) = k {
                            Variable::try_from(v).map(|var| (key.to_string(), var))
                        } else {
                            bail!("Mapping key is not a string: {:?}", k);
                        }
                    })
                    .collect();
                map.map(|map| Variable::Mapping(Mapping { map }))
            }
            Value::Tagged(t) => bail!("Unsupported type: {:?}", t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::value::{Mapping as YamlMapping, Tag, TaggedValue};
    use serde_yaml::Value;

    #[test]
    fn test_combine_no_conflicts() {
        let mut map_a = IndexMap::new();
        map_a.insert("key1".to_string(), Variable::Bool(true));
        let mut map_b = IndexMap::new();
        map_b.insert("key2".to_string(), Variable::String("hello".to_string()));

        let result = combine_variables(&map_a, &map_b, &ConflictResolution::Replace);

        assert_eq!(result.len(), 2);
        assert_eq!(result["key1"], Variable::Bool(true));
        assert_eq!(result["key2"], Variable::String("hello".to_string()));
    }

    #[test]
    fn test_combine_with_conflicts() {
        let mut map_a = IndexMap::new();
        map_a.insert("key1".to_string(), Variable::Bool(true));
        map_a.insert("key2".to_string(), Variable::Number(Number::Int(10)));

        let mut map_b = IndexMap::new();
        map_b.insert("key2".to_string(), Variable::Number(Number::Float(3.14))); // overwrites key2
        map_b.insert("key3".to_string(), Variable::String("world".to_string()));

        let result = combine_variables(&map_a, &map_b, &ConflictResolution::Replace);

        assert_eq!(result.len(), 3);
        assert_eq!(result["key1"], Variable::Bool(true));
        assert_eq!(result["key2"], Variable::Number(Number::Float(3.14))); // Overwritten value
        assert_eq!(result["key3"], Variable::String("world".to_string()));
    }

    #[test]
    fn test_combine_empty_maps() {
        let map_a: IndexMap<String, Variable> = IndexMap::new();
        let map_b: IndexMap<String, Variable> = IndexMap::new();
        let result = combine_variables(&map_a, &map_b, &ConflictResolution::Replace);
        assert!(result.is_empty());
    }

    #[test]
    fn test_combine_with_empty_b() {
        let mut map_a = IndexMap::new();
        map_a.insert("key1".to_string(), Variable::Bool(true));

        let map_b: IndexMap<String, Variable> = IndexMap::new();

        let result = combine_variables(&map_a.clone(), &map_b, &ConflictResolution::Replace);

        assert_eq!(result.len(), 1);
        assert_eq!(result["key1"], map_a["key1"]);
    }

    #[test]
    fn test_combine_with_empty_a() {
        let map_a: IndexMap<String, Variable> = IndexMap::new();

        let mut map_b = IndexMap::new();
        map_b.insert("key1".to_string(), Variable::Number(Number::Int(5)));

        let result = combine_variables(&map_a, &map_b.clone(), &ConflictResolution::Replace);

        assert_eq!(result.len(), 1);
        assert_eq!(result["key1"], map_b["key1"]);
    }

    #[test]
    fn test_null_conversion() {
        let value = Value::Null;
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Variable::Null);
    }

    #[test]
    fn test_bool_conversion() {
        let value = Value::Bool(true);
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Variable::Bool(true));
    }

    #[test]
    fn test_number_conversion_int() {
        let value = Value::Number(serde_yaml::Number::from(42));
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Variable::Number(Number::Int(42)));
    }

    #[test]
    fn test_number_conversion_float() {
        let value = Value::Number(serde_yaml::Number::from(42.5));
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Variable::Number(Number::Float(42.5)));
    }

    #[test]
    fn test_string_conversion() {
        let value = Value::String("Hello, world!".to_string());
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Variable::String("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_sequence_conversion() {
        let value = Value::Sequence(vec![
            Value::Bool(true),
            Value::Number(serde_yaml::Number::from(42)),
            Value::String("test".to_string()),
        ]);
        let result = Variable::try_from(&value);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Variable::Sequence(vec![
                Variable::Bool(true),
                Variable::Number(Number::Int(42)),
                Variable::String("test".to_string()),
            ])
        );
    }

    #[test]
    fn test_sequence_with_invalid_item() {
        let value = Value::Sequence(vec![Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!invalid_tag"),
            value: Value::Null,
        }))]);
        let result = Variable::try_from(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_conversion() {
        let mut map = YamlMapping::new();
        map.insert(Value::String("key1".to_string()), Value::Bool(true));
        map.insert(
            Value::String("key2".to_string()),
            Value::Number(serde_yaml::Number::from(42)),
        );

        let value = Value::Mapping(map);
        let result = Variable::try_from(&value);
        assert!(result.is_ok());

        let mut expected_map = IndexMap::new();
        expected_map.insert("key1".to_string(), Variable::Bool(true));
        expected_map.insert("key2".to_string(), Variable::Number(Number::Int(42)));

        assert_eq!(
            result.unwrap(),
            Variable::Mapping(Mapping { map: expected_map })
        );
    }

    #[test]
    fn test_mapping_with_invalid_key() {
        // Keys in a YAML mapping must be strings. Here, we intentionally use a non-string key.
        let mut map = YamlMapping::new();
        map.insert(Value::Null, Value::Bool(true)); // Invalid key type.

        let value = Value::Mapping(map);
        let result = Variable::try_from(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_tagged_value() {
        let value = Value::Tagged(Box::new(TaggedValue {
            tag: Tag::new("!invalid_tag"),
            value: Value::Null,
        }));
        let result = Variable::try_from(&value);
        assert!(result.is_err());
    }
}
