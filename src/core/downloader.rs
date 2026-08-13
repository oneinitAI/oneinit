use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256, Sha512};
use tokio::io::AsyncWriteExt;

use super::{CoreError, Result};

/// download结果
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub file_path: PathBuf,
    pub file_size: u64,
    pub sha256: String,
}

/// 网络下载重试次数（指数退避 1s/2s/4s）
const MAX_DOWNLOAD_RETRIES: u32 = 3;

/// 异步download文件到指定路径，带进度条
///
/// 网络错误 / HTTP 5xx 自动重试 [`MAX_DOWNLOAD_RETRIES`] 次（指数退避），
/// 4xx 与校验类错误不重试。
pub async fn download(url: &str, dest: &Path) -> Result<DownloadResult> {
    let mut last_err: Option<CoreError> = None;
    for attempt in 0..=MAX_DOWNLOAD_RETRIES {
        match download_attempt(url, dest).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let retryable = matches!(e, CoreError::Download(_)) || matches!(e, CoreError::Io(_));
                if attempt < MAX_DOWNLOAD_RETRIES && retryable {
                    let wait = std::time::Duration::from_secs(1 << attempt);
                    eprintln!(
                        "[WARN] 下载失败（{} 秒后第 {} 次重试）: {}",
                        wait.as_secs(),
                        attempt + 1,
                        e
                    );
                    tokio::time::sleep(wait).await;
                    last_err = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| {
        CoreError::Download(format!("download failed after {} retries: {url}", MAX_DOWNLOAD_RETRIES))
    }))
}

/// 单次下载尝试（不重试）
async fn download_attempt(url: &str, dest: &Path) -> Result<DownloadResult> {
    // 确保目标目录exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?;

    let total_size = response.content_length().unwrap_or(0);
    let file_name = dest
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} 下载 {msg} {bytes}/{total_bytes} {eta:>3}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_message(file_name);

    // 先清理可能存在的半下载文件（覆盖写）
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }

    let mut file = tokio::fs::File::create(dest).await.map_err(CoreError::Io)?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| CoreError::Download(e.to_string()))?;
        file.write_all(&chunk).await.map_err(CoreError::Io)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    file.flush().await.map_err(CoreError::Io)?;
    pb.finish_with_message("下载完成");

    let sha256 = compute_sha256(dest)?;
    let file_size = std::fs::metadata(dest)?.len();

    Ok(DownloadResult {
        file_path: dest.to_path_buf(),
        file_size,
        sha256,
    })
}

/// 计算文件的 SHA256 哈希
pub fn compute_sha256(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// 计算文件的 SHA512 哈希
pub fn compute_sha512(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// verify文件 SHA256 是否匹配
///
/// 按期望值长度自动选择算法：64 位 hex 用 SHA256，128 位 hex 用 SHA512
/// （.NET SDK 等官方发布使用 SHA512）。
pub fn verify_sha256(path: &Path, expected: &str) -> Result<bool> {
    let expected_trimmed = expected.trim();
    let actual = match expected_trimmed.len() {
        128 => compute_sha512(path)?,
        _ => compute_sha256(path)?,
    };
    let actual_lower = actual.to_lowercase();
    let expected_lower = expected_trimmed.to_lowercase();

    if actual_lower != expected_lower {
        return Err(CoreError::Checksum {
            file: path.display().to_string(),
            expected: expected.to_string(),
        });
    }

    Ok(true)
}

/// 解压归档文件到目标目录，自动识别格式
/// 返回解压出的文件列表
pub fn extract(archive: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dest)?;

    let file_name = archive
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if file_name.contains(".zip") {
        extract_zip(archive, dest)
    } else if file_name.contains(".tar.gz") || file_name.contains(".tgz") {
        extract_tar_gz(archive, dest)
    } else {
        Err(CoreError::Extract(format!(
            "不支持的归档格式: {}",
            archive.display()
        )))
    }
}

/// 解压 .zip 文件
fn extract_zip(archive: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let file = File::open(archive)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path = match entry.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut entry, &mut outfile)?;
            extracted.push(out_path);
        }
    }

    Ok(extracted)
}

/// 解压 .tar.gz 文件
fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let file = File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    let mut extracted = Vec::new();

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let path = entry.path()?.to_path_buf();
        let out_path = dest.join(&path);

        // 跳过绝对路径和路径遍历
        if out_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }

        if entry.header().entry_type().is_file() {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut entry, &mut outfile)?;
            extracted.push(out_path);
        } else if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out_path)?;
        }
    }

    Ok(extracted)
}
