// 环境捕获模块 — 非侵入式扫描当前机器已安装的开发环境
//
// 按 数据的采集与迁移.md 第三章实现。
// 检测器为同步 trait（Command::output 是阻塞调用，无需 async）。

pub mod detector;
pub mod python;
pub mod node;
pub mod git;
pub mod rust;
pub mod go;
pub mod java;
pub mod docker;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{CoreError, Result};
use crate::output::OutputFormatter;

// ============================================================
// 核心数据结构
// ============================================================

/// 捕获到的完整环境快照
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentSnapshot {
    /// 元信息
    pub metadata: SnapshotMetadata,
    /// 检测到的运行时环境（name -> RuntimeEnv）
    pub envs: BTreeMap<String, RuntimeEnv>,
    /// 用户配置文件（dotfiles）
    #[serde(default)]
    pub dotfiles: Vec<DotfileEntry>,
}

/// 快照元信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotMetadata {
    pub tool: String,
    pub version: String,
    pub timestamp: u64,
    pub hostname: String,
    pub os: String,
}

/// 单个运行时环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnv {
    pub name: String,
    pub version: String,
    pub install_path: String,
    #[serde(default)]
    pub mirrors: BTreeMap<String, String>,
    #[serde(default)]
    pub global_packages: Vec<String>,
}

/// 配置文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfileEntry {
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

// ============================================================
// 命令入口
// ============================================================

/// 执行环境捕获，生成 oneinit.yaml
///
/// 流程：注册检测器 -> scan -> 构建 EnvironmentSnapshot -> 序列化 YAML -> 写入文件
pub fn run_capture(formatter: &OutputFormatter, output_path: &str) -> Result<()> {
    formatter.output(
        "[SCAN] 开始扫描开发环境...",
        Some(serde_json::json!({
            "status": "scanning",
            "action": "capture",
        })),
    );

    // 1. 注册检测器
    let mut scheduler = detector::DetectorScheduler::new();
    scheduler.register_defaults();

    // 2. 执行检测
    let results = scheduler.scan();

    // 3. 构建 EnvironmentSnapshot
    let mut envs = BTreeMap::new();
    for (name, opt_env) in &results {
        if let Some(env) = opt_env {
            formatter.output(
                &format!("  [OK] {} {} ({})", env.name, env.version, env.install_path),
                Some(serde_json::json!({
                    "detector": name,
                    "found": true,
                    "version": env.version,
                    "path": env.install_path,
                })),
            );

            // 显示镜像和包信息
            for (key, val) in &env.mirrors {
                formatter.output(
                    &format!("       {} -> {}", key, val),
                    Some(serde_json::Value::Null),
                );
            }
            if !env.global_packages.is_empty() {
                formatter.output(
                    &format!("       全局包: {} 个", env.global_packages.len()),
                    Some(serde_json::Value::Null),
                );
            }

            envs.insert(name.clone(), env.clone());
        } else {
            formatter.output(
                &format!("  [--] {} 未检测到", name),
                Some(serde_json::json!({
                    "detector": name,
                    "found": false,
                })),
            );
        }
    }

    // 4. 构建 metadata
    let snapshot = EnvironmentSnapshot {
        metadata: SnapshotMetadata {
            tool: "OneInit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            hostname: hostname(),
            os: std::env::consts::OS.to_string(),
        },
        envs,
        dotfiles: Vec::new(),
    };

    // 5. 序列化为 YAML
    let yaml = serde_yaml::to_string(&snapshot)
        .map_err(|e| CoreError::Capture(format!("YAML 序列化失败: {}", e)))?;

    // 6. 写入文件
    let path = Path::new(output_path);
    std::fs::write(path, &yaml)?;

    let detected_count = snapshot.envs.len();
    formatter.output(
        &format!("[OK] 环境快照已保存到 {} (检测到 {} 个环境)", output_path, detected_count),
        Some(serde_json::json!({
            "status": "success",
            "action": "capture",
            "output": output_path,
            "detected_count": detected_count,
            "detected": snapshot.envs.keys().collect::<Vec<_>>(),
        })),
    );

    Ok(())
}

/// 获取主机名
pub fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
