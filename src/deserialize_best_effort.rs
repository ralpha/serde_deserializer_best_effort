
use serde::{Deserialize};
use std::collections::HashMap;
use serde_json::Value;

pub trait DeserializeBestEffort<'de>: Deserialize<'de>{}

pub trait DeserializeBestEffortTypes<'de, T> where
    T: DeserializeBestEffortTypes<'de,T>{
    fn add_data(&mut self, key: &str, next_value: T);
}

impl<'de> DeserializeBestEffortTypes<'de,i32> for i32{
    fn add_data(&mut self, _key: &str, next_value: i32){
        *self = next_value;
    }
}

impl<'de> DeserializeBestEffortTypes<'de,u32> for u32{
    fn add_data(&mut self, _key: &str, next_value: u32){
        *self = next_value;
    }
}

impl<'de> DeserializeBestEffortTypes<'de,String> for String {
    fn add_data(&mut self, _key: &str, next_value: String){
        *self = next_value;
    }
}

impl<'de> DeserializeBestEffortTypes<'de,Value> for Value {
    fn add_data(&mut self, _key: &str, next_value: Value){
        *self = next_value;
    }
}

impl<'de> DeserializeBestEffortTypes<'de, Value> for HashMap<String, Value> {
    fn add_data(&mut self, key: &str, next_value: Value){
        self.insert(key.to_string(), next_value);
    }
}

impl<'de,T> DeserializeBestEffortTypes<'de,T> for T where
    T: DeserializeBestEffort<'de> + Default {
    fn add_data(&mut self, _key: &str, next_value: T){
        *self = next_value;
    }
}

impl<'de,T> DeserializeBestEffortTypes<'de,T> for Vec<T> where
    T: DeserializeBestEffortTypes<'de,T> + Default {
    fn add_data(&mut self, _key: &str, next_value: T){
        self.push(next_value);
    }
}

impl<'de,T> DeserializeBestEffortTypes<'de,T> for Option<T> where
    T: DeserializeBestEffortTypes<'de,T> + Default {
    fn add_data(&mut self, _key: &str, next_value: T){
        *self = Some(next_value);
    }
}

impl<'de> DeserializeBestEffortTypes<'de,String> for Option<()> {
    fn add_data(&mut self, _key: &str, _next_value: String){
        *self = Some(());
    }
}

impl<'de,T> DeserializeBestEffortTypes<'de,T> for Option<Vec<T>> where
    T: DeserializeBestEffortTypes<'de,T> + Default {
    fn add_data(&mut self, _key: &str, next_value: T){
        if self.is_none() {
            *self = Some(Vec::new());
        }
        if let Some(list) = self{
            list.push(next_value);
        }
    }
}
