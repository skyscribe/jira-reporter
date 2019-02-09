extern crate serde;
extern crate serde_json;

use self::serde::de::Deserialize;

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Records<T> {
    pub timestamp: u64,

    #[serde(bound(deserialize = "T:Deserialize<'de>"))]
    pub records: Vec<T>,
}

use std::time::SystemTime;
impl<T> Records<T> {
    pub fn new(records: Vec<T>) -> Records<T> {
        Records {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            records,
        }
    }
}
