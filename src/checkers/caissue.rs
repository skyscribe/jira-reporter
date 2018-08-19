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
const NA_STRING : &'static str = "NA";
const DEFAULT_FB : u32 = 9999;

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct CAFields {
    pub summary: String,

    #[serde(rename="customfield_37381")]
    pub feature_id: Value,

    #[serde(rename="customfield_38727")]
    pub team : Value, //string or null

    #[serde(rename="customfield_38694")]
    pub start_fb: Value,

    #[serde(rename="customfield_38693")]
    pub end_fb: Value,

    #[serde(rename="customfield_38750")]
    pub activity_type: Value,
}

pub struct CAItem {
    pub summary: String,
    pub feature_id: String,
    pub team: String,
    pub start_fb: u32,
    pub end_fb: u32,
    pub activity: String,
}

impl CAItem {
    pub fn from(issue: &CAIssue) -> CAItem {
        CAItem {
            summary: issue.fields.summary.clone(),
            feature_id: issue.get_fid().to_string(),
            team: issue.get_team().to_string(),
            start_fb: convert_fb(issue.get_start()),
            end_fb: convert_fb(issue.get_end()),
            activity: issue.get_type().to_string(),
        }
    }

    pub fn get_summary(&self) -> (&str, &str) {
        //split by first " " only
        match self.summary.find(|x:char| x==' ' || x == '\t') {
            Some(x) => {
                let (first, last) = self.summary.split_at(x);
                let skips: &[_] = &[' ', '-', '\t'];
                (first, last.trim_left_matches(skips))
            },
            None => (&self.summary, " "),
        }
    }
}

pub type CAIssue = Issue<CAFields>;

fn get_wrapped_string(value:&Value) -> &str {
    match value {
        Value::String(ref some) => some,
        _ => &NA_STRING,
    }
}

fn convert_fb(value: &str) -> u32 {
    if value == NA_STRING {
        DEFAULT_FB.clone()
    } else {
        match u32::from_str_radix(value, 10) {
            Ok(x) => x,
            Err(_) => DEFAULT_FB.clone(),
        }
    }
}

impl CAIssue {
    pub fn log(&self) {
        info!("|{}|{}|{}|{}|{}|{}|", self.fields.summary, self.fields.feature_id, 
            self.fields.activity_type, self.fields.team, self.fields.start_fb, 
            self.fields.start_fb);
    }

    //get summary info - actual id and description
    pub fn get_summary(&self) -> (&str, &str) {
        //split by first " " only
        match self.fields.summary.find(|x:char|x == ' ' || x == '\t') {
            Some(x) => self.fields.summary.split_at(x),
            None => (&self.fields.summary, " "),
        }
    }

    pub fn get_fid(&self) -> &str {
        get_wrapped_string(&self.fields.feature_id)
    }

    pub fn get_type(&self) -> &str {
        match self.fields.activity_type {
            Value::Object(ref obj) => {
                match obj["value"] {
                    Value::String(ref x) => {
                        match x.find("Entity Specification") {
                            Some(_) => "EFS",
                            None => match x.find("Entity Testing") {
                                Some(_) => "ET",
                                None => x,
                            },
                        }
                    } 
                    _ => &NA_STRING,
                }
            },
            _ => &NA_STRING,
        }
    }

    pub fn get_start(&self) -> &str {
        get_wrapped_string(&self.fields.start_fb)
    }

    pub fn get_end(&self) -> &str {
        get_wrapped_string(&self.fields.end_fb)
    }

    pub fn get_team(&self) -> &str {
        get_wrapped_string(&self.fields.team)
    }
}