use anyhow::bail;
use hashbrown::HashMap;
use serde_yaml::Value;

pub type Sequence = Vec<Variable>;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Mapping {
    map: HashMap<String, Variable>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    Null,
    Bool(bool),
    Number(Number),
    Sequence(Sequence),
    Mapping(Mapping),
    String(String),
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
                let sequence: Result<Sequence, _> =
                    s.into_iter().map(|v| Variable::try_from(v)).collect();
                sequence.map(Variable::Sequence)
            }
            Value::Mapping(m) => {
                let map: Result<HashMap<String, Variable>, _> = m
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
    use hashbrown::HashMap;
    use serde_yaml::value::{Mapping as YamlMapping, Tag, TaggedValue};
    use serde_yaml::Value;

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

        let mut expected_map = HashMap::new();
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
