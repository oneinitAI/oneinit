use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

/// oneinit.yaml 配置文件结构
#[derive(Debug, Deserialize)]
pub struct SyncConfig {
    /// 环境工具列表（如 python: 3.11, node: 18）
    pub envs: BTreeMap<String, serde_yaml::Value>,
    /// 镜像源配置（如 pip: tsinghua, npm: taobao）
    pub mirrors: Option<BTreeMap<String, String>>,
    /// 安装后执行的命令列表
    pub post_install: Option<Vec<String>>,
}

/// 从 oneinit.yaml 文件解析配置
pub fn load_config(yaml_path: &Path) -> crate::core::Result<SyncConfig> {
    let content = std::fs::read_to_string(yaml_path)?;
    let config: SyncConfig = serde_yaml::from_str(&content)
        .map_err(|e| crate::core::CoreError::Other(format!("YAML 解析失败: {}", e)))?;
    Ok(config)
}

/// 将 envs 中的键值对转换为配方名
/// 例如: "python" + "3.11" → "python3.11"
pub fn envs_to_recipe_names(config: &SyncConfig) -> Vec<String> {
    config
        .envs
        .iter()
        .map(|(name, version)| {
            let ver_str = match version {
                serde_yaml::Value::String(s) => s.clone(),
                serde_yaml::Value::Number(n) => n.to_string(),
                other => format!("{:?}", other),
            };
            format!("{}{}", name, ver_str)
        })
        .collect()
}

/// 执行 post_install 命令列表
pub fn run_post_install(commands: &[String], formatter: &crate::output::OutputFormatter) -> crate::core::Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    for (i, cmd_str) in commands.iter().enumerate() {
        formatter.output(
            &format!("⚡ [{}] 执行: {}", i + 1, cmd_str),
            Some(serde_json::json!({
                "step": i + 1,
                "command": cmd_str,
            })),
        );

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", cmd_str])
                .output()?
        } else {
            Command::new("sh")
                .args(["-c", cmd_str])
                .output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if !stdout.trim().is_empty() {
                formatter.output(
                    &format!("   {}", stdout.trim()),
                    Some(serde_json::Value::Null),
                );
            }
        } else {
            formatter.output(
                &format!("   ❌ 命令失败 (exit code {:?})", output.status.code()),
                Some(serde_json::json!({
                    "status": "failed",
                    "exit_code": output.status.code(),
                    "stderr": stderr.trim(),
                })),
            );
            // post_install 失败不中断，继续执行后续命令
        }
    }

    Ok(())
}
