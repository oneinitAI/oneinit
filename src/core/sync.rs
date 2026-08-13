use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::lockfile::Lockfile;

/// oneinit.yaml / team.yaml 配置文件结构
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SyncConfig {
    /// 环境工具列表（如 python: 3.11, node: 18）
    pub envs: BTreeMap<String, serde_yaml::Value>,
    /// 镜像源配置（如 pip: tsinghua, npm: taobao）
    pub mirrors: Option<BTreeMap<String, String>>,
    /// 安装后执行的命令列表
    pub post_install: Option<Vec<String>>,
    /// 团队元信息（team.yaml 专有，oneinit.yaml 可缺省）
    pub team: Option<TeamMeta>,
    /// 环境变量（写入用户 profile，幂等）
    #[serde(default)]
    pub env_vars: BTreeMap<String, String>,
    /// 追加到 PATH 的条目（模板变量可渲染）
    #[serde(default)]
    pub path: Vec<String>,
    /// 配置文件模板（写入用户 home，带路径安全检查）
    #[serde(default)]
    pub config_files: Vec<TeamConfigFile>,
}

/// 团队元信息
#[derive(Debug, Clone, Deserialize)]
pub struct TeamMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    /// Ed25519 公钥 hex（配 team.yaml.sig 使用）
    pub signing_key: Option<String>,
}

/// 配置文件模板条目
#[derive(Debug, Clone, Deserialize)]
pub struct TeamConfigFile {
    /// 目标路径（支持 {{user_home}} 等模板变量）
    pub path: String,
    /// 文件内容模板（支持 {{mirror_npm}} 等变量）
    pub template: String,
}

/// 从 oneinit.yaml 文件解析配置
pub fn load_config(yaml_path: &Path) -> crate::core::Result<SyncConfig> {
    let content = std::fs::read_to_string(yaml_path)?;
    parse_config(&content)
}

/// 从字符串解析配置（团队环境直接使用远程内容，无需落盘）
pub fn parse_config(content: &str) -> crate::core::Result<SyncConfig> {
    let config: SyncConfig = serde_yaml::from_str(content)
        .map_err(|e| crate::core::CoreError::Other(format!("YAML parse failed: {}", e)))?;
    Ok(config)
}

/// 将 envs 中的键值对转换为recipe名
/// 例如: "python" + "3.11" → "python@3.11"（含点 → 动态/社区配方语义）
///       "node" + 18 → "node18"（不含点 → 内置语义）
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
            if ver_str.contains('.') {
                format!("{name}@{ver_str}") // 完整版本 → 动态/社区配方
            } else {
                format!("{name}{ver_str}") // 简短 major/minor → 内置
            }
        })
        .collect()
}

/// 从锁文件提取所有需要安装的包名（recipe 字段）
pub fn lockfile_to_package_names(lock: &Lockfile) -> Vec<String> {
    lock.tools.values().map(|t| t.recipe.clone()).collect()
}

/// execute post_install 命令列表
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
                &format!(
                    "   [ERROR] 命令执行失败（退出码 {:?}）",
                    output.status.code()
                ),
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
        // 完整 semver → name@version（动态/社区配方语义）
        envs.insert(
            "rg".to_string(),
            serde_yaml::Value::String("15.2.0".to_string()),
        );

        let config = SyncConfig {
            envs,
            ..Default::default()
        };

        let names = envs_to_recipe_names(&config);
        // 含点 → @version；不含点 → name+version
        assert!(names.contains(&"python@3.11".to_string()));
        assert!(names.contains(&"node18".to_string()));
        assert!(names.contains(&"rg@15.2.0".to_string()));
    }

    #[test]
    fn test_envs_to_recipe_names_empty() {
        let config = SyncConfig {
            envs: BTreeMap::new(),
            ..Default::default()
        };
        assert!(envs_to_recipe_names(&config).is_empty());
    }

    #[test]
    fn test_lockfile_to_package_names() {
        let mut tools = BTreeMap::new();
        tools.insert(
            "python".to_string(),
            crate::core::lockfile::LockedTool {
                recipe: "python3.11".to_string(),
                version: "3.11.9".to_string(),
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                source: "builtin".to_string(),
                archive_url: "https://example.com/python-3.11.9.tar.gz".to_string(),
            },
        );
        tools.insert(
            "rg".to_string(),
            crate::core::lockfile::LockedTool {
                recipe: "rg".to_string(),
                version: "15.2.0".to_string(),
                sha256: "d6a1f5c3d8b0e9a27e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934c"
                    .to_string(),
                source: "dynamic".to_string(),
                archive_url: "https://example.com/rg-15.2.0.zip".to_string(),
            },
        );
        let lock = crate::core::lockfile::Lockfile { version: 1, tools };

        let names = lockfile_to_package_names(&lock);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"python3.11".to_string()));
        assert!(names.contains(&"rg".to_string()));
    }

    #[test]
    fn test_lockfile_to_package_names_empty() {
        let lock = crate::core::lockfile::Lockfile {
            version: 1,
            tools: BTreeMap::new(),
        };
        assert!(lockfile_to_package_names(&lock).is_empty());
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
