mod cli;
mod core;
mod output;

use core::Result;
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let formatter = output::OutputFormatter::new(cli.json);

    match cli.command {
        Commands::Init { preset } => cli::run_init(&formatter, preset.as_deref()),
        Commands::Install { package } => cli::run_install(&formatter, &package),
        Commands::Uninstall { package } => cli::run_uninstall(&formatter, &package),
        Commands::List => cli::run_list(&formatter),
        Commands::Search { keyword } => cli::run_search(&formatter, keyword.as_deref()),
        Commands::Sync => cli::run_sync(&formatter),
    }
}
