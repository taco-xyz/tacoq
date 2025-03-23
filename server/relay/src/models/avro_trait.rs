use apache_avro::{from_avro_datum, from_value, to_avro_datum, types::Value, Schema};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

/// Converts a serializable type into a vector of key-value pairs suitable for
/// Avro serialization.
///
/// # Arguments
/// * `val` - Reference to a value that implements `Serialize`
///
/// # Returns
/// A vector of tuples containing static string field names and their corresponding Avro values
fn convert_to_avro_value<T: Serialize>(val: &T) -> Vec<(&'static str, Value)> {
    let value = apache_avro::to_value(val).unwrap().into();
    match value {
        Value::Record(map) => map
            .into_iter()
            .map(|(k, v)| (Box::leak(k.into_boxed_str()) as &str, v))
            .collect(),
        _ => panic!("Expected Map type"),
    }
}

/// Trait for types that can be serialized to and deserialized from Avro format.
///
/// Implementing types must provide their Avro schema and can then use the default
/// implementations for serialization and deserialization.
///
/// If you are using NaiveDateTime or Option<NaiveDateTime> you can use the helper functions
/// in the [serde_avro_datetime](crate::models::serde_avro_datetime) and
/// [serde_avro_datetime_opt](crate::models::serde_avro_datetime_opt) modules.
///
/// # Example
/// ```rust
/// use apache_avro::Schema;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyType {
///     field1: String,
///     field2: i32,
/// }
///
/// impl AvroSerializable for MyType {
///     fn schema() -> &'static Schema {
///         Schema::parse_str(r#"{"type": "record", "name": "MyType", "fields": [{"name": "field1", "type": "string"}, {"name": "field2", "type": "int"}]}"#).unwrap()
///     }
/// }
///
/// let my_type = MyType { field1: "Hello".to_string(), field2: 42 };
/// let avro_bytes = my_type.into_avro_bytes();
/// let deserialized_my_type = MyType::from_avro_bytes(&avro_bytes);
/// ```
pub trait AvroSerializable: Sized + Serialize + DeserializeOwned {
    /// Returns the Avro schema for this type
    fn schema() -> &'static Schema;

    /// Serializes the implementing type into Avro binary format
    ///
    /// # Returns
    /// A vector of bytes containing the Avro-encoded data
    fn try_into_avro_bytes(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let datum = Value::Record(
            convert_to_avro_value(self)
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        );
        to_avro_datum(Self::schema(), datum)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    /// Deserializes Avro binary data into an instance of the implementing type
    ///
    /// # Arguments
    /// * `data` - Slice of bytes containing Avro-encoded data
    ///
    /// # Returns
    /// An instance of the implementing type
    fn try_from_avro_bytes(data: &[u8]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut reader = data;
        let value = from_avro_datum(Self::schema(), &mut reader, None)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);

        match value {
            Ok(value) => {
                from_value::<Self>(&value).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
            }
            Err(e) => Err(e),
        }
    }
}

/// Helper functions for serializing and deserializing datetime values in Avro
/// format.
///
/// # Example
/// ```rust
///
/// #[derive(Serialize, Deserialize)]
/// struct MyType {
///     #[serde(with = "serde_avro_datetime")]
///     field1: NaiveDateTime,
/// }
/// ```
pub mod serde_avro_datetime {
    use chrono::{DateTime, NaiveDateTime};
    use serde::{de::Deserializer, Deserialize, Serializer};

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ts = dt.and_utc().timestamp_micros();
        serializer.serialize_i64(ts)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = i64::deserialize(deserializer)?;
        DateTime::from_timestamp_micros(ts)
            .map(|dt| dt.naive_utc())
            .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
    }
}

/// Helper functions for serializing and deserializing optional datetime
/// values in Avro format.
///
/// # Example
/// ```rust
/// #[derive(Serialize, Deserialize)]
/// struct MyType {
///     #[serde(with = "serde_avro_datetime_opt")]
///     field1: Option<NaiveDateTime>,
/// }
/// ```
pub mod serde_avro_datetime_opt {
    use super::serde_avro_datetime;
    use chrono::{DateTime, NaiveDateTime};
    use serde::{de::Deserializer, Deserialize, Serializer};

    pub fn serialize<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => serde_avro_datetime::serialize(dt, serializer),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts: Option<i64> = Option::deserialize(deserializer)?;
        Ok(ts.and_then(|ts| DateTime::from_timestamp_micros(ts).map(|dt| dt.naive_utc())))
    }
}
