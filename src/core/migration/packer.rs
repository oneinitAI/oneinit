// 导出打包器 — 环境快照 + 可选缓存 -> tar.gz
//
// 零新增依赖：flate2 + tar 已有，SHA256 复用 downloader::compute_sha256，
// 递归遍历手写 read_dir，临时目录用 core::temp_dir() + uuid。

use std::fs::File;
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;

use super::ExportResult;
use super::manifest::{CacheEntry, ManifestMetadata, MigrationManifest, PackageListEntry};
use crate::core::{CoreError, envs_dir};
use crate::core::{Result, capture, temp_dir};
use crate::output::OutputFormatter;

/// 执行导出
pub fn export(
    formatter: &OutputFormatter,
    output: &str,
    include_envs: bool,
) -> Result<ExportResult> {
    // 1. 扫描环境
    let mut scheduler = capture::detector::DetectorScheduler::new();
    scheduler.register_defaults();
    let scan_results = scheduler.scan();

    // 构建 EnvironmentSnapshot
    let mut envs = std::collections::BTreeMap::new();
    for (name, opt_env) in &scan_results {
        if let Some(env) = opt_env {
            envs.insert(name.clone(), env.clone());
        }
    }

    let snapshot = capture::EnvironmentSnapshot {
        metadata: capture::SnapshotMetadata {
            tool: "OneInit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            hostname: capture::hostname(),
            os: std::env::consts::OS.to_string(),
        },
        envs: envs.clone(),
        dotfiles: Vec::new(),
    };

    let env_count = snapshot.envs.len();
    formatter.output(
        &format!("[EXPORT] {} environments detected", env_count),
        Some(serde_json::Value::Null),
    );

    // 2. 创建临时工作目录
    let work_dir = temp_dir().join(format!("export-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work_dir)?;

    // 3. 写入 recipe/oneinit.yaml
    let recipe_dir = work_dir.join("recipe");
    std::fs::create_dir_all(&recipe_dir)?;
    let recipe_path = recipe_dir.join("oneinit.yaml");
    let yaml = serde_yaml::to_string(&snapshot)
        .map_err(|e| CoreError::Migration(format!("YAML serialize failed: {}", e)))?;
    std::fs::write(&recipe_path, &yaml)?;

    // 4. 可选：打包 envs/ 目录
    let mut cache_files = Vec::new();
    let mut total_size: u64 = 0;

    if include_envs {
        let envs_source = envs_dir();
        if envs_source.exists() {
            let cache_dest = work_dir.join("envs");
            std::fs::create_dir_all(&cache_dest)?;
            copy_dir_recursive(
                &envs_source,
                &cache_dest,
                &envs_source,
                &mut cache_files,
                &mut total_size,
            )?;
            formatter.output(
                &format!("[EXPORT] packing {} cache files", cache_files.len()),
                Some(serde_json::Value::Null),
            );
        }
    }

    // 5. 收集全局包列表
    let mut global_packages = Vec::new();
    if let Some(python_env) = snapshot.envs.get("python")
        && !python_env.global_packages.is_empty()
    {
        global_packages.push(PackageListEntry {
            manager: "pip".to_string(),
            packages: python_env.global_packages.clone(),
        });
    }
    if let Some(node_env) = snapshot.envs.get("node")
        && !node_env.global_packages.is_empty()
    {
        global_packages.push(PackageListEntry {
            manager: "npm".to_string(),
            packages: node_env.global_packages.clone(),
        });
    }

    // 6. 生成 manifest.json
    let manifest = MigrationManifest {
        metadata: ManifestMetadata {
            tool: "OneInit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: snapshot.metadata.timestamp,
            source_os: snapshot.metadata.os.clone(),
            source_hostname: snapshot.metadata.hostname.clone(),
            compression: "gzip".to_string(),
            total_size,
            env_count,
        },
        recipe: "recipe/oneinit.yaml".to_string(),
        cache_files,
        global_packages,
        checksums: std::collections::BTreeMap::new(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| CoreError::Migration(format!("JSON serialize failed: {}", e)))?;
    std::fs::write(work_dir.join("manifest.json"), manifest_json)?;

    // 7. 打包为 tar.gz
    let output_path = Path::new(output);
    create_archive(&work_dir, output_path)?;

    let archive_size = std::fs::metadata(output_path)?.len();
    let cache_count = manifest.cache_files.len();

    // 8. 清理临时目录
    let _ = std::fs::remove_dir_all(&work_dir);

    Ok(ExportResult {
        path: output.to_string(),
        total_size: archive_size,
        env_count,
        cache_count,
    })
}

/// 递归复制目录，收集文件信息和 SHA256
///
/// `src_root` 是最初的源根目录，用于计算文件相对路径。
fn copy_dir_recursive(
    src: &Path,
    dest: &Path,
    src_root: &Path,
    entries: &mut Vec<CacheEntry>,
    total_size: &mut u64,
) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&src_path, &dest_path, src_root, entries, total_size)?;
        } else if src_path.is_file() {
            // 复制文件
            std::fs::copy(&src_path, &dest_path)?;

            // 计算 SHA256
            let sha256 = crate::core::downloader::compute_sha256(&src_path)
                .unwrap_or_else(|_| "unknown".to_string());

            // 获取大小
            let size = std::fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);

            entries.push(CacheEntry {
                package: file_name.to_string_lossy().to_string(),
                filename: src_path
                    .strip_prefix(src_root)
                    .unwrap_or(&src_path)
                    .to_string_lossy()
                    .to_string(),
                size,
                sha256,
            });

            *total_size += size;
        }
    }

    Ok(())
}

/// 创建 tar.gz 归档
fn create_archive(source_dir: &Path, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let gz_encoder = GzEncoder::new(file, Compression::new(6));
    let mut tar_builder = Builder::new(gz_encoder);

    // 递归添加所有文件
    add_dir_to_tar(&mut tar_builder, source_dir, source_dir)?;

    tar_builder
        .finish()
        .map_err(|e| CoreError::Migration(format!("tar pack failed: {}", e)))?;

    Ok(())
}

/// 递归添加目录到 tar 归档
fn add_dir_to_tar(builder: &mut Builder<GzEncoder<File>>, dir: &Path, base: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(base)
            .map_err(|e| CoreError::Migration(format!("path resolve failed: {}", e)))?;

        if path.is_dir() {
            builder
                .append_dir_all(relative, &path)
                .map_err(|e| CoreError::Migration(format!("tar add dir failed: {}", e)))?;
        } else {
            builder
                .append_path_with_name(&path, relative)
                .map_err(|e| CoreError::Migration(format!("tar add file failed: {}", e)))?;
        }
    }

    Ok(())
}
