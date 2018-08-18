extern crate serde;
extern crate serde_json;

use self::serde_json::Value;
use query::issue::Issue;

pub(crate) const FS2EE_FIELDS_SUMMARY  : &'static str = "summary";
pub(crate) const FS2EE_FIELDS_TITLE    : &'static str = "customfield_38703";
pub(crate) const FS2EE_FIELDS_EE       : &'static str = "customfield_38692";

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Fs2Fields {
    #[serde(rename="customfield_38692")]
    pub efforts : Value,

    #[serde(rename="customfield_38703")]
    pub title: Value,

    pub summary: String,
}

pub type Fs2Issue = Issue<Fs2Fields>;

impl Fs2Issue {
    pub fn get_efforts_display(&self) -> String {
        match self.get_efforts() {
            Some(ref effort) => effort.to_string(),
            None => String::from("NA"),
        }
    }

    pub fn get_title_display(&self) -> String {
        match self.fields.title {
            Value::String(ref some) => some.clone(),
            _ => "NA".to_string(), 
        }
    }

    //check if we have set efforts
    pub fn has_efforts(&self) -> bool {
        self.get_efforts().is_some()
    }

    //get my efforts
    pub fn get_efforts(&self) -> Option<u32> {
        match self.fields.efforts {
            Value::Number(ref efforts) => efforts.as_f64().map(|f| f as u32),
            _ => None,
        }
    }
}