use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use std::fs::File;
use failure::Error;
use std::io::BufReader;


// Imports needed for custom trait/derive
mod deserialize_best_effort;
use deserialize_best_effort::{DeserializeBestEffort, DeserializeBestEffortTypes};
use custom_derive::{DeserializeBestEffort};
use serde::de;
use std::collections::HashMap;
use serde_json::Value;

// This code is an example for parsing files with duplicate tags in xml
// This fixes the problem described in:
//    https://github.com/RReverser/serde-xml-rs/issues/55
// This code of the `custom_derive` is not writen very well,
// but shows that it can be done.
fn main() {
    let file_path = "./test.xml";

    #[allow(dead_code)]
    enum TestSelect{
        NotWorking, Working, WorkingManualImpl,
    }
    let selected_test: TestSelect = TestSelect::WorkingManualImpl;

     match selected_test {
        TestSelect::NotWorking => {
            let parse_xml = parse_file_now_working(file_path); // This SHOULD give an error:
            // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
            // Custom { field: "duplicate field `field1`" }
            println!("Print Parsed output: {:#?}", parse_xml);
        },
        TestSelect::Working => {
            let parse_xml = parse_file_working(file_path);
            println!("Print Parsed output: {:#?}", parse_xml);
        },
        TestSelect::WorkingManualImpl => {
            let parse_xml = parse_file_working_manual_impl(file_path);
            println!("Print Parsed output: {:#?}", parse_xml);
        }
    }
}


#[derive(Serialize, Debug, Deserialize, Clone, Default)]
pub struct RootNotWorking {
    pub field1: Vec<String>,
    pub field2: Vec<String>,
}

// Default trait is needed for providing data, but could be removed if
// `Option<T>` is used for variables as in the example:
// https://serde.rs/deserialize-struct.html
#[derive(Serialize, Debug, DeserializeBestEffort, Clone, Default)]
pub struct RootWorking {
    pub field1: Vec<String>,
    pub field2: Vec<String>,

    // this is needed because of how I wrote the deserializer
    // but is not needed for actual demo
    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

// Trait implemented manually see `impl_trait_manualy.rs`
mod impl_trait_manualy;
#[derive(Serialize, Debug, Clone, Default)]
pub struct RootWorkingManualImpl {
    pub field1: Vec<String>,
    pub field2: Vec<String>,

    // this is needed because of how I wrote the deserializer
    // but is not needed for actual demo
    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}


// function of reading file
fn read_xml_file<C: DeserializeOwned>
(filename: &str) -> Result<C, Error>{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let parsed_object: C = serde_xml_rs::from_reader(reader)?;
    Ok(parsed_object)
}

pub fn parse_file_now_working(filename: &str) -> RootNotWorking{
    read_xml_file( filename ).unwrap()
}

pub fn parse_file_working(filename: &str) -> RootWorking{
    read_xml_file( filename ).unwrap()
}

pub fn parse_file_working_manual_impl(filename: &str) -> RootWorking{
    read_xml_file( filename ).unwrap()
}
