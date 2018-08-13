extern crate serde;
extern crate serde_json;

use self::serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Issue <T> { 
    expand: String,
    id: String,

    //HATEOS link for next visit
    #[serde(rename="self")]
    pub self_link: String,

    //actual key shown in UI
    pub key: String,

    //Field structure
    #[serde(bound(deserialize = "T:Deserialize<'de>"))]
    pub fields : T,
}