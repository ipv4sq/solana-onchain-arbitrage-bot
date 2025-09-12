use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct JitoBundle {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct JitoBundleResponse {
    pub result: Option<String>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TipFloorData {
    #[allow(dead_code)]
    pub time: String,
    pub landed_tips_25th_percentile: f64,
    pub landed_tips_50th_percentile: f64,
    pub landed_tips_75th_percentile: f64,
    pub landed_tips_95th_percentile: f64,
    pub landed_tips_99th_percentile: f64,
    pub ema_landed_tips_50th_percentile: f64,
}
