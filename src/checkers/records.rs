extern crate serde;
extern crate serde_json;

use self::serde::de::Deserialize;

#[derive(Deserialize, Serialize)]
pub (crate) struct Records <T> {
    pub timestamp: u64, 
    
    #[serde(bound(deserialize = "T:Deserialize<'de>"))]
    pub records: Vec<T>,
}