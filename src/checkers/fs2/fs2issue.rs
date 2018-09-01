extern crate serde;
extern crate serde_json;
extern crate itertools;

use self::serde_json::Value;
use query::issue::Issue;
use super::super::datatypes::ParsedData;
use self::itertools::Itertools;

pub(crate) const FS2EE_FIELDS_SUMMARY  : &'static str = "summary";
pub(crate) const FS2EE_FIELDS_DESCRIPT : &'static str = "description";
pub(crate) const FS2EE_FIELDS_STATUS   : &'static str = "status";
pub(crate) const FS2EE_FIELDS_TITLE    : &'static str = "customfield_38703";
pub(crate) const FS2EE_FIELDS_EE       : &'static str = "customfield_38692";
pub(crate) const FS2EE_FIELDS_RELEASE  : &'static str = "customfield_38724";

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Fs2Fields {
    #[serde(rename="customfield_38692")]
    pub efforts : Value,

    #[serde(rename="customfield_38703")]
    pub title: Value,

    #[serde(rename="customfield_38724")]
    pub release: Value, 

    pub summary: String,
    pub description: Value,
    pub status: Value,
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

    pub fn get_description(&self) -> String {
        match self.fields.description {
            Value::String(ref x) => x.clone(),
            _ => "".to_string(),
        }
    }

    //get release string
    pub fn get_release(&self) -> String {
        //println!("{} release is: {}", self.fields.summary, self.fields.release);
        match self.fields.release {
            Value::Array(ref releases) => {
                releases.iter()
                    .map(|it| get_wrapped_object_attr(&it, "value").to_string())
                    .filter(|it| it != "")
                    .join(",")
            },
            _ => "".to_string(),
        }
    }

    //get status string per json map
    pub fn get_status(&self) -> String {
        get_wrapped_object_attr(&self.fields.status, "name").to_string()
    }
}

fn get_wrapped_object_attr<'a>(value:&'a Value, attr: &str) -> &'a str {
    match value {
        Value::Object(ref obj) => {
            match obj[attr] {
                Value::String(ref x) => x,
                _ => "",
            } 
        },
        _ => "",
    }
}

impl ParsedData for Fs2Issue {
    //get field lists
    fn get_field_list() -> Vec<String> {
        vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE, 
                FS2EE_FIELDS_RELEASE, FS2EE_FIELDS_DESCRIPT, FS2EE_FIELDS_STATUS]
            .iter().map(|x| x.to_string()).collect()
    }
}