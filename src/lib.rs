#[cfg(feature = "async_graphql")]
use async_graphql::{registry, InputType, InputValueError, InputValueResult, Value};
#[cfg(feature = "serde")]
use serde::{ser::Error as SerError, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "async_graphql")]
use std::borrow::Cow;

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Maybe<T> {
    #[default]
    Void,
    None,
    Some(T),
}

impl<T> Maybe<T> {
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }
}

impl<T> Clone for Maybe<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Void => Self::Void,
            Self::None => Self::None,
            Self::Some(value) => Self::Some(value.clone()),
        }
    }
}

impl<T> Copy for Maybe<T> where T: Copy {}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(option: Option<T>) -> Maybe<T> {
        match option {
            Some(value) => Maybe::Some(value),
            None => Maybe::None,
        }
    }
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(maybe: Maybe<T>) -> Self {
        match maybe {
            Maybe::Some(value) => Some(value),
            _ => None,
        }
    }
}

/*
impl<T> From<Maybe<T>> for Option<Option<T>> {
    fn from(maybe_undefined: Maybe<T>) -> Self {
        match maybe_undefined {
            Maybe::Void => None,
            Maybe::None => Some(None),
            Maybe::Some(value) => Some(Some(value)),
        }
    }
}

impl<T> From<Option<Option<T>>> for Maybe<T> {
    fn from(value: Option<Option<T>>) -> Self {
        match value {
            Some(Some(value)) => Self::Some(value),
            Some(None) => Self::None,
            None => Self::Void,
        }
    }
}
*/

#[cfg(feature = "async_graphql")]
impl<T> InputType for Maybe<T>
where
    T: InputType,
{
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        T::type_name().to_string()
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value {
            None => Ok(Self::Void),
            Some(Value::Null) => Ok(Self::None),
            Some(value) => Ok(Self::Some(
                T::parse(Some(value)).map_err(InputValueError::propagate)?,
            )),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Some(value) => value.to_value(),
            _ => Value::Null,
        }
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        if let Self::Some(value) = self {
            value.as_raw_value()
        } else {
            None
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Maybe<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for Maybe<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let error_message = r#"'Maybe' fields need to be annotated with:
        #[serde(default, skip_serializing_if = "Maybe::is_void")]
        "#;
        match self {
            Maybe::Some(value) => value.serialize(serializer),
            Maybe::None => serializer.serialize_none(),
            // should have been skipped
            Maybe::Void => Err(SerError::custom(error_message)),
        }
    }
}

#[cfg(all(test, feature = "async_graphql", feature = "serde"))]
mod test {
    use super::*;
    use serde_json;

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    struct Dto {
        a: String,
        #[serde(default, skip_serializing_if = "Maybe::is_void")]
        b: Maybe<i32>,
        c: Option<i32>,
    }

    #[test]
    pub fn test_it_deserializes_with_some_maybe_option_field() {
        let expected = Dto {
            a: "Hello!".into(),
            b: Maybe::Some(34),
            c: None,
        };
        let actual: Dto = serde_json::from_str(
            r#"
        {
            "a": "Hello!",
            "b": 34,
            "c": null
        }
        "#,
        )
        .expect("Couldn't deserialize");
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_it_deserializes_with_undef_maybe_option_field() {
        let expected = Dto {
            a: "Hello!".into(),
            b: Maybe::Void,
            c: Some(0),
        };
        let actual: Dto = serde_json::from_str(
            r#"
        {
            "a": "Hello!",
            "c": 0
        }
        "#,
        )
        .expect("Couldn't deserialize");
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_it_deserializes_with_none_maybe_option_field() {
        let expected = Dto {
            a: "Hello!".into(),
            b: Maybe::None,
            c: Some(0),
        };
        let actual: Dto = serde_json::from_str(
            r#"
        {
            "a": "Hello!",
            "b": null,
            "c": 0
        }
        "#,
        )
        .expect("Couldn't deserialize");
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn it_serializes_with_some_maybe_option_field() {
        let expected = r#"{"a":"Hello!","b":42,"c":null}"#;
        let actual = serde_json::to_string(&Dto {
            a: "Hello!".into(),
            b: Maybe::Some(42),
            c: None,
        })
        .expect("Couldn't serialize");
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn it_serializes_with_none_maybe_option_field() {
        let expected = r#"{"a":"Hello!","b":null,"c":101}"#;
        let actual = serde_json::to_string(&Dto {
            a: "Hello!".into(),
            b: Maybe::None,
            c: Some(101),
        })
        .expect("Couldn't serialize");
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn it_serializes_with_undef_maybe_option_field() {
        let expected = r#"{"a":"Hello!","c":101}"#;
        let actual = serde_json::to_string(&Dto {
            a: "Hello!".into(),
            b: Maybe::Void,
            c: Some(101),
        })
        .expect("Couldn't serialize");
        assert_eq!(expected, actual);
    }
}
