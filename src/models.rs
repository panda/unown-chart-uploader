use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(rename = "statusCode")]
    pub status_code: u64,
    pub description: String,
    pub body: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct ImportResult {
    pub chart_hash: String,
    pub title: String,
    pub artist: String,
    pub level: u8,
    pub difficulty: u8,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub path: std::path::PathBuf,
    pub success: bool,
    pub message: String,
}
