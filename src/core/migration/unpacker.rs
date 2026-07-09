// 导入解包器 — tar.gz -> 校验 -> 恢复配方/环境
//
// 复用 downloader::extract 解压 tar.gz，手写递归遍历恢复文件。

use std::path::{Path, PathBuf};

use super::manifest::MigrationManifest;
use super::ImportResult;
use crate::core::{recipes_dir, temp_dir, CoreError, Result};
use crate::output::OutputFormatter;

/// 执行导入
pub fn import(
    formatter: &OutputFormatter,
    archive: &str,
    dry_run: bool,
    force: bool,
    skip_checksum: bool,
) -> Result<ImportResult> {
    let archive_path = Path::new(archive);
    if !archive_path.exists() {
        return Err(CoreError::Migration(format!(
            "文件不存在: {}",
            archive
        )));
    }

    // 1. 解压到临时目录
    let extract_dir = temp_dir().join(format!("import-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&extract_dir)?;

    formatter.output(
        &format!("[IMPORT] 解压 {}...", archive),
        Some(serde_json::Value::Null),
    );

    crate::core::downloader::extract(archive_path, &extract_dir)?;

    // 2. 读取 manifest.json
    let manifest_path = extract_dir.join("manifest.json");
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CoreError::Migration(format!("读取 manifest.json 失败: {}", e)))?;
    let manifest: MigrationManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| CoreError::Migration(format!("解析 manifest.json 失败: {}", e)))?;

    formatter.output(
        &format!(
            "[IMPORT] 清单: {} 个环境, {} 个缓存, {} 个包列表 (源: {}/{})",
            manifest.metadata.env_count,
            manifest.cache_files.len(),
            manifest.global_packages.len(),
            manifest.metadata.source_os,
            manifest.metadata.source_hostname,
        ),
        Some(serde_json::Value::Null),
    );

    // 3. 可选：SHA256 校验缓存文件
    if !skip_checksum && !manifest.cache_files.is_empty() {
        let envs_dir = extract_dir.join("envs");
        if envs_dir.exists() {
            let mut verified = 0;
            let mut failed = 0;
            for entry in &manifest.cache_files {
                let file_path = envs_dir.join(&entry.filename);
                if file_path.exists() {
                    match crate::core::downloader::compute_sha256(&file_path) {
                        Ok(actual) => {
                            if actual == entry.sha256 {
                                verified += 1;
                            } else {
                                failed += 1;
                                formatter.output(
                                    &format!("[WARN] 校验失败: {}", entry.filename),
                                    Some(serde_json::Value::Null),
                                );
                            }
                        }
                        Err(_) => {
                            failed += 1;
                        }
                    }
                }
            }
            formatter.output(
                &format!("[IMPORT] SHA256 校验: {} 通过, {} 失败", verified, failed),
                Some(serde_json::Value::Null),
            );
            if failed > 0 && !force {
                return Err(CoreError::Migration(format!(
                    "{} 个文件校验失败，使用 --force 跳过",
                    failed
                )));
            }
        }
    }

    // 4. 恢复配方
    let recipe_src = extract_dir.join(&manifest.recipe);
    let recipe_dest = recipes_dir().join("imported.yaml");
    let recipe_path_str = recipe_dest.to_string_lossy().to_string();

    if !dry_run {
        if recipe_src.exists() {
            if recipe_dest.exists() && !force {
                formatter.output(
                    "[WARN] imported.yaml 已存在，使用 --force 覆盖",
                    Some(serde_json::Value::Null),
                );
            } else {
                std::fs::copy(&recipe_src, &recipe_dest)?;
                formatter.output(
                    &format!("[OK] 配方恢复: {}", recipe_dest.display()),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    // 5. 恢复 envs/ 目录
    let mut cache_restored = 0;
    let envs_src = extract_dir.join("envs");
    let envs_dest = crate::core::envs_dir();

    if envs_src.exists() {
        if dry_run {
            cache_restored = count_files_recursive(&envs_src);
            formatter.output(
                &format!("[PREVIEW] 将恢复 {} 个文件到 {}", cache_restored, envs_dest.display()),
                Some(serde_json::Value::Null),
            );
        } else {
            std::fs::create_dir_all(&envs_dest)?;
            cache_restored = copy_dir_recursive(&envs_src, &envs_dest, force)?;
            formatter.output(
                &format!("[OK] 恢复 {} 个文件到 envs/", cache_restored),
                Some(serde_json::Value::Null),
            );
        }
    }

    // 6. 统计全局包
    let packages_restored: usize = manifest
        .global_packages
        .iter()
        .map(|p| p.packages.len())
        .sum();

    if !dry_run && packages_restored > 0 {
        formatter.output(
            &format!(
                "[INFO] 包列表已记录（共 {} 个包，需手动安装或用 oneinit sync）",
                packages_restored
            ),
            Some(serde_json::Value::Null),
        );
    }

    // 7. 清理临时目录
    if !dry_run {
        let _ = std::fs::remove_dir_all(&extract_dir);
    }

    Ok(ImportResult {
        recipe_path: recipe_path_str,
        cache_restored,
        packages_restored,
        dry_run,
    })
}

/// 递归复制目录，返回复制的文件数
fn copy_dir_recursive(src: &Path, dest: &Path, force: bool) -> Result<usize> {
    let mut count = 0;
    if !src.is_dir() {
        return Ok(0);
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            count += copy_dir_recursive(&src_path, &dest_path, force)?;
        } else if src_path.is_file() {
            if dest_path.exists() && !force {
                // 跳过已存在的文件
            } else {
                std::fs::copy(&src_path, &dest_path)?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// 递归统计目录中的文件数
fn count_files_recursive(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }

    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files_recursive(&path);
            } else if path.is_file() {
                count += 1;
            }
        }
    }
    count
}
