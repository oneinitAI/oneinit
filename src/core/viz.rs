//! oneinit viz — 环境可视化
//!
//! 输出 `~/.oneinit/` 的 ASCII 树状图（已装工具/版本/激活状态/全局包/
//! 缓存状态/磁盘占用），并支持生成 HTML(SVG) 报告与「Issue 环境快照」。
//!
//! 全部数据来自本机磁盘（Manifest SQLite / envs/ / cache/ / recipes/），
//! 不发起任何网络请求。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{CoreError, Result, cache_dir, data_dir, db_dir, envs_dir, recipes_dir, temp_dir};

// ============================================================
// 数据模型
// ============================================================

#[derive(Debug, Clone, Serialize, Default)]
pub struct VizReport {
    pub tool_version: String,
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub oneinit_dir: String,
    pub total_bytes: u64,
    pub envs: Vec<VizEnv>,
    pub db: VizDb,
    pub cache: VizCache,
    pub recipes_count: usize,
    pub temp_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VizEnv {
    pub name: String,
    pub version: String,
    pub install_path: String,
    pub active: bool,
    pub present: bool,
    pub disk_bytes: u64,
    pub installed_at: String,
    pub global_packages: Vec<String>,
    pub depends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct VizDb {
    pub db_bytes: u64,
    pub records: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct VizCache {
    pub file_count: usize,
    pub size_bytes: u64,
    pub has_index: bool,
    pub index_stale: bool,
    pub packages: usize,
    pub last_updated: String,
}

// ============================================================
// 数据采集
// ============================================================

/// 采集环境报告。`scan_packages=false` 时跳过全局包扫描（快速模式）。
pub fn gather(scan_packages: bool) -> VizReport {
    let manifest = super::manifest::Manifest::open().ok();
    let records = manifest
        .as_ref()
        .and_then(|m| m.list().ok())
        .unwrap_or_default();

    // 全局包扫描（仅 python/node 检测器会产生包列表），按工具族匹配
    let scanned = if scan_packages {
        scan_runtime_envs()
    } else {
        BTreeMap::new()
    };

    let envs = records
        .into_iter()
        .map(|rec| {
            let install_dir = envs_dir().join(&rec.install_path);
            let present = install_dir.exists();
            let active = rec.path_entries.iter().any(|p| {
                let rendered = super::community_recipe::render_template(p, &envs_dir());
                super::path_mgr::is_in_path(Path::new(&rendered))
            });
            let family = tool_family(&rec.name);
            let global_packages = scanned
                .get(&family)
                .map(|e| e.global_packages.clone())
                .unwrap_or_default();
            let depends = super::community_recipe::resolve(&rec.name)
                .and_then(|r| r.depends)
                .unwrap_or_default();

            VizEnv {
                name: rec.name,
                version: rec.version.unwrap_or_default(),
                install_path: rec.install_path,
                active,
                present,
                disk_bytes: dir_bytes(&install_dir),
                installed_at: rec.installed_at,
                global_packages,
                depends,
            }
        })
        .collect::<Vec<_>>();
    let env_count = envs.len();

    let (cache_files, cache_size) = dir_stats(&cache_dir());
    let index = super::registry::load_cached_index();
    let db_bytes = fs_metadata_len(&db_dir().join("oneinit.db"));

    let recipes_count = count_yaml(&recipes_dir());
    let (_, temp_size) = dir_stats(&temp_dir());
    let (_, total) = dir_stats(&data_dir());

    VizReport {
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: super::capture::hostname(),
        oneinit_dir: data_dir().to_string_lossy().to_string(),
        total_bytes: total,
        envs,
        db: VizDb {
            db_bytes,
            records: env_count,
        },
        cache: VizCache {
            file_count: cache_files,
            size_bytes: cache_size,
            has_index: index.is_some(),
            index_stale: super::registry::is_index_stale(24),
            packages: index.as_ref().map(|i| i.packages.len()).unwrap_or(0),
            last_updated: index
                .as_ref()
                .map(|i| i.last_updated.clone())
                .unwrap_or_default(),
        },
        recipes_count,
        temp_bytes: temp_size,
    }
}

/// 运行全局环境扫描（同步；检测失败会被吞掉，返回空 map）
fn scan_runtime_envs() -> BTreeMap<String, super::capture::RuntimeEnv> {
    let mut scheduler = super::capture::detector::DetectorScheduler::new();
    scheduler.register_defaults();
    scheduler
        .scan()
        .into_iter()
        .filter_map(|(k, v)| v.map(|env| (k, env)))
        .collect()
}

/// 工具族：python3.11 → python，node20 → node，rust-stable → rust
fn tool_family(name: &str) -> String {
    let idx = name
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(name.len());
    name[..idx].trim_end_matches('-').to_string()
}

/// 递归统计目录 (文件数, 字节数)
fn dir_stats(dir: &Path) -> (usize, u64) {
    let mut files = 0usize;
    let mut bytes = 0u64;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if let Ok(md) = std::fs::metadata(&p) {
                if md.is_dir() {
                    let (f, b) = dir_stats(&p);
                    files += f;
                    bytes += b;
                } else {
                    files += 1;
                    bytes += md.len();
                }
            }
        }
    }
    (files, bytes)
}

/// 目录总字节数
fn dir_bytes(dir: &Path) -> u64 {
    let (_, b) = dir_stats(dir);
    b
}

fn fs_metadata_len(p: &Path) -> u64 {
    std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

fn count_yaml(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yaml"))
                .count()
        })
        .unwrap_or(0)
}

// ============================================================
// 格式化工具
// ============================================================

/// 字节 → 可读大小（B/KB/MB/GB/TB）
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0usize;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", v, UNITS[i])
    }
}

/// 工具显示名：python3.11@3.11.9
fn env_display_name(env: &VizEnv) -> String {
    if env.version.is_empty() {
        env.name.clone()
    } else {
        format!("{}@{}", env.name, env.version)
    }
}

// ============================================================
// ASCII 树
// ============================================================

pub fn render_ascii(report: &VizReport) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "{}  (total {})",
        report.oneinit_dir,
        human_bytes(report.total_bytes)
    ));
    s.push('\n');

    // envs/
    s.push_str("├── envs/");
    if report.envs.is_empty() {
        s.push_str("  (empty)\n");
    } else {
        s.push_str(&format!("  ({} tools)\n", report.envs.len()));
        for (i, env) in report.envs.iter().enumerate() {
            let last_env = i + 1 == report.envs.len();
            let branch = if last_env { "└──" } else { "├──" };
            let cont = if last_env { "    " } else { "│   " };
            let mut line = format!("│   {branch} {}/", env_display_name(env));
            if env.active {
                line.push_str("  (active)");
            }
            if !env.present {
                line.push_str("  (⚠ dir missing)");
            }
            s.push_str(&line);
            s.push('\n');
            if !env.global_packages.is_empty() || !env.depends.is_empty() {
                let mut sub = format!("│{cont}└── pip: {}", env.global_packages.join(", "));
                if !env.depends.is_empty() {
                    sub.push_str(&format!("  [depends: {}]", env.depends.join(", ")));
                }
                s.push_str(&sub);
                s.push('\n');
            }
        }
    }

    // db/
    s.push_str("├── db/\n");
    s.push_str(&format!(
        "│   └── oneinit.db  (SQLite manifest · {} records · {})\n",
        report.db.records,
        human_bytes(report.db.db_bytes)
    ));

    // cache/
    s.push_str(&format!(
        "├── cache/  ({} · {} files)\n",
        human_bytes(report.cache.size_bytes),
        report.cache.file_count
    ));
    if report.cache.has_index {
        let stale = if report.cache.index_stale {
            " · stale"
        } else {
            ""
        };
        s.push_str(&format!(
            "│   └── INDEX.json  ({} packages{stale})\n",
            report.cache.packages
        ));
    } else {
        s.push_str("│   └── (no INDEX.json)\n");
    }

    // recipes/
    s.push_str("├── recipes/");
    if report.recipes_count == 0 {
        s.push_str("  (empty)\n");
    } else {
        s.push_str(&format!("  ({} community recipes)\n", report.recipes_count));
    }

    // temp/（最后一项）
    s.push_str(&format!(
        "└── temp/  ({})\n",
        human_bytes(report.temp_bytes)
    ));

    s
}

// ============================================================
// HTML(SVG) 报告
// ============================================================

/// HTML 特殊字符转义
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 渲染自包含 HTML 报告（内联 CSS + 内联 SVG 依赖树，无外部依赖）
pub fn render_html(report: &VizReport) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"zh\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>OneInit Environment Report</title>\n");
    out.push_str("<style>\n");
    out.push_str(include_str!("viz_html_head.css"));
    out.push_str("</style>\n");
    out.push_str("</head>\n<body>\n");

    // 头部
    out.push_str("<header><h1>🧰 OneInit Environment Report</h1><p class=\"meta\">");
    out.push_str(&format!(
        "oneinit {} · {} / {} · {}",
        html_escape(&report.tool_version),
        html_escape(&report.os),
        html_escape(&report.arch),
        html_escape(&report.hostname)
    ));
    out.push_str("</p></header>\n");

    // 统计卡
    out.push_str("<section class=\"stats\">\n");
    out.push_str(&stat_card(
        "已安装工具",
        &report.envs.len().to_string(),
        "tools",
    ));
    out.push_str(&stat_card(
        "缓存",
        &human_bytes(report.cache.size_bytes),
        "cache",
    ));
    out.push_str(&stat_card(
        "注册表包",
        &report.cache.packages.to_string(),
        if report.cache.index_stale {
            "stale"
        } else {
            "ok"
        },
    ));
    out.push_str(&stat_card(
        "总占用",
        &human_bytes(report.total_bytes),
        "total",
    ));
    out.push_str("</section>\n");

    // SVG 依赖树
    out.push_str("<section class=\"tree\"><h2>环境依赖树 / Environment Tree</h2>\n");
    out.push_str(&render_svg(report));
    out.push_str("</section>\n");

    out.push_str("<footer>generated by <code>oneinit viz --html</code></footer>\n");
    out.push_str("</body>\n</html>\n");
    out
}

fn stat_card(label: &str, value: &str, badge: &str) -> String {
    format!(
        "<div class=\"card\"><span class=\"label\">{}</span><span class=\"value\">{}</span><span class=\"badge {}\">{}</span></div>\n",
        html_escape(label),
        html_escape(value),
        html_escape(badge),
        html_escape(badge)
    )
}

/// 树节点（用于 SVG 布局）
struct TNode {
    label: String,
    badge: Option<(String, String)>, // (text, css-class)
    children: Vec<TNode>,
}

fn build_tree(report: &VizReport) -> TNode {
    let envs_node = TNode {
        label: format!("envs/ · {} tools", report.envs.len()),
        badge: None,
        children: report
            .envs
            .iter()
            .map(|env| {
                let badge = if env.active {
                    Some(("active".to_string(), "b-active".to_string()))
                } else if !env.present {
                    Some(("missing".to_string(), "b-missing".to_string()))
                } else {
                    None
                };
                let mut label = format!("{}/", env_display_name(env));
                if !env.global_packages.is_empty() {
                    label.push_str(&format!("  · pip: {}", env.global_packages.join(", ")));
                }
                TNode {
                    label,
                    badge,
                    children: Vec::new(),
                }
            })
            .collect(),
    };

    let db_node = TNode {
        label: format!("db/ · oneinit.db · {} records", report.db.records),
        badge: None,
        children: Vec::new(),
    };

    let cache_node = TNode {
        label: format!(
            "cache/ · {} · {} files",
            human_bytes(report.cache.size_bytes),
            report.cache.file_count
        ),
        badge: if report.cache.index_stale {
            Some(("stale".to_string(), "b-warn".to_string()))
        } else {
            None
        },
        children: if report.cache.has_index {
            vec![TNode {
                label: format!("INDEX.json · {} packages", report.cache.packages),
                badge: None,
                children: Vec::new(),
            }]
        } else {
            vec![]
        },
    };

    let recipes_node = TNode {
        label: format!("recipes/ · {} community recipes", report.recipes_count),
        badge: None,
        children: Vec::new(),
    };

    let temp_node = TNode {
        label: format!("temp/ · {}", human_bytes(report.temp_bytes)),
        badge: None,
        children: Vec::new(),
    };

    TNode {
        label: format!(
            "{}  (total {})",
            report.oneinit_dir,
            human_bytes(report.total_bytes)
        ),
        badge: None,
        children: vec![envs_node, db_node, cache_node, recipes_node, temp_node],
    }
}

/// 展平树 → 行（depth, is_last_in_parent, label, badge）
struct Row {
    depth: usize,
    last: bool,
    label: String,
    badge: Option<(String, String)>, // (text, css-class)
}

fn flatten(node: &TNode, depth: usize, last: bool, out: &mut Vec<Row>) {
    out.push(Row {
        depth,
        last,
        label: node.label.clone(),
        badge: node.badge.clone(),
    });
    let n = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        flatten(child, depth + 1, i + 1 == n, out);
    }
}

/// 渲染 SVG 树
fn render_svg(report: &VizReport) -> String {
    let tree = build_tree(report);
    let mut rows: Vec<Row> = Vec::new();
    flatten(&tree, 0, true, &mut rows);

    let row_h = 46.0;
    let pad = 24.0;
    let width = 900.0;
    let height = pad * 2.0 + rows.len() as f64 * row_h;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {width} {height}\" xmlns=\"http://www.w3.org/2000/svg\" font-family=\"ui-monospace, SFMono-Regular, Menlo, Consolas, monospace\">\n"
    ));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#0d0d14\"/>\n");

    // 连线（先画，再画节点）
    for (i, row) in rows.iter().enumerate() {
        if row.depth == 0 {
            continue;
        }
        let y = pad + i as f64 * row_h + row_h / 2.0;
        let indent = pad + 20.0 + row.depth as f64 * 30.0;
        let elbow_x = indent - 18.0;
        // 横向短肘
        svg.push_str(&format!(
            "<line x1=\"{elbow_x}\" y1=\"{y}\" x2=\"{indent}\" y2=\"{y}\" stroke=\"#3f3f4d\" stroke-width=\"2\"/>\n"
        ));
        // 纵向主干：从父行中心到本行（最后子项则止于此，否则延伸一格）
        let parent_y = {
            // 找父行：向上找第一个 depth == row.depth - 1 的行
            let mut py = pad + row_h / 2.0;
            for (j, r) in rows[..i].iter().enumerate().rev() {
                if r.depth + 1 == row.depth {
                    py = pad + j as f64 * row_h + row_h / 2.0;
                    break;
                }
            }
            py
        };
        let end_y = if row.last { y } else { y + row_h / 2.0 + 1.0 };
        if (end_y - parent_y).abs() > 1.0 {
            svg.push_str(&format!(
                "<line x1=\"{elbow_x}\" y1=\"{parent_y}\" x2=\"{elbow_x}\" y2=\"{end_y}\" stroke=\"#3f3f4d\" stroke-width=\"2\"/>\n"
            ));
        }
    }

    // 节点
    for (i, row) in rows.iter().enumerate() {
        let y = pad + i as f64 * row_h;
        let indent = pad + 20.0 + row.depth as f64 * 30.0;
        let box_w = width - indent - 30.0;
        let box_h = 34.0;
        let ry = y + (row_h - box_h) / 2.0;
        let is_root = row.depth == 0;
        let fill = if is_root { "#10b9811f" } else { "#1a1a24" };
        let stroke = if is_root { "#10b981" } else { "#2c2c3a" };
        svg.push_str(&format!(
            "<rect x=\"{indent}\" y=\"{ry}\" width=\"{box_w}\" height=\"{box_h}\" rx=\"8\" fill=\"{fill}\" stroke=\"{stroke}\"/>\n"
        ));
        // badge 在右侧
        if let Some((text, class)) = &row.badge {
            let tw = (text.len() * 8) as f64 + 24.0;
            let bx = indent + box_w - tw - 12.0;
            let color = match class.as_str() {
                "b-active" => "#10b981",
                "b-missing" => "#ef4444",
                _ => "#f59e0b",
            };
            let by = ry + 6.0;
            let tx = bx + tw / 2.0;
            let ty = ry + 21.0;
            svg.push_str(&format!(
                "<rect x=\"{bx}\" y=\"{by}\" width=\"{tw}\" height=\"22\" rx=\"11\" fill=\"{color}22\" stroke=\"{color}\"/><text x=\"{tx}\" y=\"{ty}\" fill=\"{color}\" font-size=\"12\" text-anchor=\"middle\">{}</text>\n",
                html_escape(text)
            ));
        }
        // 标签
        let text_color = if is_root { "#ffffff" } else { "#d4d4d8" };
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"13\">{}</text>\n",
            indent + 12.0,
            ry + box_h / 2.0 + 4.5,
            text_color,
            html_escape(&row.label)
        ));
    }

    svg.push_str("</svg>\n");
    svg
}

// ============================================================
// Issue 快照（Markdown）
// ============================================================

/// 生成可直接粘贴到 GitHub Issue 的环境快照
pub fn render_issue(report: &VizReport) -> String {
    let mut s = String::new();
    s.push_str("<!-- generated by `oneinit viz --issue` -->\n");
    s.push_str("## Environment Snapshot / 环境快照\n\n");
    s.push_str(&format!("- **OneInit**: `{}`\n", report.tool_version));
    s.push_str(&format!(
        "- **OS / Arch**: `{}` / `{}`\n",
        report.os, report.arch
    ));
    s.push_str(&format!("- **Host**: `{}`\n", report.hostname));
    s.push_str(&format!(
        "- **~/.oneinit**: `{}` (total {})\n",
        report.oneinit_dir,
        human_bytes(report.total_bytes)
    ));
    s.push('\n');

    s.push_str("### Installed tools / 已安装工具\n\n");
    if report.envs.is_empty() {
        s.push_str("_none_\n\n");
    } else {
        s.push_str("| Tool | Version | Active | Install path |\n");
        s.push_str("|------|---------|--------|--------------|\n");
        for env in &report.envs {
            s.push_str(&format!(
                "| {} | {} | {} | `{}` |\n",
                env.name,
                if env.version.is_empty() {
                    "?"
                } else {
                    &env.version
                },
                if env.active { "✅" } else { "—" },
                env.install_path
            ));
        }
        s.push('\n');
        // 全局包
        let with_pkgs: Vec<&VizEnv> = report
            .envs
            .iter()
            .filter(|e| !e.global_packages.is_empty())
            .collect();
        if !with_pkgs.is_empty() {
            s.push_str("### Global packages / 全局包\n\n");
            for env in with_pkgs {
                s.push_str(&format!(
                    "- **{}**: {}\n",
                    env.name,
                    env.global_packages.join(", ")
                ));
            }
            s.push('\n');
        }
    }

    s.push_str("### Environment tree / 环境树\n\n```text\n");
    s.push_str(&render_ascii(report));
    s.push_str("```\n\n");

    s.push_str("### Cache / 缓存\n\n");
    if report.cache.has_index {
        s.push_str(&format!(
            "- INDEX.json: {} packages · {}\n",
            report.cache.packages,
            if report.cache.index_stale {
                "stale".to_string()
            } else {
                "fresh".to_string()
            }
        ));
    } else {
        s.push_str("- INDEX.json: not cached\n");
    }
    s.push_str(&format!(
        "- cache size: {} / {} files\n",
        human_bytes(report.cache.size_bytes),
        report.cache.file_count
    ));
    s
}

// ============================================================
// 输出辅助
// ============================================================

/// 将 ASCII 树写入文件 / 打印，供 cli 层调用
pub fn write_output(path: &str, content: &str) -> Result<()> {
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, content)
        .map_err(|e| CoreError::Other(format!("写入 {} 失败: {}", path, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> VizReport {
        VizReport {
            tool_version: "0.1.0-beta.2".to_string(),
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            hostname: "TEST-PC".to_string(),
            oneinit_dir: "C:\\Users\\t\\.oneinit".to_string(),
            total_bytes: 1_288_490_188, // ~1.2 GB
            envs: vec![
                VizEnv {
                    name: "python3.11".to_string(),
                    version: "3.11.9".to_string(),
                    install_path: "python3.11".to_string(),
                    active: true,
                    present: true,
                    disk_bytes: 512_000_000,
                    installed_at: "2026-08-01T00:00:00Z".to_string(),
                    global_packages: vec!["numpy".into(), "pandas".into(), "requests".into()],
                    depends: vec![],
                },
                VizEnv {
                    name: "node20".to_string(),
                    version: "20.18.1".to_string(),
                    install_path: "node20".to_string(),
                    active: false,
                    present: true,
                    disk_bytes: 300_000_000,
                    installed_at: "2026-07-01T00:00:00Z".to_string(),
                    global_packages: vec![],
                    depends: vec![],
                },
                VizEnv {
                    name: "go".to_string(),
                    version: "1.23.4".to_string(),
                    install_path: "go".to_string(),
                    active: false,
                    present: false,
                    disk_bytes: 0,
                    installed_at: "2026-06-01T00:00:00Z".to_string(),
                    global_packages: vec![],
                    depends: vec![],
                },
            ],
            db: VizDb {
                db_bytes: 4096,
                records: 3,
            },
            cache: VizCache {
                file_count: 3,
                size_bytes: 1_288_486_092,
                has_index: true,
                index_stale: true,
                packages: 5,
                last_updated: "2026-07-30T00:00:00Z".to_string(),
            },
            recipes_count: 2,
            temp_bytes: 12_582_912,
        }
    }

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1536), "1.5 KB");
        assert_eq!(human_bytes(1_288_490_188), "1.2 GB");
        assert_eq!(human_bytes(1_288_490_188_000), "1.2 TB");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<a & \"b\">"), "&lt;a &amp; &quot;b&quot;&gt;");
    }

    #[test]
    fn test_tool_family() {
        assert_eq!(tool_family("python3.11"), "python");
        assert_eq!(tool_family("node20"), "node");
        assert_eq!(tool_family("rust"), "rust");
        assert_eq!(tool_family("go"), "go");
        assert_eq!(tool_family("java17"), "java");
    }

    #[test]
    fn test_render_ascii_tree() {
        let tree = render_ascii(&sample_report());
        assert!(tree.contains("python3.11@3.11.9"));
        assert!(tree.contains("(active)"));
        assert!(tree.contains("⚠ dir missing"));
        assert!(tree.contains("numpy, pandas, requests"));
        assert!(tree.contains("oneinit.db"));
        assert!(tree.contains("5 packages"));
        assert!(tree.contains("stale"));
        assert!(tree.contains("2 community recipes"));
    }

    #[test]
    fn test_render_ascii_empty() {
        let report = VizReport::default();
        let tree = render_ascii(&report);
        assert!(tree.contains("envs/  (empty)"));
        assert!(tree.contains("no INDEX.json"));
        assert!(tree.contains("recipes/  (empty)"));
    }

    #[test]
    fn test_render_html_contains_svg() {
        let html = render_html(&sample_report());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<svg"));
        assert!(html.contains("python3.11@3.11.9"));
        assert!(html.contains(">active<")); // active 徽标
        assert!(html.contains("</html>"));
        // 内联 CSS 已嵌入 <style>
        assert!(html.contains("<style>"));
        assert!(html.contains(".tree"));
    }

    #[test]
    fn test_render_issue_snapshot() {
        let md = render_issue(&sample_report());
        assert!(md.contains("## Environment Snapshot"));
        assert!(md.contains("| python3.11 | 3.11.9 | ✅ |"));
        assert!(md.contains("```text"));
        assert!(md.contains("numpy, pandas, requests"));
        assert!(md.contains("INDEX.json: 5 packages"));
    }
}
