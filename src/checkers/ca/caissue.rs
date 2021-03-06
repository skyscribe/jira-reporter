use crate::checkers::datatypes::ParsedData;
use crate::checkers::utils::*;
use crate::query::issue::Issue;
use serde_json::Value;

const CA_FIELDS_SUMMARY: &str = "summary";
const CA_FIELDS_FEATUREID: &str = "customfield_37381";
const CA_FIELDS_TEAM: &str = "customfield_38727";
const CA_FIELDS_STARTFB: &str = "customfield_38694";
const CA_FIELDS_ENDFB: &str = "customfield_38693";
const CA_FIELDS_TYPE: &str = "customfield_38750";
const CA_FIELDS_ORIG_EFF: &str = "timeoriginalestimate";
const CA_FIELDS_TARGET: &str = "customfield_38723";

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct CAFields {
    pub summary: String,

    #[serde(rename = "customfield_37381")]
    pub feature_id: Value,

    #[serde(rename = "customfield_38727")]
    pub team: Value, //string or null

    #[serde(rename = "customfield_38694")]
    pub start_fb: Value,

    #[serde(rename = "customfield_38693")]
    pub end_fb: Value,

    #[serde(rename = "customfield_38750")]
    pub activity_type: Value,

    #[serde(rename = "timeoriginalestimate")]
    pub original_eff: Value,

    #[serde(rename = "customfield_38723")]
    pub target_pt: Value,
}

pub type CAIssue = Issue<CAFields>;

impl CAIssue {
    pub fn log(&self) {
        info!(
            "|{}|{}|{}|{}|{}|{}|",
            self.fields.summary,
            self.fields.feature_id,
            self.fields.activity_type,
            self.fields.team,
            self.fields.start_fb,
            self.fields.start_fb
        );
    }

    pub fn get_fid(&self) -> &str {
        get_wrapped_or_na(&self.fields.feature_id)
    }

    pub fn get_start(&self) -> &str {
        get_wrapped_or_na(&self.fields.start_fb)
    }

    pub fn get_end(&self) -> &str {
        get_wrapped_or_na(&self.fields.end_fb)
    }

    pub fn get_team(&self) -> &str {
        get_wrapped_or_na(&self.fields.team)
    }

    pub fn get_target(&self) -> &str {
        get_wrapped_or_na(&self.fields.target_pt)
    }

    pub fn get_type(&self) -> &str {
        match self.fields.activity_type {
            Value::Object(ref obj) => match obj["value"] {
                Value::String(ref x) => x,
                _ => NA_STRING,
            },
            _ => NA_STRING,
        }
    }

    pub fn get_efforts(&self) -> i32 {
        match self.fields.original_eff {
            Value::Number(ref num) => (num.as_u64().unwrap() / 3600) as i32,
            _ => {
                debug!("Not specififed efforts = <{}>!", self.fields.original_eff);
                -1
            }
        }
    }
}

impl ParsedData for CAIssue {
    //get field lists
    fn get_field_list() -> Vec<String> {
        vec![
            CA_FIELDS_FEATUREID,
            CA_FIELDS_SUMMARY,
            CA_FIELDS_TEAM,
            CA_FIELDS_TYPE,
            CA_FIELDS_STARTFB,
            CA_FIELDS_ENDFB,
            CA_FIELDS_ORIG_EFF,
            CA_FIELDS_TARGET,
        ]
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}
