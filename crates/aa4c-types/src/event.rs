use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 引擎推送的事件：`{"type": "transfer.progress", "at": …, "data": {…}}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub kind: String,
    /// 毫秒级 Unix 时间戳，由引擎盖章（本 crate 不碰时钟）。
    pub at: u64,
    pub data: Value,
}
