/// 统一输出格式化模块
/// 支持 human-readable 和 --json 两种输出模式

use serde::Serialize;

/// 所有命令返回的 JSON 结构统一包装
#[derive(Serialize)]
#[allow(dead_code)]
pub struct JsonResult<T: Serialize> {
    pub status: String,
    pub data: T,
}

impl<T: Serialize> JsonResult<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data,
        }
    }
}

/// 输出格式控制器
pub struct OutputFormatter {
    json_mode: bool,
}

impl OutputFormatter {
    pub fn new(json_mode: bool) -> Self {
        Self { json_mode }
    }

    /// 输出结果：json 模式输出 JSON，否则输出人类可读文本
    pub fn output<T: Serialize>(&self, human_text: &str, json_data: Option<T>) {
        if self.json_mode {
            if let Some(data) = json_data {
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            } else {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": human_text
                })).unwrap_or_default());
            }
        } else {
            println!("{}", human_text);
        }
    }
}
