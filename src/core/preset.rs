// 预置套装系统 — 一键安装开发场景所需的全部工具

/// 预置套装定义
#[derive(Debug, Clone)]
pub struct Preset {
    /// 套装标识符（如 "python", "ai", "frontend"）
    pub name: String,
    /// 显示名称（如 "Python 开发环境"）
    pub display_name: String,
    /// 套装描述
    pub description: String,
    /// 包含的recipe名列表（如 ["python3.11"]）
    pub packages: Vec<String>,
}

/// 根据套装名查找预置套装
pub fn resolve(name: &str) -> Option<Preset> {
    match name {
        "python" => Some(python_preset()),
        "ai" => Some(ai_preset()),
        "frontend" => Some(frontend_preset()),
        "full" => Some(full_preset()),
        _ => None,
    }
}

/// 列出所有预置套装
pub fn list_presets() -> Vec<Preset> {
    vec![
        python_preset(),
        ai_preset(),
        frontend_preset(),
        full_preset(),
    ]
}

// ============================================================
// 内置套装定义
// ============================================================

/// Python 开发环境
fn python_preset() -> Preset {
    Preset {
        name: "python".to_string(),
        display_name: "Python 开发环境".to_string(),
        description: "Python 3.11 + pip 清华源".to_string(),
        packages: vec!["python3.11".to_string()],
    }
}

/// AI 开发套装
fn ai_preset() -> Preset {
    Preset {
        name: "ai".to_string(),
        display_name: "AI 开发套装".to_string(),
        description: "Python 3.11（机器学习/深度学习基础环境）".to_string(),
        packages: vec!["python3.11".to_string()],
    }
}

/// 前端开发套装
fn frontend_preset() -> Preset {
    Preset {
        name: "frontend".to_string(),
        display_name: "前端开发套装".to_string(),
        description: "（暂无可用recipe，等待 Node.js recipe实现）".to_string(),
        packages: vec![],
    }
}

/// 全栈开发套装
fn full_preset() -> Preset {
    Preset {
        name: "full".to_string(),
        display_name: "全栈开发套装".to_string(),
        description: "安装所有可用recipe".to_string(),
        packages: vec!["python3.11".to_string()],
    }
}
