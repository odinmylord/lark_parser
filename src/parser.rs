use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Deserialize, Debug)]
pub struct ParserNode {
    pub send_channel: Option<String>,
    pub receive_channel: Option<String>,
    pub message: Option<String>,
    pub direction: Option<String>,
    pub if_statem: Option<Vec<ParserNode>>,
    pub else_statem: Option<Vec<ParserNode>>,
}

pub fn data_parser<'de, R>(
    mut deserializer: serde_json::Deserializer<R>,
) -> Result<HashMap<String, Vec<ParserNode>>, serde_json::Error>
where
    R: serde_json::de::Read<'de>,
{
    struct DataVisitor;

    impl<'de> Visitor<'de> for DataVisitor {
        type Value = HashMap<String, Vec<ParserNode>>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("a map of strings to vectors of messages")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut result: HashMap<String, Vec<ParserNode>> = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, Vec<ParserNode>>()? {
                if let Some(existing) = result.get_mut(&key) {
                    existing.extend(value);
                } else {
                    result.insert(key, value);
                }
            }
            Ok(result)
        }
    }

    let visitor = DataVisitor;
    deserializer.deserialize_map(visitor)
}
