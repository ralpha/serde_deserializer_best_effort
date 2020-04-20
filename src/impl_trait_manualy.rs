

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

use crate::deserialize_best_effort::{DeserializeBestEffort, DeserializeBestEffortTypes};
use crate::RootWorkingManualImpl;
use std::collections::HashMap;
use serde_json::Value;

// This code is derived from:
//    https://serde.rs/deserialize-struct.html
// And a little bit of:
//    https://serde.rs/impl-deserializer.html
impl<'de> DeserializeBestEffort<'de> for RootWorkingManualImpl {}
impl<'de> Deserialize<'de> for RootWorkingManualImpl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Field1, Field2, Unknown(String) };
        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("Did not expect this... se default is not working.")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "field1" => Ok(Field::Field1),
                            "field2" => Ok(Field::Field2),
                            // _ => Err(de::Error::unknown_field(value, FIELDS)),
                            _ => Ok(Field::Unknown(value.to_string())),
                        }
                    }
                }
                // deserialize_any
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct RootWorkingManualImplVisitor;

        impl<'de> Visitor<'de> for RootWorkingManualImplVisitor {
            type Value = RootWorkingManualImpl;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RootWorkingManualImpl")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<RootWorkingManualImpl, V::Error>
            where
                V: SeqAccess<'de>,
            {
                // note used for struct it seems
                let field1 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let field2 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(RootWorkingManualImpl{
                    field1,
                    field2,
                    ..Default::default()
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<RootWorkingManualImpl, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut field1:Vec<String> = Default::default();
                let mut field2:Vec<String> = Default::default();
                let mut unknown:HashMap<String, Value> = Default::default();
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Field1 => {
                            let next_value = map.next_value().unwrap_or_default();
                            field1.add_data("field1",next_value);
                        }
                        Field::Field2 => {
                            let next_value = map.next_value().unwrap_or_default();
                            field2.add_data("field2",next_value);
                        }
                        Field::Unknown(key_name) => {
                            let next_value = map.next_value().unwrap_or_default();
                            unknown.add_data(&key_name,next_value);
                        }
                    }
                }
                Ok(RootWorkingManualImpl{
                    field1,
                    field2,
                    unknown,
                    // ..Default::default()
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["field1", "field2"];
        deserializer.deserialize_struct("RootWorkingManualImpl" , FIELDS, RootWorkingManualImplVisitor)
   }
}
