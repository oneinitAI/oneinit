//! 团队环境同步（team.yaml）
//!
//! 团队在 GitHub 建一个仓库描述共享开发环境（工具 + 镜像 + 环境变量 +
//! PATH + 配置文件 + post_install），成员 fork/配置后：
//! - `oneinit team add <url>` 配置并固定签名公钥（TOFU）
//! - 每次运行 oneinit 自动检测（`maybe_team_sync`），变化时逐个确认同步
//! - `oneinit team sync` 手动强制同步
//!
//! 安全模型：
//! - team.yaml 可选 Ed25519 签名（team.yaml.sig + team.signing_key），
//!   配置时固定公钥，之后每次同步强制验签，不匹配拒绝
//! - 未签名仓库可正常使用但给出 [WARN] 提示
//! - config_files / PATH 条目做路径安全检查（拒绝 `..` 越界）

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{CoreError, Result, data_dir, sync};

/// 团队环境检测间隔（小时）：每次运行 oneinit 时若距上次检查超过该值才联网检测
pub const CHECK_INTERVAL_HOURS: u64 = 24;

/// 镜像别名 → 实际 URL 解析（pip）
fn pip_mirror_url(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "tsinghua" => "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(),
        "aliyun" | "ali" => "https://mirrors.aliyun.com/pypi/simple/".to_string(),
        "ustc" => "https://mirrors.ustc.edu.cn/pypi/simple/".to_string(),
        // 其他：假定已是完整 URL
        _ => value.trim().to_string(),
    }
}

/// 镜像别名 → 实际 URL 解析（npm/yarn）
fn npm_mirror_url(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "npmmirror" | "taobao" => "https://registry.npmmirror.com".to_string(),
        _ => value.trim().to_string(),
    }
}

/// ~/.oneinit/team.json — 团队环境配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TeamConfig {
    /// raw base URL，如 https://raw.githubusercontent.com/<org>/<repo>/main
    pub team_url: String,
    /// 分支（展示用）
    pub branch: String,
    /// 团队名（来自 team.yaml 的 team.name）
    pub team_name: Option<String>,
    /// 上次检查时间（RFC3339）
    pub last_check: String,
    /// 上次成功同步时间（RFC3339）
    pub last_sync: String,
    /// 上次同步的 team.yaml 内容 SHA256（变化检测）
    pub cached_sha256: String,
    /// 固定（TOFU）的 Ed25519 公钥 hex，None = 未签名
    pub public_key: Option<String>,
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            team_url: String::new(),
            branch: "main".to_string(),
            team_name: None,
            last_check: String::new(),
            last_sync: String::new(),
            cached_sha256: String::new(),
            public_key: None,
        }
    }
}

/// ~/.oneinit/team.json
pub fn config_path() -> PathBuf {
    data_dir().join("team.json")
}

/// 读取团队配置（缺失/损坏时回退默认）
pub fn load_config() -> TeamConfig {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 保存团队配置
pub fn save_config(cfg: &TeamConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| CoreError::Other(format!("team.json serialize failed: {e}")))?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// 是否已配置团队环境
pub fn is_configured() -> bool {
    !load_config().team_url.is_empty()
}

/// 距上次检查是否超过间隔（或从未检查）——每次运行时的轻量判定
pub fn needs_check(cfg: &TeamConfig) -> bool {
    if cfg.last_check.is_empty() {
        return true;
    }
    match chrono::DateTime::parse_from_rfc3339(&cfg.last_check) {
        Ok(t) => {
            let elapsed = chrono::Utc::now().signed_duration_since(t.with_timezone(&chrono::Utc));
            elapsed.num_hours() >= CHECK_INTERVAL_HOURS as i64
        }
        Err(_) => true,
    }
}

/// 规范化仓库 URL → raw base URL
///
/// 支持：
/// - https://github.com/<owner>/<repo>
/// - https://github.com/<owner>/<repo>.git
/// - https://github.com/<owner>/<repo>/tree/<branch>
/// - https://raw.githubusercontent.com/<owner>/<repo>/<branch>（原样）
pub fn normalize_url(url: &str, branch: &str) -> Result<String> {
    let url = url.trim().trim_end_matches('/');
    if let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
    {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(CoreError::Other(format!(
                "无效的 GitHub 仓库地址: {url}（应为 https://github.com/<owner>/<repo>）"
            )));
        }
        let repo = parts[1].trim_end_matches(".git");
        // /tree/<branch> 形式
        if parts.len() >= 4 && parts[2] == "tree" {
            return Ok(format!(
                "https://raw.githubusercontent.com/{}/{}/{}",
                parts[0], repo, parts[3]
            ));
        }
        return Ok(format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            parts[0], repo, branch
        ));
    }
    if url.starts_with("https://raw.githubusercontent.com/")
        || url.starts_with("http://raw.githubusercontent.com/")
    {
        return Ok(url.to_string());
    }
    if url.starts_with("https://") || url.starts_with("http://") {
        return Ok(url.to_string());
    }
    Err(CoreError::Other(format!(
        "不支持的 URL: {url}（请使用 https://github.com/<owner>/<repo> 或 raw 地址）"
    )))
}

/// sha256 hex（变化检测 + 缓存指纹）
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// 构建 HTTP 客户端（禁用重定向，同注册表安全策略 M-2）
pub fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::Registry(format!("build http client failed: {e}")))
}

/// 拉取 raw 文件（如 team.yaml / team.yaml.sig）
pub async fn fetch_raw(client: &reqwest::Client, base: &str, file: &str) -> Result<String> {
    let url = format!("{}/{}", base.trim_end_matches('/'), file);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::Registry(format!("GET {url} failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(CoreError::Registry(format!(
            "GET {url} -> HTTP {}",
            resp.status()
        )));
    }
    resp.text()
        .await
        .map_err(|e| CoreError::Registry(format!("read {url} failed: {e}")))
}

/// 拉取 team.yaml（+ 可选 team.yaml.sig）
pub async fn fetch_team_file(
    client: &reqwest::Client,
    base: &str,
) -> Result<(String, Option<String>)> {
    let content = fetch_raw(client, base, "team.yaml").await?;
    let sig = fetch_raw(client, base, "team.yaml.sig").await.ok();
    Ok((content, sig))
}

/// 提取 team.yaml 中声明的签名公钥
fn declared_key(content: &str) -> Option<String> {
    let value: serde_yaml::Value = serde_yaml::from_str(content).ok()?;
    value
        .get("team")?
        .get("signing_key")?
        .as_str()
        .map(|s| s.trim().to_string())
}

/// 验签策略：
/// - expected_key = Some(固定公钥)：强制验签，且声明的公钥必须一致（TOFU 后续同步）
/// - expected_key = None（首次 add）：信任仓库自声明的公钥
///
/// 规则：
/// - 声明了 signing_key 但缺 team.yaml.sig → 拒绝（无法建立信任）
/// - 固定了公钥但缺 .sig → 拒绝
/// - .sig 存在但未声明公钥 → 拒绝
/// - 签名不匹配 → 拒绝
/// - 完全未签名（无声明、无 .sig）→ 通过（[WARN] 由调用方提示）
pub fn verify_content_signature(
    content: &str,
    sig: &Option<String>,
    expected_key: Option<&str>,
) -> Result<()> {
    let declared = declared_key(content);

    match sig {
        None => {
            if declared.is_some() {
                return Err(CoreError::Other(
                    "team.yaml 声明了 signing_key 但仓库缺少 team.yaml.sig — 无法建立信任".into(),
                ));
            }
            if expected_key.is_some() {
                return Err(CoreError::Other(
                    "已固定签名公钥但仓库缺少 team.yaml.sig — 拒绝同步".into(),
                ));
            }
            Ok(())
        }
        Some(sig_text) => {
            let Some(pub_hex) = declared.or_else(|| expected_key.map(String::from)) else {
                return Err(CoreError::Other(
                    "team.yaml.sig 存在但 team.yaml 未声明 signing_key，无法验证".into(),
                ));
            };
            if let Some(exp) = expected_key
                && pub_hex != exp
            {
                return Err(CoreError::Other(
                    "team.yaml 声明的签名公钥与本地固定公钥不一致 — 内容可能被篡改，拒绝同步"
                        .into(),
                ));
            }
            if super::registry::verify_signature(content.as_bytes(), sig_text.trim(), &pub_hex) {
                Ok(())
            } else {
                Err(CoreError::Other(
                    "team.yaml 签名验证失败 — 内容可能被篡改，拒绝同步".into(),
                ))
            }
        }
    }
}

/// 配置团队环境（team add）
///
/// - 拉取 team.yaml + .sig，验签后固定公钥（TOFU）
/// - 记录内容哈希，供后续变化检测
pub async fn add_team(
    formatter: &crate::output::OutputFormatter,
    url: &str,
    branch: &str,
    force: bool,
) -> Result<()> {
    let base = normalize_url(url, branch)?;
    let existing = load_config();
    if !existing.team_url.is_empty() && existing.team_url != base && !force {
        return Err(CoreError::Other(format!(
            "已配置团队环境 {} — 覆盖请使用 --force",
            existing.team_url
        )));
    }

    let client = http_client()?;
    formatter.output(
        &format!("[TEAM] 拉取 {} ...", base),
        Some(serde_json::Value::Null),
    );
    let (content, sig) = fetch_team_file(&client, &base).await?;

    // 校验最小结构（能解析即可，完整字段在同步时使用）
    let config: sync::SyncConfig = sync::parse_config(&content)?;
    // 首次配置：信任仓库自声明公钥并固定
    verify_content_signature(&content, &sig, None)?;

    let mut cfg = existing;
    cfg.team_url = base;
    cfg.branch = branch.to_string();
    cfg.team_name = config.team.as_ref().and_then(|t| t.name.clone());
    cfg.public_key = declared_key(&content);
    cfg.cached_sha256 = sha256_hex(content.as_bytes());
    cfg.last_check = chrono::Utc::now().to_rfc3339();
    save_config(&cfg)?;

    formatter.output(
        &format!(
            "[OK] 团队环境已配置: {}",
            cfg.team_name.as_deref().unwrap_or("(未命名)")
        ),
        Some(serde_json::Value::Null),
    );
    if sig.is_some() {
        formatter.output(
            "[OK] team.yaml 签名已验证并固定公钥",
            Some(serde_json::Value::Null),
        );
    } else {
        formatter.output(
            "[WARN] 该仓库未签名 — 建议团队配置 Ed25519 签名（见 oneinit-team-env 模板）",
            Some(serde_json::Value::Null),
        );
    }
    Ok(())
}

/// 移除团队环境配置
pub fn remove_team() -> Result<bool> {
    let path = config_path();
    if path.exists() {
        std::fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 输出团队环境状态
pub fn status(formatter: &crate::output::OutputFormatter) {
    let cfg = load_config();
    if cfg.team_url.is_empty() {
        formatter.output(
            "[TEAM] 未配置团队环境 — 使用 oneinit team add <url>",
            Some(serde_json::json!({
                "action": "team_status",
                "status": "not_configured",
                "configured": false,
                "message": "Use `oneinit team add <url>` to configure",
            })),
        );
        return;
    }
    formatter.begin_document("team_status");
    formatter.output(
        &format!("[TEAM] URL: {}", cfg.team_url),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!(
            "[TEAM] 团队: {}",
            cfg.team_name.as_deref().unwrap_or("(未命名)")
        ),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!(
            "[TEAM] 签名: {}",
            if cfg.public_key.is_some() {
                "已固定 Ed25519 公钥"
            } else {
                "未签名"
            }
        ),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!(
            "[TEAM] 上次检查: {}",
            if cfg.last_check.is_empty() {
                "从未".to_string()
            } else {
                cfg.last_check.clone()
            }
        ),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!(
            "[TEAM] 上次同步: {}",
            if cfg.last_sync.is_empty() {
                "从未".to_string()
            } else {
                cfg.last_sync.clone()
            }
        ),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        "[TEAM] 团队环境已配置",
        Some(serde_json::json!({
            "action": "team_status",
            "status": "configured",
            "configured": true,
            "team_url": cfg.team_url,
            "team_name": cfg.team_name.clone(),
            "signed": cfg.public_key.is_some(),
            "last_check": cfg.last_check,
            "last_sync": cfg.last_sync,
        })),
    );
    formatter.end_document();
}

/// 检测团队环境是否有变化；有变化时返回新 team.yaml 内容
///
/// - 未配置 / 未到检查间隔 → Ok(None)
/// - 拉取 + 验签（固定公钥）→ 失败返回 Err
/// - 内容哈希与缓存一致 → 更新 last_check，Ok(None)
/// - 内容变化 → 返回 Ok(Some(content))（由调用方同步，成功后更新哈希）
pub async fn fetch_if_changed(
    formatter: &crate::output::OutputFormatter,
    force: bool,
) -> Result<Option<String>> {
    let cfg = load_config();
    if cfg.team_url.is_empty() {
        return Ok(None);
    }
    if !force && !needs_check(&cfg) {
        return Ok(None);
    }

    let client = http_client()?;
    formatter.output(
        &format!("[TEAM] 检查 {} ...", cfg.team_url),
        Some(serde_json::Value::Null),
    );
    let (content, sig) = fetch_team_file(&client, &cfg.team_url).await?;

    // 验签：已固定公钥则强制验证
    if let Some(key) = cfg.public_key.as_deref() {
        verify_content_signature(&content, &sig, Some(key))?;
    }

    let hash = sha256_hex(content.as_bytes());

    // 无论是否变化都更新 last_check
    let mut new_cfg = cfg.clone();
    new_cfg.last_check = chrono::Utc::now().to_rfc3339();
    save_config(&new_cfg)?;

    if hash == cfg.cached_sha256 && !force {
        formatter.output("[TEAM] 团队环境无变化", Some(serde_json::Value::Null));
        return Ok(None);
    }
    Ok(Some(content))
}

// ============================================================
// 同步执行：镜像 / 环境变量 / PATH / 配置文件
// （工具的安装由 cli 层走 3 层配方解析，见 cli::apply_team_env）
// ============================================================

/// 应用镜像源配置到用户级配置文件（幂等，带 OneInit 标记）
pub fn apply_mirrors(
    mirrors: &BTreeMap<String, String>,
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    for (key, value) in mirrors {
        match key.as_str() {
            "pip" => apply_pip_mirror(&pip_mirror_url(value), formatter)?,
            "npm" => apply_npm_mirror(&npm_mirror_url(value), formatter)?,
            "yarn" => apply_yarn_mirror(&npm_mirror_url(value), formatter)?,
            other => formatter.output(
                &format!("[WARN] 未知镜像类型: {}（支持 pip/npm/yarn）", other),
                Some(serde_json::Value::Null),
            ),
        }
    }
    Ok(())
}

/// 写入 pip 镜像配置（Windows: %APPDATA%\pip\pip.ini；其他: ~/.pip/pip.conf）
fn apply_pip_mirror(url: &str, formatter: &crate::output::OutputFormatter) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| CoreError::Other("无法确定用户目录".into()))?;
    let (dir, file) = if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| home.join("AppData/Roaming").to_string_lossy().to_string());
        (PathBuf::from(appdata).join("pip"), "pip.ini".to_string())
    } else {
        (home.join(".pip"), "pip.conf".to_string())
    };
    std::fs::create_dir_all(&dir)?;
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("pypi.tuna.tsinghua.edu.cn");
    let content = format!(
        "# Managed by OneInit team sync\n[global]\nindex-url = {url}\ntrusted-host = {host}\n"
    );
    let target = dir.join(&file);
    std::fs::write(&target, content)?;
    formatter.output(
        &format!("[TEAM] pip 镜像 -> {}", target.display()),
        Some(serde_json::Value::Null),
    );
    Ok(())
}

/// 写入 npm 镜像配置（追加到 ~/.npmrc，去重）
fn apply_npm_mirror(url: &str, formatter: &crate::output::OutputFormatter) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| CoreError::Other("无法确定用户目录".into()))?;
    let rc = home.join(".npmrc");
    let line = format!("registry={url}\n");
    let existing = std::fs::read_to_string(&rc).unwrap_or_default();
    if existing.contains(line.trim_end_matches('\n')) {
        return Ok(());
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc)?;
    writeln!(f, "{line}# ^ added by OneInit team sync")?;
    formatter.output(
        &format!("[TEAM] npm 镜像 -> {}", rc.display()),
        Some(serde_json::Value::Null),
    );
    Ok(())
}

/// 写入 yarn 镜像配置（追加到 ~/.yarnrc，去重）
fn apply_yarn_mirror(url: &str, formatter: &crate::output::OutputFormatter) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| CoreError::Other("无法确定用户目录".into()))?;
    let rc = home.join(".yarnrc");
    let line = format!("registry \"{url}\"\n");
    let existing = std::fs::read_to_string(&rc).unwrap_or_default();
    if existing.contains(line.trim_end_matches('\n')) {
        return Ok(());
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc)?;
    writeln!(f, "{line}# ^ added by OneInit team sync")?;
    formatter.output(
        &format!("[TEAM] yarn 镜像 -> {}", rc.display()),
        Some(serde_json::Value::Null),
    );
    Ok(())
}

/// 应用环境变量到用户 profile（Unix：带 marker 追加；Windows：setx）
pub fn apply_env_vars(
    vars: &BTreeMap<String, String>,
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    for (key, value) in vars {
        #[cfg(target_os = "windows")]
        {
            let out = std::process::Command::new("setx")
                .args([key, value])
                .output()?;
            formatter.output(
                &format!("[TEAM] 环境变量 {} setx -> {:?}", key, out.status.code()),
                Some(serde_json::Value::Null),
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            append_env_to_profiles(key, value, formatter)?;
        }
    }
    Ok(())
}

/// Unix：把 export KEY="value" 追加到存在的 shell profile（marker 去重）
#[cfg(not(target_os = "windows"))]
fn append_env_to_profiles(
    key: &str,
    value: &str,
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    use std::io::Write;
    let home = dirs::home_dir().ok_or_else(|| CoreError::Other("无法确定用户目录".into()))?;
    let marker = "# OneInit team env";
    let rc_files = [
        home.join(".bashrc"),
        home.join(".zshrc"),
        home.join(".config/fish/config.fish"),
    ];
    let mut written_any = false;
    for rc in rc_files {
        if !rc.exists() {
            continue;
        }
        let existing = std::fs::read_to_string(&rc).unwrap_or_default();
        if existing.contains(marker) && existing.contains(&format!("{key}=")) {
            continue; // 已由 OneInit 管理过该变量
        }
        let mut f = std::fs::OpenOptions::new().append(true).open(&rc)?;
        write!(f, "\n{marker}\nexport {key}=\"{value}\"\n")?;
        formatter.output(
            &format!("[TEAM] 环境变量 {key} -> {}", rc.display()),
            Some(serde_json::Value::Null),
        );
        written_any = true;
    }
    if !written_any {
        // 没有可用的 profile 文件，退回写入 ~/.bashrc
        let rc = home.join(".bashrc");
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc)?;
        write!(f, "\n{marker}\nexport {key}=\"{value}\"\n")?;
        formatter.output(
            &format!("[TEAM] 环境变量 {key} -> {}", rc.display()),
            Some(serde_json::Value::Null),
        );
    }
    Ok(())
}

/// 应用 PATH 条目（模板渲染 + 去重 + 拒绝 `..`）
pub fn apply_path_entries(
    entries: &[String],
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    for entry in entries {
        let rendered = super::community_recipe::render_template(entry, &super::envs_dir());
        let p = std::path::Path::new(&rendered);
        if p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            formatter.output(
                &format!("[WARN] 跳过含 `..` 的 PATH 条目: {rendered}"),
                Some(serde_json::Value::Null),
            );
            continue;
        }
        super::path_mgr::add(p)?;
        formatter.output(
            &format!("[TEAM] PATH 添加 {rendered}"),
            Some(serde_json::Value::Null),
        );
    }
    Ok(())
}

/// 应用配置文件模板（预览 + 确认 + 路径安全检查：绝对路径且在 home 下，拒绝 `..`）
pub fn apply_config_files(
    files: &[sync::TeamConfigFile],
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }
    let envs_dir = super::envs_dir();
    let home = dirs::home_dir().ok_or_else(|| CoreError::Other("无法确定用户目录".into()))?;

    // 预览
    formatter.output("[TEAM] 将写入以下配置文件:", Some(serde_json::Value::Null));
    for f in files {
        let rendered_path = super::community_recipe::render_template(&f.path, &envs_dir);
        formatter.output(
            &format!("  - {rendered_path}"),
            Some(serde_json::Value::Null),
        );
    }

    // 确认（与工具安装一致的交互安全模型）
    use std::io::Write;
    print!("[SECURITY] 确认写入以上配置文件? (y/N): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        formatter.output("[CANCEL] 已取消配置文件写入", Some(serde_json::Value::Null));
        return Ok(());
    }

    for f in files {
        let rendered_path = super::community_recipe::render_template(&f.path, &envs_dir);
        let rendered_content = super::community_recipe::render_template(&f.template, &envs_dir);

        // 安全检查：绝对路径、拒绝 `..`、必须在 home 下
        let p = std::path::Path::new(&rendered_path);
        let has_parent = p
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        let under_home = p.strip_prefix(&home).map(|_| true).unwrap_or(false);
        if has_parent || !p.is_absolute() || !under_home {
            return Err(CoreError::Other(format!(
                "非法配置文件路径（需为 home 下绝对路径且不含 `..`）: {rendered_path}"
            )));
        }

        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(p, rendered_content)?;
        formatter.output(
            &format!("[TEAM] wrote {rendered_path}"),
            Some(serde_json::Value::Null),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url_github_repo() {
        assert_eq!(
            normalize_url("https://github.com/acme/team-env", "main").unwrap(),
            "https://raw.githubusercontent.com/acme/team-env/main"
        );
        assert_eq!(
            normalize_url("https://github.com/acme/team-env.git", "main").unwrap(),
            "https://raw.githubusercontent.com/acme/team-env/main"
        );
        assert_eq!(
            normalize_url("https://github.com/acme/team-env/tree/dev", "main").unwrap(),
            "https://raw.githubusercontent.com/acme/team-env/dev"
        );
    }

    #[test]
    fn test_normalize_url_raw_and_invalid() {
        assert_eq!(
            normalize_url(
                "https://raw.githubusercontent.com/acme/team-env/main",
                "main"
            )
            .unwrap(),
            "https://raw.githubusercontent.com/acme/team-env/main"
        );
        assert!(normalize_url("ftp://bad", "main").is_err());
        assert!(normalize_url("https://github.com/onlyowner", "main").is_err());
    }

    #[test]
    fn test_team_config_roundtrip() {
        let cfg = TeamConfig {
            team_url: "https://raw.githubusercontent.com/acme/team-env/main".to_string(),
            team_name: Some("Acme".to_string()),
            cached_sha256: "abc".to_string(),
            public_key: Some("4d3b".to_string()),
            ..Default::default()
        };
        // 直接测试序列化往返
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let back: TeamConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.team_url, cfg.team_url);
        assert_eq!(back.team_name, cfg.team_name);
        assert_eq!(back.public_key, cfg.public_key);
    }

    #[test]
    fn test_sha256_hex_known() {
        // sha256("abc")
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_verify_content_signature_valid_and_tampered() {
        use ed25519_dalek::{Signer, SigningKey};

        let seed = [7u8; 32];
        let signing = SigningKey::from_bytes(&seed);
        let pub_hex: String = signing
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let content = format!(
            "team:\n  name: Acme\n  signing_key: \"{}\"\nenvs:\n  node: \"20\"\n",
            pub_hex
        );
        let sig: String = signing
            .sign(content.as_bytes())
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        // 正路径：声明公钥 + 有效签名 → 通过
        assert!(verify_content_signature(&content, &Some(sig.clone()), None).is_ok());
        // 固定公钥一致 + 有效签名 → 通过
        assert!(verify_content_signature(&content, &Some(sig.clone()), Some(&pub_hex)).is_ok());
        // 篡改内容 → 拒绝
        assert!(
            verify_content_signature(
                &content.replace("node", "python"),
                &Some(sig.clone()),
                Some(&pub_hex)
            )
            .is_err()
        );
        // 公钥不一致 → 拒绝
        let other_pub = "11".repeat(32);
        assert!(verify_content_signature(&content, &Some(sig.clone()), Some(&other_pub)).is_err());
        // 声明公钥但无 .sig → 拒绝
        assert!(verify_content_signature(&content, &None, None).is_err());
        // 无签名（无声明、无 .sig）→ 通过
        let unsigned = "team:\n  name: Acme\nenvs:\n  node: \"20\"\n";
        assert!(verify_content_signature(unsigned, &None, None).is_ok());
        // 已固定公钥但无 .sig → 拒绝
        assert!(verify_content_signature(unsigned, &None, Some(&pub_hex)).is_err());
    }

    #[test]
    fn test_parse_full_team_yaml() {
        let yaml = r#"
team:
  name: Acme
  description: Team env
  version: "1"
envs:
  node: "20"
  python: "3.11"
mirrors:
  pip: tsinghua
  npm: npmmirror
env_vars:
  NODE_ENV: development
path:
  - "{{user_home}}/acme/bin"
config_files:
  - path: "{{user_home}}/.npmrc"
    template: "registry={{mirror_npm}}\n"
post_install:
  - "echo hi"
"#;
        let cfg = sync::parse_config(yaml).unwrap();
        assert_eq!(cfg.envs.len(), 2);
        assert_eq!(cfg.env_vars.get("NODE_ENV").unwrap(), "development");
        assert_eq!(cfg.path.len(), 1);
        assert_eq!(cfg.config_files.len(), 1);
        assert_eq!(cfg.team.as_ref().unwrap().name.as_deref(), Some("Acme"));
        assert!(cfg.mirrors.is_some());
        assert!(cfg.post_install.is_some());
    }

    #[test]
    fn test_parse_legacy_oneinit_yaml_compat() {
        // 旧格式（只有 envs）仍可解析
        let yaml = "envs:\n  python: \"3.11\"\n";
        let cfg = sync::parse_config(yaml).unwrap();
        assert_eq!(cfg.envs.len(), 1);
        assert!(cfg.env_vars.is_empty());
        assert!(cfg.path.is_empty());
        assert!(cfg.config_files.is_empty());
        assert!(cfg.team.is_none());
    }
}
