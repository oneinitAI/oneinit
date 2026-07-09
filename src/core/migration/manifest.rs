// 迁移清单结构 — 写入 tar.gz 根目录的 manifest.json
//
// 记录导出包的元信息、包含的文件列表、SHA256 校验值。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 迁移清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationManifest {
    /// 元信息
    pub metadata: ManifestMetadata,
    /// oneinit.yaml 在包内的相对路径
    pub recipe: String,
    /// 缓存文件列表（可选，含 SHA256）
    #[serde(default)]
    pub cache_files: Vec<CacheEntry>,
    /// 全局包列表（按包管理器分组）
    #[serde(default)]
    pub global_packages: Vec<PackageListEntry>,
    /// 文件路径 -> SHA256 校验值
    #[serde(default)]
    pub checksums: BTreeMap<String, String>,
}

/// 清单元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    /// 工具名（"OneInit"）
    pub tool: String,
    /// 工具版本
    pub version: String,
    /// 创建时间戳（Unix 秒）
    pub created_at: u64,
    /// 源操作系统
    pub source_os: String,
    /// 源主机名
    pub source_hostname: String,
    /// 压缩算法
    pub compression: String,
    /// 包总大小（字节）
    pub total_size: u64,
    /// 检测到的环境数量
    pub env_count: usize,
}

/// 缓存文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 包名
    pub package: String,
    /// 文件名
    pub filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// SHA256 校验值
    pub sha256: String,
}

/// 全局包列表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageListEntry {
    /// 包管理器（"pip" / "npm"）
    pub manager: String,
    /// 包列表
    pub packages: Vec<String>,
}
