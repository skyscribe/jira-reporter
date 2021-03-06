use super::super::datatypes::ParsedData;
use super::super::utils::*;
use crate::query::issue::Issue;
use serde_json::Value;

const FS2EE_FIELDS_SUMMARY: &str = "summary";
const FS2EE_FIELDS_DESCRIPT: &str = "description";
const FS2EE_FIELDS_STATUS: &str = "status";
const FS2EE_FIELDS_TITLE: &str = "customfield_38703";
const FS2EE_FIELDS_EE: &str = "customfield_38692";
const FS2EE_FIELDS_RELEASE: &str = "customfield_38724";

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Fs2Fields {
    #[serde(rename = "customfield_38692")]
    pub efforts: Value,

    #[serde(rename = "customfield_38703")]
    pub title: Value,

    #[serde(rename = "customfield_38724")]
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
            None => String::from(NA_STRING),
        }
    }

    pub fn get_title_display(&self) -> String {
        get_wrapped_or_na(&self.fields.title).to_string()
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
        get_releases_from(&self.fields.release)
    }

    //get status string per json map
    pub fn get_status(&self) -> String {
        get_wrapped_object_attr(&self.fields.status, "name").to_string()
    }
}

impl ParsedData for Fs2Issue {
    //get field lists
    fn get_field_list() -> Vec<String> {
        vec![
            FS2EE_FIELDS_SUMMARY,
            FS2EE_FIELDS_TITLE,
            FS2EE_FIELDS_EE,
            FS2EE_FIELDS_RELEASE,
            FS2EE_FIELDS_DESCRIPT,
            FS2EE_FIELDS_STATUS,
        ]
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}
