extern crate serde;
extern crate serde_json;

use self::serde_json::Value;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Issue {
    expand: String,
    id: String,

    //HATEOS link for next visit
    #[serde(rename="self")]
    pub self_link: String,

    //actual key shown in UI
    pub key: String,

    pub fields : Fields,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Fields {
    #[serde(rename="customfield_38692")]
    pub efforts : Value,

    #[serde(rename="customfield_38703")]
    pub title: Value,

    pub summary: String,
}

impl Issue {
    pub fn log(&self) {
        let efforts = match self.get_efforts() {
            Some(ref effort) => effort.to_string(),
            None => String::from("NA"),
        };
        let title = match self.fields.title {
            Value::String(ref some) => &some,
            _ => "NA", 
        };

        info!("|{}|{}|{}|{}", self.key, self.self_link, title, efforts);
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

#[derive(Deserialize)]
pub struct IssueList {
    issues: Vec<Issue>,
}

// parse an arry of issues into array of structure
pub fn parse_from_issue_list(json_list : &str) -> Vec<Issue> {
    let issues: IssueList = serde_json::from_str(&json_list).unwrap();
    issues.issues
}