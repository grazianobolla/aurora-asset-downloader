use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CoversResponse {
    #[serde(rename = "Covers")]
    pub covers: Vec<Cover>,

    #[serde(rename = "CoversCount")]
    pub covers_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct Cover {
    #[serde(rename = "CoverID")]
    pub cover_id: String,
}
