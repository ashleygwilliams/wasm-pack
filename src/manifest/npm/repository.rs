#[derive(Deserialize, Serialize)]
pub struct Repository {
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}
