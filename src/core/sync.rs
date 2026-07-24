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
pub fn run_post_install(
    commands: &[String],
    formatter: &crate::output::OutputFormatter,
) -> crate::core::Result<()> {
    if commands.is_empty() {
        return Ok(());
    }

    for (i, cmd_str) in commands.iter().enumerate() {
        formatter.output(
            &format!("[RUN] [{}] 执行: {}", i + 1, cmd_str),
            Some(serde_json::json!({
                "step": i + 1,
                "command": cmd_str,
            })),
        );

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", cmd_str]).output()?
        } else {
            Command::new("sh").args(["-c", cmd_str]).output()?
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
                &format!("   [ERROR] 命令失败 (exit code {:?})", output.status.code()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envs_to_recipe_names() {
        let mut envs = BTreeMap::new();
        envs.insert(
            "python".to_string(),
            serde_yaml::Value::String("3.11".to_string()),
        );
        envs.insert(
            "node".to_string(),
            serde_yaml::Value::Number(serde_yaml::Number::from(18)),
        );

        let config = SyncConfig {
            envs,
            mirrors: None,
            post_install: None,
        };

        let names = envs_to_recipe_names(&config);
        assert!(names.contains(&"python3.11".to_string()));
        assert!(names.contains(&"node18".to_string()));
    }

    #[test]
    fn test_envs_to_recipe_names_empty() {
        let config = SyncConfig {
            envs: BTreeMap::new(),
            mirrors: None,
            post_install: None,
        };
        assert!(envs_to_recipe_names(&config).is_empty());
    }

    #[test]
    fn test_load_config_valid() {
        let yaml = "envs:\n  python: \"3.11\"\nmirrors:\n  pip: tsinghua\n";
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_test_valid.yaml");
        std::fs::write(&path, yaml).unwrap();

        let config = load_config(&path).unwrap();
        assert_eq!(config.envs.len(), 1);
        assert!(config.mirrors.is_some());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_config_invalid_yaml() {
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_test_invalid.yaml");
        std::fs::write(&path, "this: is: not: valid: yaml: {{{").unwrap();

        let result = load_config(&path);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&path);
    }
}
