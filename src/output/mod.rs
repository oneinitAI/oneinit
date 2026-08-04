//! 统一输出格式化模块
//! 支持 human-readable 和 --json 两种输出模式

use crate::core::CoreError;
use serde::Serialize;
use std::cell::RefCell;

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
    /// Global `-y/--yes`: skip interactive confirmations
    pub auto_yes: bool,
    /// Global `-v/--debug`: emit extra debug lines
    pub debug: bool,
    /// Active JSON document action name; while set, `output` items are
    /// buffered and flushed as one document by `end_document`.
    json_doc: RefCell<Option<String>>,
    json_items: RefCell<Vec<serde_json::Value>>,
}

impl OutputFormatter {
    pub fn new(json_mode: bool) -> Self {
        Self {
            json_mode,
            auto_yes: false,
            debug: false,
            json_doc: RefCell::new(None),
            json_items: RefCell::new(Vec::new()),
        }
    }

    /// 是否为 JSON 模式
    pub fn is_json(&self) -> bool {
        self.json_mode
    }

    /// Debug mode enabled?
    pub fn debug_mode(&self) -> bool {
        self.debug
    }

    /// Emit a debug line (only when --debug is set; JSON mode gets a structured line)
    pub fn debug_line(&self, msg: &str) {
        if self.debug {
            if self.json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "level": "debug",
                        "message": msg
                    }))
                    .unwrap_or_default()
                );
            } else {
                eprintln!("[DEBUG] {}", msg);
            }
        }
    }

    /// Begin collecting subsequent `output` items into a single JSON document
    /// (human mode: no-op). Nested calls are ignored until `end_document`.
    pub fn begin_document(&self, action: &str) {
        if self.json_mode && self.json_doc.borrow().is_none() {
            *self.json_doc.borrow_mut() = Some(action.to_string());
            self.json_items.borrow_mut().clear();
        }
    }

    /// Flush buffered items as one `{action, count, items}` object
    /// (human mode: no-op).
    pub fn end_document(&self) {
        if !self.json_mode {
            return;
        }
        let Some(action) = self.json_doc.borrow_mut().take() else {
            return;
        };
        let items = std::mem::take(&mut *self.json_items.borrow_mut());
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": action,
                "count": items.len(),
                "items": items,
            }))
            .unwrap_or_default()
        );
    }

    /// 输出成功结果
    pub fn output<T: Serialize>(&self, human_text: &str, json_data: Option<T>) {
        if !self.json_mode {
            println!("{}", human_text);
            return;
        }
        // JSON 模式：转成 Value；纯装饰行（null）直接抑制，不输出
        let value = match json_data {
            Some(data) => serde_json::to_value(&data).unwrap_or(serde_json::Value::Null),
            None => serde_json::json!({ "status": "success", "message": human_text }),
        };
        if value.is_null() {
            return; // decorative human-only line
        }
        if self.json_doc.borrow().is_some() {
            self.json_items.borrow_mut().push(value);
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).unwrap_or_default()
            );
        }
    }

    /// 输出错误信息（附带解决建议 HINT）
    pub fn error(&self, err: &CoreError) {
        let suggestion = err.suggestion();
        if self.json_mode {
            let value = serde_json::json!({
                "status": "error",
                "error": err.to_string(),
                "suggestion": suggestion,
            });
            if self.json_doc.borrow().is_some() {
                self.json_items.borrow_mut().push(value);
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                );
            }
        } else {
            eprintln!("[ERROR] {}", err);
            if let Some(hint) = suggestion {
                eprintln!("[HINT] {}", hint);
            }
        }
    }
}
