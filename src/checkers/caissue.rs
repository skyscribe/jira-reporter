extern crate serde;
extern crate serde_json;

use self::serde_json::Value;
use query::issue::Issue;

pub(crate) const CA_FIELDS_SUMMARY      : &'static str = "summary";
pub(crate) const CA_FIELDS_FEATUREID    : &'static str = "customfield_37381";
pub(crate) const CA_FIELDS_TEAM         : &'static str = "customfield_38727";
pub(crate) const CA_FIELDS_STARTFB      : &'static str = "customfield_38694";
pub(crate) const CA_FIELDS_ENDFB        : &'static str = "customfield_38693";
pub(crate) const CA_FIELDS_TYPE         : &'static str = "customfield_38750";

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct CAFields {
    pub summary: String,

    #[serde(rename="customfield_37381")]
    pub feature_id: String,

    #[serde(rename="customfield_38727")]
    pub team : Value, //string or null

    #[serde(rename="customfield_38694")]
    pub start_fb: Value,

    #[serde(rename="customfield_38693")]
    pub end_fb: Value,

    #[serde(rename="customfield_38750")]
    pub activity_type: Value,
}

pub type CAIssue = Issue<CAFields>;

impl CAIssue {
    pub fn log(&self) {
        info!("|{}|{}|{}|{}|{}|{}|", self.fields.summary, self.fields.feature_id, 
            self.fields.activity_type, self.fields.team, self.fields.start_fb, 
            self.fields.start_fb);
    }
}