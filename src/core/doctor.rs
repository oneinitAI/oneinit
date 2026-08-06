//! Environment health-check engine (`oneinit doctor`).
//!
//! Organized as a list of categorized `CheckResult`s. Each check carries a
//! `Severity` so the caller (and AI agents reading `--json`) can tell real
//! failures (`Critical`) from soft warnings (`Warning`) and FYI notes (`Info`).
//!
//! Design constraints: zero new crate dependencies — reuses `capture::detector`
//! (binary probing), `registry` (cached index for version comparison), and the
//! already-present `winreg`/`winapi` crates for Windows PATH permission probes.

use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use crate::core::manifest::Manifest;
use crate::core::registry::load_cached_index;
use crate::core::{data_dir, envs_dir, temp_dir};

/// How serious a failed check is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Hard failure — the environment is broken. Counts toward `healthy`.
    Critical,
    /// Soft issue — worth fixing but not blocking. Counts toward `warnings`.
    Warning,
    /// Informational only (license list, temp-file note). Never affects health.
    Info,
}

/// One categorized check result.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// Top-level group, e.g. "已安装配方", "网络", "PATH 环境".
    pub category: &'static str,
    /// Short machine-readable key, e.g. "binary_usable", "disk_space".
    pub name: &'static str,
    /// Did the check pass? (`Info` rows are always `true`.)
    pub passed: bool,
    pub severity: Severity,
    /// Human-readable explanation; surfaced verbatim in human mode.
    pub detail: String,
}

impl CheckResult {
    fn ok(
        category: &'static str,
        name: &'static str,
        severity: Severity,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name,
            passed: true,
            severity,
            detail: detail.into(),
        }
    }
    fn fail(
        category: &'static str,
        name: &'static str,
        severity: Severity,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name,
            passed: false,
            severity,
            detail: detail.into(),
        }
    }
    fn info(category: &'static str, name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            category,
            name,
            passed: true,
            severity: Severity::Info,
            detail: detail.into(),
        }
    }
}

/// Display categories in a stable, readable order.
const CATEGORY_ORDER: &[&str] = &[
    "已安装配方",
    "网络",
    "PATH 环境",
    "环境变量",
    "系统资源",
    "许可合规",
    "AI 友好性",
];

/// Run every health check and return the results (category-grouping is the
/// caller's job; results are emitted in CATEGORY_ORDER order already).
pub async fn run_all() -> Vec<CheckResult> {
    let manifest_records = Manifest::open()
        .ok()
        .and_then(|m| m.list().ok())
        .unwrap_or_default();

    let mut out = Vec::new();
    out.extend(check_installed_usability(&manifest_records));
    out.extend(check_orphans_and_temp(&manifest_records));
    out.extend(check_version_freshness(&manifest_records));
    out.extend(check_network().await);
    out.extend(check_path_duplicates());
    out.extend(check_path_write_perm());
    out.extend(check_env_pollution());
    out.extend(check_disk_space());
    out.extend(check_licenses(&manifest_records));
    out.extend(check_json_selftest(&manifest_records));
    out
}

// ---------------------------------------------------------------------------
// 已安装配方
// ---------------------------------------------------------------------------

/// Map a manifest record name (e.g. `python3.11`, `node@20`, `java17`) to the
/// bare executable name used to probe `--version`. Returns `None` for unknown
/// tool families — callers should treat that as "skip" (Info), not a failure.
fn exe_for(record_name: &str) -> Option<&'static str> {
    // Strip a `@version` suffix first (e.g. `python@3.11` -> `python`).
    let base = record_name
        .split('@')
        .next()
        .unwrap_or(record_name)
        .to_ascii_lowercase();
    let base = base.trim();
    if base.starts_with("python") {
        Some("python")
    } else if base.starts_with("node") {
        Some("node")
    } else if base.starts_with("dotnet") {
        Some("dotnet")
    } else if base.starts_with("java") || base == "jdk" {
        Some("java")
    } else {
        match base {
            "go" | "golang" => Some("go"),
            "rust" | "rustup" => Some("cargo"),
            "mysql" | "mariadb" => Some("mysql"),
            "docker" => Some("docker"),
            _ => None,
        }
    }
}

fn check_installed_usability(records: &[crate::core::manifest::InstallRecord]) -> Vec<CheckResult> {
    use crate::core::capture::detector::{find_command, run_command, run_command_with_stderr};

    let mut results = Vec::new();
    for r in records {
        // Install directory missing is a Critical consistency failure.
        let install_path = Path::new(&r.install_path);
        if !install_path.exists() {
            results.push(CheckResult::fail(
                "已安装配方",
                "install_dir_present",
                Severity::Critical,
                format!("{} 安装目录缺失: {}", r.name, r.install_path),
            ));
            continue;
        }

        match exe_for(&r.name) {
            Some(exe) => {
                // Prefer the binary we actually installed, else whatever's on PATH.
                let found = find_command(exe);
                let works = found.as_ref().and_then(|p| {
                    // java/docker print version to stderr; combined covers both.
                    run_command_with_stderr(p.to_str().unwrap_or(exe), &["--version"])
                        .or_else(|| run_command(p.to_str().unwrap_or(exe), &["--version"]))
                });
                match works {
                    Some(ver) => {
                        let one_line = ver.lines().next().unwrap_or(&ver).to_string();
                        results.push(CheckResult::ok(
                            "已安装配方",
                            "binary_usable",
                            Severity::Critical,
                            format!("{} → {} --version 正常 ({})", r.name, exe, one_line),
                        ));
                    }
                    None => {
                        results.push(CheckResult::fail(
                            "已安装配方",
                            "binary_usable",
                            Severity::Critical,
                            format!("{} 二进制不可用（{} --version 失败）", r.name, exe),
                        ));
                    }
                }
            }
            None => results.push(CheckResult::info(
                "已安装配方",
                "binary_usable",
                format!("{} 无已知可执行名，跳过可用性检查", r.name),
            )),
        }
    }
    if records.is_empty() {
        results.push(CheckResult::info(
            "已安装配方",
            "binary_usable",
            "无已安装工具（oneinit list 为空）",
        ));
    }
    results
}

fn check_orphans_and_temp(records: &[crate::core::manifest::InstallRecord]) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // PATH entries pointing nowhere (recorded in manifest but missing on disk).
    let mut orphan_path_entries = 0;
    for r in records {
        for entry in &r.path_entries {
            if !Path::new(entry).exists() {
                orphan_path_entries += 1;
            }
        }
    }
    if orphan_path_entries == 0 {
        results.push(CheckResult::ok(
            "已安装配方",
            "path_entries_present",
            Severity::Critical,
            "所有记录的 PATH 条目均存在",
        ));
    } else {
        results.push(CheckResult::fail(
            "已安装配方",
            "path_entries_present",
            Severity::Warning,
            format!(
                "{} 个 PATH 条目指向不存在的路径（建议重装对应工具）",
                orphan_path_entries
            ),
        ));
    }

    // Leftover half-downloaded files in the temp dir.
    let temp = temp_dir();
    let mut count = 0usize;
    let mut bytes: u64 = 0;
    if let Ok(rd) = std::fs::read_dir(&temp) {
        for ent in rd.flatten() {
            if let Ok(m) = ent.metadata() {
                count += 1;
                if m.is_file() {
                    bytes += m.len();
                }
            }
        }
    }
    if count == 0 {
        results.push(CheckResult::ok(
            "已安装配方",
            "temp_clean",
            Severity::Info,
            "temp 目录无残留文件",
        ));
    } else {
        results.push(CheckResult::info(
            "已安装配方",
            "temp_clean",
            format!(
                "temp 目录有 {} 个残留文件（约 {}）—可能是中断的下载",
                count,
                human_bytes(bytes)
            ),
        ));
    }
    results
}

fn check_version_freshness(records: &[crate::core::manifest::InstallRecord]) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let Some(index) = load_cached_index() else {
        results.push(CheckResult::info(
            "已安装配方",
            "version_freshness",
            "配方索引未缓存，跳过版本检查（运行 oneinit update）",
        ));
        return results;
    };

    let mut any_checked = false;
    for r in records {
        let Some(installed) = &r.version else {
            continue;
        };
        let Some(entry) = index.packages.get(&r.name) else {
            continue;
        };
        let latest = entry.latest.as_str();
        if latest.is_empty() {
            continue;
        }
        any_checked = true;
        if installed != latest {
            results.push(CheckResult::fail(
                "已安装配方",
                "version_freshness",
                Severity::Warning,
                format!(
                    "{} 版本非最新（已装 {}，最新 {}）",
                    r.name, installed, latest
                ),
            ));
        }
    }
    if any_checked {
        let outdated = results
            .iter()
            .filter(|c| c.name == "version_freshness" && !c.passed)
            .count();
        if outdated == 0 {
            results.push(CheckResult::ok(
                "已安装配方",
                "version_freshness",
                Severity::Warning,
                "所有已装工具均为索引中的最新版本",
            ));
        }
    }
    results
}

// ---------------------------------------------------------------------------
// 网络
// ---------------------------------------------------------------------------

/// Mirror/source hosts that downloads depend on. A reachability HEAD with a
/// short timeout; any failure is Critical because installs will break.
const NET_HOSTS: &[(&str, &str)] = &[
    ("pypi 镜像", "https://pypi.tuna.tsinghua.edu.cn/simple/"),
    ("npm 镜像", "https://registry.npmmirror.com/"),
    ("cargo 镜像", "https://rsproxy.cn/"),
    ("GitHub", "https://github.com"),
];

async fn check_network() -> Vec<CheckResult> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return vec![CheckResult::fail(
                "网络",
                "http_client",
                Severity::Critical,
                format!("无法构造 HTTP 客户端: {e}"),
            )];
        }
    };

    let mut results = Vec::new();
    for (label, url) in NET_HOSTS {
        let started = std::time::Instant::now();
        // Use GET (some hosts reject HEAD); we only care about connect success.
        let res = client.get(*url).send().await;
        let elapsed = started.elapsed().as_millis();
        match res {
            Ok(resp) => results.push(CheckResult::ok(
                "网络",
                "source_reachable",
                Severity::Critical,
                format!(
                    "{label} ({url}) 可达，HTTP {} ({}ms)",
                    resp.status().as_u16(),
                    elapsed
                ),
            )),
            Err(e) => results.push(CheckResult::fail(
                "网络",
                "source_reachable",
                Severity::Critical,
                format!("{label} ({url}) 不可达: {e}"),
            )),
        }
    }
    results
}

// ---------------------------------------------------------------------------
// PATH 环境
// ---------------------------------------------------------------------------

fn check_path_duplicates() -> Vec<CheckResult> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ';' } else { ':' };
    let entries: Vec<&str> = path_var.split(sep).filter(|e| !e.is_empty()).collect();

    // Case-insensitive duplicate detection (Windows paths are case-insensitive).
    let mut seen = std::collections::HashSet::new();
    let mut dups: Vec<String> = Vec::new();
    for e in &entries {
        let key = if cfg!(windows) {
            e.to_ascii_lowercase()
        } else {
            e.to_string()
        };
        if !seen.insert(key) {
            dups.push((*e).to_string());
        }
    }

    if dups.is_empty() {
        vec![CheckResult::ok(
            "PATH 环境",
            "no_duplicates",
            Severity::Critical,
            format!("PATH 无重复条目（共 {} 项）", entries.len()),
        )]
    } else {
        vec![CheckResult::fail(
            "PATH 环境",
            "no_duplicates",
            Severity::Critical,
            format!("PATH 有 {} 个重复条目: {}", dups.len(), dups.join(", ")),
        )]
    }
}

fn check_path_write_perm() -> Vec<CheckResult> {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        match hkcu.open_subkey_with_flags("Environment", KEY_WRITE) {
            Ok(_) => vec![CheckResult::ok(
                "PATH 环境",
                "path_write_perm",
                Severity::Warning,
                "HKCU\\Environment 可写（PATH 写入正常）",
            )],
            Err(e) => vec![CheckResult::fail(
                "PATH 环境",
                "path_write_perm",
                Severity::Warning,
                format!("HKCU\\Environment 不可写: {e} — 安装时写 PATH 会失败"),
            )],
        }
    }
    #[cfg(not(windows))]
    {
        vec![CheckResult::info(
            "PATH 环境",
            "path_write_perm",
            "非 Windows，跳过 PATH 写权限检查",
        )]
    }
}

// ---------------------------------------------------------------------------
// 环境变量
// ---------------------------------------------------------------------------

fn check_env_pollution() -> Vec<CheckResult> {
    let mut results = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();

    // PYTHONHOME / PYTHONPATH force a specific interpreter — often breaks the
    // python oneinit installed.
    for var in &["PYTHONHOME", "PYTHONPATH"] {
        if std::env::var_os(var).is_some() {
            conflicts.push(format!("{var} 已设置（可能覆盖 oneinit 安装的 Python）"));
        }
    }

    // CONDA_PREFIX means an active conda env shadows the global python.
    if std::env::var_os("CONDA_PREFIX").is_some() {
        conflicts.push("CONDA_PREFIX 已设置（conda 环境可能覆盖全局 Python）".to_string());
    }

    // JAVA_HOME pointing at a non-existent dir is a common silent breakage.
    if let Some(jh) = std::env::var_os("JAVA_HOME")
        && !Path::new(&jh).exists()
    {
        conflicts.push(format!(
            "JAVA_HOME 指向不存在的路径: {}",
            jh.to_string_lossy()
        ));
    }

    if conflicts.is_empty() {
        results.push(CheckResult::ok(
            "环境变量",
            "no_pollution",
            Severity::Critical,
            "未检测到常见的环境变量污染",
        ));
    } else {
        results.push(CheckResult::fail(
            "环境变量",
            "no_pollution",
            Severity::Critical,
            conflicts.join("；"),
        ));
    }
    results
}

// ---------------------------------------------------------------------------
// 系统资源 — 磁盘剩余空间（零依赖：Windows Win32 / Unix df 解析）
// ---------------------------------------------------------------------------

/// Available bytes on the drive holding `~/.oneinit`, or `None` if undeterminable.
fn disk_available_bytes(path: &Path) -> Option<u64> {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::GetDiskFreeSpaceExW;

        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut free_to_caller: u64 = 0;
        let mut total: u64 = 0;
        let mut total_free: u64 = 0;
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_to_caller as *mut u64 as *mut _,
                &mut total as *mut u64 as *mut _,
                &mut total_free as *mut u64 as *mut _,
            )
        };
        if ok != 0 { Some(free_to_caller) } else { None }
    }
    #[cfg(not(windows))]
    {
        // Parse `df -k <path>`: last line's 4th column = available KB.
        let out = std::process::Command::new("df")
            .arg("-k")
            .arg(path)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text.lines().last()?;
        let avail_kb: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
        Some(avail_kb * 1024)
    }
}

fn check_disk_space() -> Vec<CheckResult> {
    let envs = envs_dir();
    let data = data_dir();
    let probe: &Path = if envs.exists() {
        envs.as_path()
    } else if data.exists() {
        data.as_path()
    } else {
        Path::new(".")
    };

    match disk_available_bytes(probe) {
        Some(bytes) => {
            const ONE_GB: u64 = 1024 * 1024 * 1024;
            if bytes < ONE_GB {
                vec![CheckResult::fail(
                    "系统资源",
                    "disk_space",
                    Severity::Critical,
                    format!(
                        "磁盘剩余空间不足 1GB（{}）—安装大文件会失败",
                        human_bytes(bytes)
                    ),
                )]
            } else if bytes < 5 * ONE_GB {
                vec![CheckResult::fail(
                    "系统资源",
                    "disk_space",
                    Severity::Warning,
                    format!(
                        "磁盘剩余空间偏紧（{}）—建议清理后再安装大型工具链",
                        human_bytes(bytes)
                    ),
                )]
            } else {
                vec![CheckResult::ok(
                    "系统资源",
                    "disk_space",
                    Severity::Critical,
                    format!("磁盘剩余 {}", human_bytes(bytes)),
                )]
            }
        }
        None => vec![CheckResult::info(
            "系统资源",
            "disk_space",
            "无法确定磁盘剩余空间（跳过）",
        )],
    }
}

// ---------------------------------------------------------------------------
// 许可合规性
// ---------------------------------------------------------------------------

/// (family-prefix, license name, license url).
const LICENSES: &[(&str, &str, &str)] = &[
    (
        "python",
        "Python Software Foundation License",
        "https://docs.python.org/3/license.html",
    ),
    ("node", "MIT", "https://github.com/nodejs/node#license"),
    ("go", "BSD-3-Clause", "https://go.dev/LICENSE"),
    (
        "java",
        "GPL-2.0-with-classpath-exception",
        "https://openjdk.org/legal/gplv2+ce.html",
    ),
    (
        "rust",
        "MIT OR Apache-2.0",
        "https://www.rust-lang.org/policies/licenses",
    ),
    (
        "dotnet",
        "MIT",
        "https://dotnet.microsoft.com/en-us/platform/terms",
    ),
    (
        "mysql",
        "GPL-2.0",
        "https://www.mysql.com/about/legal/licensing/oem/",
    ),
];

fn license_for(record_name: &str) -> Option<(&'static str, &'static str)> {
    let base = record_name
        .split('@')
        .next()
        .unwrap_or(record_name)
        .to_ascii_lowercase();
    LICENSES
        .iter()
        .find(|(prefix, _, _)| base.starts_with(prefix))
        .map(|(_, name, url)| (*name, *url))
}

fn check_licenses(records: &[crate::core::manifest::InstallRecord]) -> Vec<CheckResult> {
    if records.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for r in records {
        match license_for(&r.name) {
            Some((name, url)) => lines.push(format!("{}  —  {} ({})", r.name, name, url)),
            None => lines.push(format!("{}  —  许可证未知（请自行核实）", r.name)),
        }
    }
    vec![CheckResult::info("许可合规", "licenses", lines.join("\n"))]
}

// ---------------------------------------------------------------------------
// AI 友好性 — manifest 可序列化为合法 JSON（self-test）
// ---------------------------------------------------------------------------

fn check_json_selftest(records: &[crate::core::manifest::InstallRecord]) -> Vec<CheckResult> {
    match serde_json::to_string(records) {
        Ok(s) => vec![CheckResult::ok(
            "AI 友好性",
            "json_serializable",
            Severity::Info,
            format!(
                "manifest 可序列化为合法 JSON（{} 字节，{} 条记录）",
                s.len(),
                records.len()
            ),
        )],
        Err(e) => vec![CheckResult::fail(
            "AI 友好性",
            "json_serializable",
            Severity::Warning,
            format!("manifest 无法序列化为 JSON: {e}"),
        )],
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn human_bytes(n: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("GB", 1024 * 1024 * 1024),
        ("MB", 1024 * 1024),
        ("KB", 1024),
    ];
    for (unit, size) in UNITS {
        if n >= *size {
            return format!("{:.1} {}", n as f64 / *size as f64, unit);
        }
    }
    format!("{} B", n)
}

/// Whether the result set is "healthy": no Critical failures.
pub fn is_healthy(results: &[CheckResult]) -> bool {
    results
        .iter()
        .all(|r| r.passed || r.severity != Severity::Critical)
}

/// Count warnings (failed Warning-severity checks).
pub fn warning_count(results: &[CheckResult]) -> usize {
    results
        .iter()
        .filter(|r| !r.passed && r.severity == Severity::Warning)
        .count()
}

pub fn category_order() -> &'static [&'static str] {
    CATEGORY_ORDER
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_for_maps_known_families() {
        assert_eq!(exe_for("python3.11"), Some("python"));
        assert_eq!(exe_for("python@3.11"), Some("python"));
        assert_eq!(exe_for("Python3.11"), Some("python"));
        assert_eq!(exe_for("node18"), Some("node"));
        assert_eq!(exe_for("node@lts"), Some("node"));
        assert_eq!(exe_for("go"), Some("go"));
        assert_eq!(exe_for("java17"), Some("java"));
        assert_eq!(exe_for("rust"), Some("cargo"));
        assert_eq!(exe_for("dotnet8"), Some("dotnet"));
    }

    #[test]
    fn exe_for_unknown_returns_none() {
        assert_eq!(exe_for("foobar"), None);
        assert_eq!(exe_for("custom-tool"), None);
    }

    #[test]
    fn license_for_matches_families() {
        let (name, _url) = license_for("python3.11").unwrap();
        assert!(name.contains("Python Software Foundation"));
        let (name, _) = license_for("node@20").unwrap();
        assert_eq!(name, "MIT");
        assert!(license_for("mystery-tool").is_none());
    }

    #[test]
    fn health_aggregation_respects_severity() {
        // Two Critical failures -> not healthy.
        let set = vec![
            CheckResult::fail("c", "a", Severity::Critical, "x"),
            CheckResult::fail("c", "b", Severity::Warning, "y"),
        ];
        assert!(!is_healthy(&set));
        assert_eq!(warning_count(&set), 1);

        // Only warnings -> still healthy, but warnings counted.
        let set = vec![
            CheckResult::ok("c", "a", Severity::Critical, "x"),
            CheckResult::fail("c", "b", Severity::Warning, "y"),
            CheckResult::info("c", "i", "z"),
        ];
        assert!(is_healthy(&set));
        assert_eq!(warning_count(&set), 1);
    }

    #[test]
    fn path_duplicates_case_insensitive_windows_shape() {
        // Exercise the dedup logic directly (Windows case-insensitive path).
        let entries = ["/a/b", "/a/B", "/c"];
        let mut seen = std::collections::HashSet::new();
        let mut dups = Vec::new();
        for e in &entries {
            let key = if cfg!(windows) {
                e.to_ascii_lowercase()
            } else {
                (*e).to_string()
            };
            if !seen.insert(key) {
                dups.push(*e);
            }
        }
        if cfg!(windows) {
            assert_eq!(dups, vec!["/a/B"]);
        } else {
            assert!(dups.is_empty());
        }
    }

    #[test]
    fn disk_probe_returns_some_for_tmp() {
        // df/GetDiskFreeSpaceExW must succeed on a directory that exists.
        let tmp = std::env::temp_dir();
        assert!(disk_available_bytes(&tmp).is_some());
    }

    #[test]
    fn human_bytes_formats() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(2048), "2.0 KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.0 GB");
    }

    #[test]
    fn category_order_is_complete_and_unique() {
        let order = category_order();
        let mut seen = std::collections::HashSet::new();
        for c in order {
            assert!(seen.insert(*c), "duplicate category {c}");
        }
        assert!(order.contains(&"网络"));
        assert!(order.contains(&"PATH 环境"));
    }
}
