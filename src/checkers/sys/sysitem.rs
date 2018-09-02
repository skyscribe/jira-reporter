use super::sysissue::SysIssue;
use super::super::datatypes::StoredData;
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct SysItem {
    pub summary: String,
    pub title: String,
    pub release: String,
    pub status: String,
    pub area: String,
    pub key: String,
}

impl SysItem {
    pub fn from(raw: &SysIssue) -> SysItem {
        SysItem {
            summary: raw.fields.summary.clone(),
            title: raw.get_title(),
            release: raw.get_release(),
            status: raw.get_status(),
            area: raw.get_area(),
            key: raw.key.clone(),
        }
    }
}

impl Ord for SysItem {
    fn cmp(&self, other: &SysItem) -> Ordering {
        self.area.cmp(&other.area)
            .then(self.summary.cmp(&other.summary))
            .then(self.status.cmp(&other.status))
            .then(self.key.cmp(&other.key))
    }
}

impl PartialOrd for SysItem {
    fn partial_cmp(&self, other:&SysItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SysItem {
    fn eq(&self, other:&SysItem) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl StoredData for SysItem {
    type Parsed = SysIssue;

    fn parse_from(issue: &Self::Parsed) -> Self {
        Self::from(issue)
    }
}