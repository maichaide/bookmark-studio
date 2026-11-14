use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub id: u64,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub folders: Vec<String>,
    #[serde(default)]
    pub add_date: Option<String>,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortField {
    Title,
    Tags,
    Url,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Default)]
pub struct DuplicateRules {
    pub ignore_fragment: bool,
    pub ignore_query: bool,
    pub ignore_trailing_slash: bool,
    pub ignore_scheme: bool,
    pub ignore_www: bool,
    pub lowercase_host: bool,
}

impl DuplicateRules {
    pub fn is_strict(&self) -> bool {
        !self.ignore_fragment
            && !self.ignore_query
            && !self.ignore_trailing_slash
            && !self.ignore_scheme
            && !self.ignore_www
            && !self.lowercase_host
    }
}
