use serde::{Deserialize, Deserializer, Serializer};
use time::OffsetDateTime;

pub fn serialize_datetime<S>(dt: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Format without the +00 prefix
    let s = dt
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(s.trim_start_matches("+00"))
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String as Deserialize>::deserialize(deserializer)?;
    OffsetDateTime::parse(&s, &time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(serde::de::Error::custom)
}

pub fn serialize_datetime_option<S>(
    dt: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(dt) => serialize_datetime(dt, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_datetime_option<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <Option<String> as Deserialize>::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(
            OffsetDateTime::parse(&s, &time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(serde::de::Error::custom)?,
        )),
        None => Ok(None),
    }
}
