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

pub type CAIssue = Issue<CAFields>;

fn get_wrapped_string(value:&Value) -> &str {
    match value {
        Value::String(ref some) => some,
        _ => &NA_STRING,
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
        let v:Vec<&str> = self.fields.summary.split(" - ").collect();
        if v.len() > 1 {
            (v[0], v[v.len()-1])
        } else {
            //split by first " " then
            match self.fields.summary.find(" ") {
                Some(x) => self.fields.summary.split_at(x),
                None => (v[0], " "),
            }
        }
    }

    pub fn get_fid(&self) -> &str {
        get_wrapped_string(&self.fields.feature_id)
    }

    pub fn get_type(&self) -> &str {
        //TODO: refactor this in functional manner!
        match self.fields.activity_type {
            Value::Object(ref obj) => {
                match obj["value"] {
                    Value::String(ref x) => {
                        match x.find("Entity Specification") {
                            Some(_) => "EFS",
                            None => match x.find("Entity Testing") {
                                Some(_) => "ET",
                                None => &NA_STRING,
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
}