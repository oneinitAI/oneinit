// AI Skill 管理模块
//
// 将 oneinit 内置的 SKILL.md 自动安装到检测到的 AI 助手目录：
//   ~/.zcode/skills/oneinit/SKILL.md   (ZCode)
//   ~/.codex/skills/oneinit/SKILL.md   (Codex)
//   ~/.claude/skills/oneinit/SKILL.md  (Claude)
//   ~/.agents/skills/oneinit/SKILL.md  (通用)

use std::path::PathBuf;

use crate::output::OutputFormatter;

/// 内置 SKILL.md 内容（编译时嵌入）
const SKILL_CONTENT: &str = include_str!("../.agents/skills/oneinit/SKILL.md");

/// 检测已安装的 AI 助手目录
///
/// 返回 (助手名称, skills 目录路径) 列表
pub fn detect_agents() -> Vec<(&'static str, PathBuf)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let candidates: Vec<(&'static str, PathBuf)> = vec![
        ("zcode", home.join(".zcode/skills")),
        ("codex", home.join(".codex/skills")),
        ("claude", home.join(".claude/skills")),
        ("agents", home.join(".agents/skills")),
    ];

    candidates
        .into_iter()
        .filter(|(_, dir)| dir.exists() || dir.parent().map(|p| p.exists()).unwrap_or(false))
        .collect()
}

/// 获取指定助手的 skill 目标路径
fn skill_target(agent: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = match agent {
        "zcode" => home.join(".zcode/skills/oneinit"),
        "codex" => home.join(".codex/skills/oneinit"),
        "claude" => home.join(".claude/skills/oneinit"),
        "agents" => home.join(".agents/skills/oneinit"),
        _ => return None,
    };
    Some(dir)
}

/// 安装 Skill 到指定助手目录
pub fn install_to(agent: &str, formatter: &OutputFormatter) -> bool {
    let target_dir = match skill_target(agent) {
        Some(d) => d,
        None => {
            formatter.output(
                &format!(
                    "[ERROR] 未知的 AI 助手: {} (支持: zcode, codex, claude, agents)",
                    agent
                ),
                Some(serde_json::json!({
                    "status": "error", "action": "skill_install",
                    "agent": agent, "message": "Unknown agent"
                })),
            );
            return false;
        }
    };

    let skill_file = target_dir.join("SKILL.md");

    // 创建目录
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        formatter.output(
            &format!("[ERROR] create dir failed: {}", e),
            Some(serde_json::Value::Null),
        );
        return false;
    }

    // 写入 SKILL.md
    let was_update = skill_file.exists();
    if let Err(e) = std::fs::write(&skill_file, SKILL_CONTENT) {
        formatter.output(
            &format!("[ERROR] write failed: {}", e),
            Some(serde_json::Value::Null),
        );
        return false;
    }

    formatter.output(
        &format!(
            "[OK] {} oneinit Skill -> {}",
            if was_update { "updated" } else { "installed" },
            skill_file.display()
        ),
        Some(serde_json::json!({
            "status": "success", "action": "skill_install",
            "agent": agent, "path": skill_file.to_string_lossy(),
            "updated": was_update,
        })),
    );
    true
}

/// 安装到所有检测到的助手
pub fn install_all(formatter: &OutputFormatter) -> usize {
    let agents = detect_agents();
    if agents.is_empty() {
        formatter.output(
            "[WARN] not detected任何 AI 助手目录。手动安装：将 .agents/skills/oneinit/SKILL.md 复制到你的 AI 助手 skills 目录。",
            Some(serde_json::json!({
                "status": "warning", "action": "skill_install",
                "message": "No AI agent directories detected",
            })),
        );
        return 0;
    }

    let mut count = 0;
    for (name, _) in &agents {
        if install_to(name, formatter) {
            count += 1;
        }
    }

    formatter.output(
        &format!("[OK] Skill installed to {} AI agents", count),
        Some(serde_json::json!({
            "status": "success", "action": "skill_install",
            "installed_count": count,
            "target": "all",
        })),
    );
    count
}

/// 检查所有助手的安装状态
pub fn status(formatter: &OutputFormatter) {
    let agents = detect_agents();
    if agents.is_empty() {
        formatter.output(
            "[INFO] not detected AI 助手目录",
            Some(serde_json::json!({
                "status": "empty", "action": "skill_status",
            })),
        );
        return;
    }

    let mut results = Vec::new();
    for (name, skills_dir) in &agents {
        let skill_path = skills_dir.join("oneinit/SKILL.md");
        let installed = skill_path.exists();
        let tag = if installed { "[OK]" } else { "[--]" };
        formatter.output(
            &format!("  {} {} - {}", tag, name, skill_path.display()),
            Some(serde_json::json!({
                "agent": name, "installed": installed,
                "path": skill_path.to_string_lossy(),
            })),
        );
        results.push(serde_json::json!({
            "agent": name, "installed": installed,
        }));
    }

    let installed_count = results.iter().filter(|r| r["installed"] == true).count();
    formatter.output(
        &format!(
            "\n[INFO] {}/{} 个助手已安装 Skill",
            installed_count,
            results.len()
        ),
        Some(serde_json::json!({
            "status": "success", "action": "skill_status",
            "agents": results, "installed_count": installed_count,
        })),
    );
}

/// 从所有助手卸载 Skill
pub fn uninstall(formatter: &OutputFormatter) {
    let agents = detect_agents();
    let mut count = 0;

    for (name, skills_dir) in &agents {
        let skill_dir = skills_dir.join("oneinit");
        if skill_dir.exists() {
            if std::fs::remove_dir_all(&skill_dir).is_ok() {
                formatter.output(
                    &format!("[OK] uninstalled from {}", name),
                    Some(serde_json::json!({
                        "status": "success", "action": "skill_uninstall",
                        "agent": name,
                    })),
                );
                count += 1;
            }
        }
    }

    if count == 0 {
        formatter.output(
            "[INFO] 没有找到已安装的 Skill",
            Some(serde_json::json!({
                "status": "empty", "action": "skill_uninstall",
            })),
        );
    } else {
        formatter.output(
            &format!("[OK] Skill uninstalled from {} agents", count),
            Some(serde_json::json!({
                "status": "success", "action": "skill_uninstall",
                "uninstalled_count": count,
            })),
        );
    }
}
