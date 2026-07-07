use std::fs;
use std::path::{Path, PathBuf};

use super::Result;

/// 配置文件描述
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// 相对于安装目录的路径（如 "pip/pip.ini" 或 ".npmrc"）
    pub rel_path: String,
    /// 配置文件内容
    pub content: String,
}

/// 将配置文件写入安装目录
/// 返回实际写入的文件绝对路径列表
pub fn apply_configs(install_dir: &Path, configs: &[AppConfig]) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();

    for config in configs {
        let full_path = install_dir.join(&config.rel_path);

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, &config.content)?;
        written.push(full_path);
    }

    Ok(written)
}

/// 删除之前生成的配置文件
pub fn remove_configs(install_dir: &Path, configs: &[AppConfig]) -> Result<()> {
    for config in configs {
        let full_path = install_dir.join(&config.rel_path);
        if full_path.exists() {
            fs::remove_file(&full_path)?;
        }
    }
    Ok(())
}

// ============================================================
// 预置镜像源配置生成器
// ============================================================

/// pip 镜像源配置（自动根据平台选择格式）
pub fn pip_mirror_config() -> AppConfig {
    #[cfg(target_os = "windows")]
    {
        AppConfig {
            rel_path: "pip\\pip.ini".to_string(),
            content: "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple\ntrusted-host = pypi.tuna.tsinghua.edu.cn\n".to_string(),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        AppConfig {
            rel_path: "pip.conf".to_string(),
            content: "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple\ntrusted-host = pypi.tuna.tsinghua.edu.cn\n".to_string(),
        }
    }
}

/// npm 淘宝镜像配置
pub fn npm_mirror_config() -> AppConfig {
    AppConfig {
        rel_path: ".npmrc".to_string(),
        content: "registry=https://registry.npmmirror.com\n".to_string(),
    }
}

/// yarn 淘宝镜像配置
pub fn yarn_mirror_config() -> AppConfig {
    AppConfig {
        rel_path: ".yarnrc".to_string(),
        content: "registry \"https://registry.npmmirror.com\"\n".to_string(),
    }
}
