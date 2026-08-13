#![allow(dead_code)]

mod cli;
mod core;
mod output;
mod security;
mod skill_mgr;
mod tui2;

use clap::{Parser, Subcommand};

/// OneInit — One command to init your dev machine
///
/// The first tool to install on a new machine.
/// Developer-ready after one command.
#[derive(Parser)]
#[command(name = "oneinit", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Output in JSON format (AI-friendly)
    #[arg(
        global = true,
        long = "json",
        help = "Output in JSON format (AI-friendly)"
    )]
    json: bool,

    /// Auto-confirm all prompts (skip interactive confirmations)
    #[arg(
        global = true,
        short = 'y',
        long = "yes",
        help = "Auto-confirm all prompts (skip interactive confirmations)"
    )]
    yes: bool,

    /// Enable debug output
    #[arg(
        global = true,
        short = 'v',
        long = "debug",
        help = "Enable debug output"
    )]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize dev environment with presets
    Init {
        /// 预置套装名称（如 "python", "frontend", "ai"）
        #[arg(short, long)]
        preset: Option<String>,
        /// 只预览将要执行的操作，不实际安装
        #[arg(long)]
        dry_run: bool,
        /// 项目感知安装：扫描项目目录（requirements.txt / package.json /
        /// Cargo.toml / go.mod），自动识别并安装对应工具链（默认当前目录）
        #[arg(long, num_args = 0..=1, default_missing_value = ".")]
        project: Option<String>,
        /// 允许执行含命令的配方（默认拒绝）
        #[arg(long)]
        allow_exec: bool,
    },

    /// Install a tool（如 python3.7, node18, python@3.11, node@lts）
    #[command(visible_alias = "i")]
    Install {
        /// 要安装的工具包名称（支持 name@version，如 python@3.11 / node@lts）
        package: String,
        /// 允许执行远程配方声明的命令 / 安装器（默认拒绝）
        #[arg(
            long,
            help = "Allow recipes to run commands/installers (default: deny)"
        )]
        allow_exec: bool,
        /// 只预览将要执行的操作，不实际安装
        #[arg(long)]
        dry_run: bool,
        /// 强制刷新版本 / 校验哈希缓存
        #[arg(long)]
        refresh: bool,
        /// 跳过校验哈希验证（风险自负）
        #[arg(long)]
        no_checksum: bool,
        /// 安装失败时跳过自动回滚（便于调试）
        #[arg(long)]
        no_rollback: bool,
    },

    /// Uninstall a tool
    #[command(visible_alias = "u", visible_alias = "rm")]
    Uninstall {
        /// 要卸载的工具包名称
        package: String,
        /// 只预览将要删除的内容，不实际卸载
        #[arg(long)]
        dry_run: bool,
    },

    /// List installed tools / available versions
    #[command(visible_alias = "ls")]
    List {
        /// 输出格式: table / csv（installed 列表）
        #[arg(long, value_parser = ["table", "csv"])]
        format: Option<String>,
        #[command(subcommand)]
        action: Option<ListAction>,
    },

    /// 查看配方版本信息（如 oneinit info python@3.11）
    Info {
        /// 包名（支持 name@version）
        package: String,
    },

    /// Search available tools
    Search {
        /// 搜索关键词
        keyword: Option<String>,
    },

    /// Sync from oneinit.yaml
    #[command(visible_alias = "up")]
    Sync {
        /// 只预览将要安装的工具，不实际安装
        #[arg(long)]
        dry_run: bool,
        /// 允许执行含命令的配方（默认拒绝）
        #[arg(long)]
        allow_exec: bool,
    },

    /// Capture environment to oneinit.yaml
    Capture {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long)]
        output: Option<String>,
        /// 输出 SyncConfig 兼容格式（envs: {name: major.minor}），可直接被 sync 解析
        #[arg(long)]
        sync_format: bool,
    },

    /// Validate community recipe YAML
    Verify {
        /// recipe文件路径
        file: String,
    },

    /// 批量检查/升级已安装工具（apt upgrade 风格）
    Update {
        /// 实际执行升级（默认仅展示可升级列表）
        #[arg(long)]
        apply: bool,
        /// 强制从官方 API 刷新版本目录（默认用缓存）
        #[arg(long)]
        refresh: bool,
    },

    /// 管理配方订阅源（多注册表）
    Registry {
        #[command(subcommand)]
        action: RegistryAction,
    },

    /// 打开配方仓库 issue 表单（请求配方 / 报告 bug）
    Issue {
        /// 表单类型: recipe / bug / 空 → 打开选择页
        #[arg(default_value = "choose")]
        kind: String,
    },

    /// Publish recipe to remote registry
    Publish {
        /// recipe文件路径
        file: String,
        /// 自动提交配方到 oneinit-recipes 并创建 PR（需要 gh CLI）
        #[arg(long)]
        pr: bool,
    },

    /// 配方脚手架工具
    Recipe {
        #[command(subcommand)]
        action: RecipeAction,
    },

    /// Export environment as tar.gz
    Export {
        /// 输出文件路径（默认 oneinit-backup.tar.gz）
        #[arg(short, long, default_value = "oneinit-backup.tar.gz")]
        output: String,
        /// 包含已安装的工具目录（~/.oneinit/envs/）
        #[arg(long)]
        include_envs: bool,
    },

    /// 从 tar.gz 包导入环境
    Import {
        /// 备份文件路径
        file: String,
        /// 只预览不实际执行
        #[arg(long)]
        dry_run: bool,
        /// 强制覆盖已exists的文件
        #[arg(long)]
        force: bool,
    },

    /// Launch interactive TUI
    Tui,

    /// Environment health check（PATH 残留、清单漂移、磁盘空间）
    #[command(visible_alias = "check")]
    Doctor,

    /// Export installed tools (like pip freeze)
    Freeze {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long, default_value = "oneinit.yaml")]
        output: String,
    },

    /// Update OneInit itself to the latest release
    #[command(visible_alias = "upgrade")]
    SelfUpdate,

    /// Generate shell completion script
    Completions {
        /// Shell 类型（bash, zsh, powershell, fish, elvish）
        shell: String,
    },

    /// Install/manage AI Skill
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// 团队环境同步（team.yaml：共享开发环境，每次运行自动检测）
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },

    /// 环境可视化（ASCII 树 / HTML 报告 / Issue 快照）
    Viz {
        /// 生成 HTML(SVG) 报告
        #[arg(long)]
        html: bool,
        /// 生成 GitHub Issue 环境快照（Markdown）
        #[arg(long)]
        issue: bool,
        /// 输出文件路径（--html 默认 report.html；--issue 默认 env-snapshot.md）
        #[arg(short, long)]
        output: Option<String>,
        /// 生成 HTML 后尝试用系统浏览器打开
        #[arg(long)]
        open: bool,
        /// 跳过全局包扫描（快速模式）
        #[arg(long)]
        no_scan: bool,
    },
}

#[derive(clap::Subcommand)]
enum ListAction {
    /// 列出配方的可用版本（如 oneinit list versions python）
    Versions {
        /// 配方名（python / node / go / java / rust）
        recipe: String,
    },
}

#[derive(clap::Subcommand)]
enum TeamAction {
    /// 添加团队环境仓库（拉取 team.yaml + 验签 + 固定公钥，随后立即同步）
    Add {
        /// 仓库 URL：https://github.com/<owner>/<repo> 或 raw 地址
        url: String,
        /// 分支（默认 main）
        #[arg(long, default_value = "main")]
        branch: String,
        /// 覆盖已有配置 / 重新固定签名公钥
        #[arg(long)]
        force: bool,
        /// 允许执行含命令的配方（默认拒绝）
        #[arg(long)]
        allow_exec: bool,
    },
    /// 移除团队环境配置
    Remove,
    /// 查看团队环境状态
    Status,
    /// 立即同步团队环境（默认静默，仅内容变化时安装缺失工具）
    Sync {
        /// 忽略 24h 检测间隔与缓存哈希，强制重新同步
        #[arg(long)]
        force: bool,
        /// 允许执行含命令的配方（默认拒绝）
        #[arg(long)]
        allow_exec: bool,
        /// 只预览将要同步的内容，不实际执行
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(clap::Subcommand)]
enum SkillAction {
    /// Install Skill to detected AI agents
    Install {
        /// 指定 AI 助手（zcode, codex, claude, agents, all）。默认 all
        #[arg(short, long, default_value = "all")]
        target: String,
    },
    /// List Skill installation status per AI agent
    List,
    /// Show Skill installation status
    Status,
    /// Uninstall Skill
    Uninstall,
}

#[derive(clap::Subcommand)]
enum RecipeAction {
    /// 在当前目录生成配方模板文件 <name>.yaml
    New { name: String },
}

#[derive(clap::Subcommand)]
enum RegistryAction {
    /// 添加自定义订阅 URL（可多个）
    Add {
        /// 注册表 base URL（需提供 INDEX.json）
        url: String,
    },
    /// 移除订阅 URL
    Remove {
        /// 注册表 base URL
        url: String,
    },
    /// 列出所有订阅
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut formatter = output::OutputFormatter::new(cli.json);
    formatter.auto_yes = cli.yes;
    formatter.debug = cli.debug;

    // 高风险操作前显示免责声明
    let is_dangerous = matches!(
        cli.command,
        Commands::Install { .. }
            | Commands::Sync { .. }
            | Commands::Import { dry_run: false, .. }
            | Commands::Team {
                action: TeamAction::Add { .. } | TeamAction::Sync { .. }
            }
    );
    if is_dangerous {
        security::print_disclaimer(&formatter);
    }

    // 每次使用自动拉取配方索引（缓存缺失或过期 >24h 时静默更新）
    let skip_auto_update = matches!(
        cli.command,
        Commands::Update { .. }
            | Commands::Registry { .. }
            | Commands::Completions { .. }
            | Commands::Team { .. }
            | Commands::Sync { .. }
            | Commands::Capture { .. }
            | Commands::Export { .. }
            | Commands::Import { .. }
            | Commands::Freeze { .. }
            | Commands::Viz { .. }
            | Commands::SelfUpdate
    );
    if !skip_auto_update {
        maybe_auto_update(&formatter).await;
    }

    // 团队环境自动检测（仅检测 + 内容变化时提示，不阻塞主命令）
    let skip_team_check = matches!(
        cli.command,
        Commands::Update { .. }
            | Commands::Registry { .. }
            | Commands::Completions { .. }
            | Commands::Team { .. }
            | Commands::Sync { .. }
            | Commands::Capture { .. }
            | Commands::Export { .. }
            | Commands::Import { .. }
            | Commands::Freeze { .. }
            | Commands::Viz { .. }
            | Commands::Doctor
            | Commands::SelfUpdate
    );
    if !skip_team_check {
        cli::maybe_team_sync(&formatter).await;
    }

    match cli.command {
        Commands::Init {
            preset,
            dry_run,
            project,
            allow_exec,
        } => {
            cli::run_init(
                &formatter,
                preset.as_deref(),
                dry_run,
                allow_exec,
                project.as_deref(),
            )
            .await
        }
        Commands::Install {
            package,
            allow_exec,
            dry_run,
            refresh,
            no_checksum,
            no_rollback,
        } => {
            cli::run_install(
                &formatter,
                &package,
                allow_exec,
                dry_run,
                refresh,
                no_checksum,
                no_rollback,
            )
            .await
        }
        Commands::Uninstall { package, dry_run } => {
            cli::run_uninstall(&formatter, &package, dry_run).await
        }
        Commands::List { format, action } => match action {
            Some(ListAction::Versions { recipe }) => {
                cli::run_list_versions(&formatter, &recipe).await
            }
            None => cli::run_list(&formatter, format.as_deref()).await,
        },
        Commands::Info { package } => cli::run_info(&formatter, &package).await,
        Commands::Search { keyword } => cli::run_search(&formatter, keyword.as_deref()).await,
        Commands::Sync {
            dry_run,
            allow_exec,
        } => cli::run_sync(&formatter, dry_run, allow_exec).await,
        Commands::Capture {
            output,
            sync_format,
        } => {
            cli::run_capture(
                &formatter,
                output.as_deref().unwrap_or("oneinit.yaml"),
                sync_format,
            )
            .await
        }
        Commands::Verify { file } => cli::run_verify(&formatter, &file).await,
        Commands::Update { apply, refresh } => cli::run_update(&formatter, apply, refresh).await,
        Commands::Registry { action } => match action {
            RegistryAction::Add { url } => cli::run_registry_add(&formatter, &url),
            RegistryAction::Remove { url } => cli::run_registry_remove(&formatter, &url),
            RegistryAction::List => cli::run_registry_list(&formatter),
        },
        Commands::Issue { kind } => cli::run_issue(&kind),
        Commands::Publish { file, pr } => cli::run_publish(&formatter, &file, pr).await,
        Commands::Export {
            output,
            include_envs,
        } => cli::run_export(&formatter, &output, include_envs).await,
        Commands::Import {
            file,
            dry_run,
            force,
        } => cli::run_import(&formatter, &file, dry_run, force).await,
        Commands::Tui => {
            if let Err(e) = tui2::run_tui(&formatter).await {
                eprintln!("TUI error: {}", e);
            }
        }
        Commands::Doctor => cli::run_doctor(&formatter).await,
        Commands::Freeze { output } => cli::run_freeze(&formatter, &output).await,
        Commands::SelfUpdate => cli::run_self_update(&formatter).await,
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let shell_name = shell.as_str();
            let shell = match shell_name {
                "bash" => clap_complete::Shell::Bash,
                "zsh" => clap_complete::Shell::Zsh,
                "powershell" => clap_complete::Shell::PowerShell,
                "fish" => clap_complete::Shell::Fish,
                "elvish" => clap_complete::Shell::Elvish,
                other => {
                    eprintln!(
                        "[ERROR] 不支持的 shell: {} (支持: bash, zsh, powershell, fish, elvish)",
                        other
                    );
                    return;
                }
            };
            clap_complete::generate(shell, &mut cmd, "oneinit", &mut std::io::stdout());
        }
        Commands::Skill { action } => match action {
            SkillAction::Install { target } => cli::run_skill_install(&formatter, &target).await,
            SkillAction::List => cli::run_skill_list(&formatter).await,
            SkillAction::Status => cli::run_skill_status(&formatter).await,
            SkillAction::Uninstall => cli::run_skill_uninstall(&formatter).await,
        },
        Commands::Recipe { action } => match action {
            RecipeAction::New { name } => cli::run_recipe_new(&formatter, &name),
        },
        Commands::Team { action } => match action {
            TeamAction::Add {
                url,
                branch,
                force,
                allow_exec,
            } => {
                cli::run_team_add(&formatter, &url, &branch, force, allow_exec).await;
            }
            TeamAction::Remove => cli::run_team_remove(&formatter),
            TeamAction::Status => cli::run_team_status(&formatter),
            TeamAction::Sync {
                force,
                allow_exec,
                dry_run,
            } => {
                cli::run_team_sync(&formatter, force, allow_exec, dry_run).await;
            }
        },
        Commands::Viz {
            html,
            issue,
            output,
            open,
            no_scan,
        } => {
            cli::run_viz(&formatter, html, issue, output.as_deref(), open, no_scan).await;
        }
    }

    // 任何命令输出过错误 → 非零退出码（AI 自动化依赖失败信号）
    if formatter.had_errors() {
        std::process::exit(1);
    }
}

/// 每次使用自动拉取配方索引
///
/// 仅当缓存缺失或过期（>24h）时拉取，失败静默（不阻塞主命令）。
async fn maybe_auto_update(formatter: &output::OutputFormatter) {
    use core::registry;

    if registry::load_cached_index().is_some() && !registry::is_index_stale(24) {
        return; // 缓存新鲜，跳过
    }

    formatter.output(
        "[AUTO] Refreshing recipe index...",
        Some(serde_json::json!({ "status": "auto_update", "action": "update" })),
    );

    if let Err(e) = registry::fetch_index().await {
        // 静默失败，不阻塞主命令；JSON 模式下输出错误供 AI 参考
        formatter.output(
            &format!("[WARN] 配方索引刷新失败: {}", e),
            Some(serde_json::json!({
                "status": "warning",
                "action": "auto_update",
                "error": e.to_string(),
            })),
        );
    }
}
