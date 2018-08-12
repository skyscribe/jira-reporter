extern crate serde;
extern crate serde_json;

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
}

impl Issue {
    pub fn log(&self) {
        info!("key={},link:{}", self.key, self.self_link);
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