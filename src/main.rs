mod cli;
mod core;
mod output;
mod tui2;

use clap::{Parser, Subcommand};

/// OneInit — 一条命令，初始化整台电脑
///
/// 拿到一台新电脑后，第一个要装的工具。
/// 装完它，这台电脑就是开发者就绪的机器。
#[derive(Parser)]
#[command(name = "oneinit", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// 全局开关：所有命令输出 JSON 格式（AI 模式）
    #[arg(global = true, long = "json", help = "Output in JSON format (AI-friendly)")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 一键初始化开发环境
    Init {
        /// 预置套装名称（如 "python", "frontend", "ai"）
        #[arg(short, long)]
        preset: Option<String>,
    },

    /// 安装指定工具（如 python3.7, node18）
    Install {
        /// 要安装的工具包名称
        package: String,
    },

    /// 卸载指定工具
    Uninstall {
        /// 要卸载的工具包名称
        package: String,
    },

    /// 列出已安装的工具
    List,

    /// 搜索可用工具
    Search {
        /// 搜索关键词
        keyword: Option<String>,
    },

    /// 从 oneinit.yaml 同步环境
    Sync,

    /// 验证社区配方文件
    Verify {
        /// 配方文件路径
        file: String,
    },

    /// 启动交互式 TUI 界面
    Tui,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let formatter = output::OutputFormatter::new(cli.json);

    match cli.command {
        Commands::Init { preset } => cli::run_init(&formatter, preset.as_deref()).await,
        Commands::Install { package } => cli::run_install(&formatter, &package).await,
        Commands::Uninstall { package } => cli::run_uninstall(&formatter, &package).await,
        Commands::List => cli::run_list(&formatter).await,
        Commands::Search { keyword } => cli::run_search(&formatter, keyword.as_deref()).await,
        Commands::Sync => cli::run_sync(&formatter).await,
        Commands::Verify { file } => cli::run_verify(&formatter, &file).await,
        Commands::Tui => {
            if let Err(e) = tui2::run_tui(&formatter).await {
                eprintln!("TUI 错误: {}", e);
            }
        }
    }
}
