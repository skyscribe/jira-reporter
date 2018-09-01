use super::fs2issue::Fs2Issue;
use super::super::datatypes::StoredData;
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Fs2Item {
    pub summary: String,
    pub efforts: i32,
    pub title: String,
    pub release: String,
    pub description: String,
    pub status: String,
}

impl Fs2Item {
    pub fn from(raw: &Fs2Issue) -> Fs2Item {
        Fs2Item {
            summary: raw.fields.summary.clone(),
            title: raw.get_title_display(),
            efforts: raw.get_efforts().map(|x| x as i32).unwrap_or(-1),
            release: raw.get_release(),
            description: raw.get_description(),
            status: raw.get_status().clone(),
        }
    }

    pub fn has_efforts(&self) -> bool {
        self.efforts != -1
    }
}

impl Ord for Fs2Item {
    fn cmp(&self, other: &Fs2Item) -> Ordering {
        self.summary.cmp(&other.summary)
            .then(self.title.cmp(&other.title))
            .then(self.efforts.cmp(&other.efforts))
    }
}

impl PartialOrd for Fs2Item {
    fn partial_cmp(&self, other:&Fs2Item) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Fs2Item {
    fn eq(&self, other:&Fs2Item) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl StoredData for Fs2Item {
    type Parsed = Fs2Issue;

    fn parse_from(issue: &Self::Parsed) -> Self {
        Self::from(issue)
    }
}