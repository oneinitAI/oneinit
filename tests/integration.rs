//! 集成测试：通过编译出的 oneinit 二进制 + 本地 HTTP 服务器验证安装全流程。
//!
//! 隔离策略：每个测试通过 `ONEINIT_HOME` 环境变量把 oneinit 的数据目录指向
//! 临时目录（Windows 上 dirs::home_dir 走已知文件夹 API，USERPROFILE/HOME
//! 无法覆盖，故用 ONEINIT_HOME）。预置 `registry.json`（fresh last_update）+
//! `cache/INDEX.json` 跳过自动索引刷新。
//!
//! 配方使用 `binary_copy` 安装类型 + 本地 HTTP 服务器提供小文件，
//! 避免依赖真实下载源。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tiny_http::Server;

/// 本地测试 HTTP 服务器（随机端口，避免测试间冲突）
struct TestServer {
    server: Arc<Server>,
    url: String,
}

impl TestServer {
    fn new() -> Self {
        let server = Server::http("127.0.0.1:0").expect("failed to bind test server");
        let addr = server.server_addr().to_ip().expect("server addr not ip");
        let url = format!("http://127.0.0.1:{}", addr.port());
        TestServer {
            server: Arc::new(server),
            url,
        }
    }

    /// 启动一个后台线程，响应 `expected_requests` 次请求后退出。
    fn serve(&self, content: Vec<u8>, expected_requests: usize) {
        let server = Arc::clone(&self.server);
        std::thread::spawn(move || {
            for _ in 0..expected_requests {
                let Ok(Some(request)) = server.recv_timeout(Duration::from_secs(30)) else {
                    return;
                };
                let response = tiny_http::Response::from_data(content.clone());
                let _ = request.respond(response);
            }
        });
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// 初始化隔离的数据目录（ONEINIT_HOME 指向 root/oneinit），并预置跳过自动更新的配置。
/// 返回 ONEINIT_HOME 路径。
fn setup_isolated_data(root: &Path) -> PathBuf {
    let data = root.join("oneinit");
    std::fs::create_dir_all(data.join("cache")).unwrap();
    std::fs::create_dir_all(data.join("recipes")).unwrap();

    // 预置空索引 + 新鲜的 last_update：让每次运行的自动索引刷新直接跳过
    std::fs::write(
        data.join("cache").join("INDEX.json"),
        r#"{"version":1,"last_updated":"2026-01-01T00:00:00Z","packages":{}}"#,
    )
    .unwrap();
    let last_update = chrono::Utc::now().to_rfc3339();
    std::fs::write(
        data.join("registry.json"),
        format!(
            r#"{{"registry_url":"http://127.0.0.1:1/","subscriptions":[],"last_update":"{last_update}"}}"#
        ),
    )
    .unwrap();

    data
}

/// 写入一个 binary_copy 类型的三平台测试配方（三平台共用同一 URL/校验和）
fn write_binary_recipe(data: &Path, name: &str, url: &str, sha256: &str) {
    let yaml = format!(
        r#"name: {name}
version: "1.0.0"
description: "integration test recipe"
platforms:
  windows:
    url: "{url}"
    sha256: "{sha256}"
    install_type: "binary_copy"
    install_path: "{name}"
    path_add: ["{{install_dir}}"]
  linux:
    url: "{url}"
    sha256: "{sha256}"
    install_type: "binary_copy"
    install_path: "{name}"
    path_add: ["{{install_dir}}"]
  darwin:
    url: "{url}"
    sha256: "{sha256}"
    install_type: "binary_copy"
    install_path: "{name}"
    path_add: ["{{install_dir}}"]
"#,
        name = name,
        url = url,
        sha256 = sha256,
    );
    std::fs::write(data.join("recipes").join(format!("{name}.yaml")), yaml).unwrap();
}

/// 以隔离的 ONEINIT_HOME 运行 oneinit 二进制
fn run_oneinit(data: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_oneinit"))
        .args(args)
        .env("ONEINIT_HOME", data)
        .output()
        .expect("failed to run oneinit")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("oneinit-it-{}", uuid::Uuid::new_v4()))
}

// ============================================================
// 测试用例
// ============================================================

/// 端到端：安装成功 → 文件落盘 → 卸载清理
#[test]
fn install_success_and_uninstall() {
    let root = temp_root();
    let data = setup_isolated_data(&root);
    let server = TestServer::new();
    let content = b"oneinit-integration-test-binary";
    server.serve(content.to_vec(), 1);
    let url = format!("{}/tool.bin", server.url);
    write_binary_recipe(&data, "itool", &url, &sha256_hex(content));

    let out = run_oneinit(&data, &["-y", "install", "itool"]);
    assert!(
        out.status.success(),
        "install failed:\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("安装完成"),
        "missing success marker:\n{}",
        stdout(&out)
    );
    let installed = data.join("envs/itool/tool.bin");
    assert!(installed.exists(), "binary was not installed");

    let out = run_oneinit(&data, &["uninstall", "itool"]);
    assert!(out.status.success(), "uninstall failed:\n{}", stdout(&out));
    assert!(!installed.exists(), "binary was not removed");

    std::fs::remove_dir_all(&root).ok();
}

/// 下载失败 → 自动回滚（PATH 恢复 + 目录清理）
#[test]
fn download_failure_rolls_back() {
    let root = temp_root();
    let data = setup_isolated_data(&root);
    // URL 指向不可达端口：连接被拒绝 → 重试 3 次 → 失败 → 回滚
    write_binary_recipe(
        &data,
        "ftool",
        "http://127.0.0.1:1/nope.bin",
        &sha256_hex(b"x"),
    );

    let out = run_oneinit(&data, &["-y", "install", "ftool"]);
    assert!(!out.status.success(), "install should fail");
    let text = stdout(&out);
    assert!(text.contains("[ROLLBACK]"), "no rollback marker:\n{text}");
    assert!(
        !data.join("envs/ftool").exists(),
        "install dir was not cleaned"
    );

    std::fs::remove_dir_all(&root).ok();
}

/// 校验和不符 → 拒绝安装并回滚
#[test]
fn checksum_mismatch_rejected() {
    let root = temp_root();
    let data = setup_isolated_data(&root);
    let server = TestServer::new();
    let content = b"actual-content";
    server.serve(content.to_vec(), 1);
    let url = format!("{}/tool.bin", server.url);
    write_binary_recipe(&data, "ctool", &url, &sha256_hex(b"different-content"));

    let out = run_oneinit(&data, &["-y", "install", "ctool"]);
    assert!(!out.status.success(), "install should fail");
    let text = stdout(&out);
    assert!(text.contains("SHA256"), "no checksum error:\n{text}");
    assert!(
        !data.join("envs/ctool").exists(),
        "install dir was not cleaned"
    );

    std::fs::remove_dir_all(&root).ok();
}

/// dry-run 渲染计划但不执行任何安装
#[test]
fn dry_run_renders_plan_without_installing() {
    let root = temp_root();
    let data = setup_isolated_data(&root);

    let out = run_oneinit(&data, &["install", "python3.11", "--dry-run"]);
    assert!(out.status.success(), "dry-run failed:\n{}", stdout(&out));
    let text = stdout(&out);
    assert!(text.contains("[PLAN]"), "no plan marker:\n{text}");
    assert!(
        !data.join("envs/python3.11").exists(),
        "dry-run must not install"
    );

    std::fs::remove_dir_all(&root).ok();
}

/// 内置配方四级解析 + 依赖渲染（python@3.12 动态配方在无网络时给出错误而非崩溃）
#[test]
fn dynamic_family_resolution_graceful() {
    let root = temp_root();
    let data = setup_isolated_data(&root);

    // 动态 python 配方含 post_install 命令 → 无 --allow-exec 时按 H-4 拒绝；
    // 无网络时校验和解析失败 → 给出"动态配方失败"错误（不得崩溃）
    let out = run_oneinit(&data, &["install", "python@3.12", "--dry-run"]);
    let text = format!("{}\n{}", stdout(&out), stderr(&out));
    // 允许三种结果：成功渲染计划 / 安全拒绝（H-4）/ 动态解析失败（无网络）
    assert!(
        text.contains("[PLAN]")
            || text.contains("--allow-exec")
            || text.contains("未找到")
            || text.contains("动态配方失败"),
        "unexpected output:\n{text}"
    );

    std::fs::remove_dir_all(&root).ok();
}
