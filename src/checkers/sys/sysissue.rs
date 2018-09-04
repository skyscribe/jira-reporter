extern crate serde;
extern crate serde_json;
extern crate itertools;

use query::issue::Issue;
use super::super::datatypes::ParsedData;
use super::super::utils::*;
use self::serde_json::Value;

const SYS_FIELDS_SUMMARY   : &'static str = "summary";
const SYS_FIELDS_AREA      : &'static str = "customfield_38711";
const SYS_FIELDS_STATUS    : &'static str = "status";
const SYS_FIELDS_TITLE    : &'static str = "customfield_38703";
const SYS_FIELDS_RELEASE  : &'static str = "customfield_38724";

#[derive(Deserialize, Debug, Clone)]
pub struct SysFields {
    #[serde(rename="customfield_38703")]
    pub title: Value,

    #[serde(rename="customfield_38724")]
    pub release: Value, 

    pub summary: String,

    #[serde(rename="customfield_38711")]
    pub area: Value,
    pub status: Value,
}

pub type SysIssue = Issue<SysFields>;

impl SysIssue {
    pub fn get_title(&self) -> String {
        match self.fields.title {
            Value::String(ref some) => some.clone(),
            _ => "NA".to_string(), 
        }
    }

    pub fn get_release(&self) -> String {
        get_releases_from(&self.fields.release)
    }

    pub fn get_status(&self) -> String {
        get_wrapped_object_attr(&self.fields.status, "name").to_string()
    }

    pub fn get_area(&self) -> String {
        get_wrapped_string(&self.fields.area, "").to_string()
    }
}

impl ParsedData for SysIssue {
    //get field lists
    fn get_field_list() -> Vec<String> {
        vec![SYS_FIELDS_SUMMARY, SYS_FIELDS_AREA, SYS_FIELDS_STATUS, 
                SYS_FIELDS_TITLE, SYS_FIELDS_RELEASE]
            .iter().map(|x| x.to_string()).collect()
    }
}

/// tests
#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn should_parse_records_with_null() {
        let json = get_test_json(); 
        let issue = serde_json::from_str::<SysIssue>(&json);
        assert!(issue.is_ok(), "{:?}", issue);
    }

    pub fn get_test_json() -> &'static str {
        r#"{
            "expand":"operations,versionedRepresentations,editmeta,changelog,renderedFields",
            "id":"4280470",
            "self":"https://jiradc.int.net.nokia.com/rest/api/2/issue/4280470",
            "key":"FFB-8738",
            "fields":{"summary":"Sample 5G Feature | 5GC008888",
            "customfield_38711":null,
            "customfield_38724":[{
                "self":"https://jiradc.int.net.nokia.com/rest/api/2/customFieldOption/209818",
                "value":"5G Future Release",
                "id":"209818"}],
            "customfield_38703":null,
            "status":{"self":"https://jiradc.int.net.nokia.com/rest/api/2/status/10044",
                "description":"When new FI, User Story or Task is created.",
                "iconUrl":"https://jiradc.int.net.nokia.com/images/icons/statuses/open.png",
                "name":"New",
                "id":"10044","statusCategory":
                    {"self":"https://jiradc.int.net.nokia.com/rest/api/2/statuscategory/2",
                    "id":2,"key":"new","colorName":"blue-gray","name":"To Do"}}}
        }"#
    }
}