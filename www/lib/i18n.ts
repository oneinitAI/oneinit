// 中英文字典 — 官网 i18n
// 用法: const { t, lang, setLang } = useLang();  t("hero.title")

export type Lang = "en" | "zh";

type Dict = Record<string, { en: string; zh: string }>;

export const dict: Dict = {
  // ===== Nav =====
  "nav.sponsor": { en: "Sponsor", zh: "赞助" },
  "nav.npm": { en: "npm", zh: "npm" },
  "nav.github": { en: "GitHub", zh: "GitHub" },

  // ===== Hero =====
  "hero.badge": { en: "AI-FIRST ENVIRONMENT INITIALIZER", zh: "AI 原生环境初始化工具" },
  "hero.title1": { en: "One command", zh: "一条命令" },
  "hero.title2": { en: "to init your", zh: "搞定你的" },
  "hero.title3": { en: "dev machine.", zh: "开发环境。" },
  "hero.subtitle": {
    en: "Python, Node.js, Rust, Go — installed, mirrored, PATH-configured. All in one line. Zero sudo. SQLite manifest for clean rollback.",
    zh: "Python、Node.js、Rust、Go —— 安装、镜像源、PATH 一条命令全部搞定。零 sudo，SQLite 清单精准回滚。",
  },
  "hero.cta": { en: "Get Started", zh: "开始使用" },
  "hero.npm": { en: "npm i -g oneinit", zh: "npm i -g oneinit" },

  // ===== Stats =====
  "stats.commands": { en: "CLI Commands", zh: "CLI 命令" },
  "stats.detectors": { en: "Language Detectors", zh: "语言检测器" },
  "stats.tests": { en: "Unit Tests", zh: "单元测试" },
  "stats.size": { en: "Binary Size", zh: "二进制体积" },

  // ===== AI Ready =====
  "aiready.badge": { en: "AI-NATIVE", zh: "AI 原生" },
  "aiready.title1": { en: "Built for", zh: "为" },
  "aiready.title2": { en: "AI agents", zh: "AI 智能体" },
  "aiready.title3": { en: "Not just developers.", zh: "而生，不止于开发者。" },
  "aiready.desc": {
    en: "OneInit is the first environment initializer designed for the AI era. Every command has a JSON mode. Every operation is parseable. AI agents don't read terminal output — they read structured data.",
    zh: "OneInit 是首个为 AI 时代设计的环境初始化工具。每个命令都有 JSON 模式，每次操作都可解析。AI 智能体不读终端输出——它们读结构化数据。",
  },
  "aiready.c1t": { en: "JSON-First Output", zh: "JSON 优先输出" },
  "aiready.c1d": {
    en: "Every command supports --json. Structured output that AI agents parse directly. No text scraping needed.",
    zh: "所有命令支持 --json，AI 智能体可直接解析结构化输出，无需文本抓取。",
  },
  "aiready.c2t": { en: "AI Skill Installer", zh: "AI 技能安装器" },
  "aiready.c2d": {
    en: "One command installs the OneInit Skill into ZCode, Claude, Codex, and Cursor. AI agents can then install tools, capture environments, and migrate machines autonomously.",
    zh: "一条命令把 OneInit 技能装进 ZCode、Claude、Codex、Cursor。AI 智能体从此可以自主安装工具、捕获环境、迁移机器。",
  },
  "aiready.c3t": { en: "Agent Installation Guide", zh: "智能体安装指南" },
  "aiready.c3d": {
    en: "Give your AI assistant one prompt. It clones the repo, builds the binary, installs tools, and configures the Skill. Zero human intervention.",
    zh: "给你的 AI 助手一段提示词。它自动克隆仓库、编译二进制、安装工具、配置技能，全程无需人工干预。",
  },
  "aiready.c4t": { en: "AI Agent Autonomous DevOps", zh: "AI 智能体自动化运维" },
  "aiready.c4d": {
    en: "AI agents can capture environments, export backups, and restore on new machines. CI/CD pipelines can self-bootstrap with one command. Zero manual config.",
    zh: "AI 智能体可捕获环境、导出备份、在新机器上恢复。CI/CD 流水线一条命令自举，零手工配置。",
  },

  // ===== Story =====
  "story.badge": { en: "The Problem", zh: "痛点" },
  "story.title1": { en: "Installing Python", zh: "装个 Python" },
  "story.title2": { en: "should not take 30 minutes.", zh: "不该花 30 分钟。" },
  "story.without": { en: "Without OneInit", zh: "没有 OneInit" },
  "story.afternoon": { en: "The Developer's Afternoon", zh: "开发者的一下午" },
  "story.q1": { en: "Which Python version do I need?", zh: "该装哪个 Python 版本？" },
  "story.a1": { en: "Don't care. Just install.", zh: "不用纠结，直接装。" },
  "story.q2": { en: "Go to python.org → Downloads → find the right installer", zh: "打开 python.org → 下载页 → 找对的安装包" },
  "story.a2": { en: "Auto-detect OS + download.", zh: "自动识别系统 + 下载。" },
  "story.q3": { en: "Run .exe → check 'Add to PATH' → Next × 5", zh: "运行 .exe → 勾选「添加到 PATH」→ 下一步 × 5" },
  "story.a3": { en: "Handled. No checkboxes.", zh: "全自动，没有复选框。" },
  "story.q4": { en: "Python works. Now how do I install pip?", zh: "Python 好了，pip 怎么装？" },
  "story.a4": { en: "Bundled. get-pip auto-bootstrap.", zh: "内置。get-pip 自动引导。" },
  "story.q5": { en: "pip is slow. Google 'pip mirror' → edit pip.ini", zh: "pip 太慢。搜「pip 镜像」→ 改 pip.ini" },
  "story.a5": { en: "Tsinghua mirror auto-configured.", zh: "清华源自动配置好。" },
  "story.q6": { en: "Did it actually work? Let me google how to check...", zh: "装好了吗？搜下怎么验证……" },
  "story.a6": { en: "Verified. SHA256, PATH, manifest all confirmed.", zh: "已验证。SHA256、PATH、清单全部确认。" },
  "story.q7": { en: "I messed up -- how do I uninstall cleanly?", zh: "装坏了——怎么干净卸载？" },
  "story.a7": { en: "oneinit uninstall. 100% rollback.", zh: "oneinit uninstall，100% 回滚。" },
  "story.timelost": {
    en: "Time lost: ~{n} minutes. Then you realize you forgot to configure the mirror...",
    zh: "耗时约 {n} 分钟。然后你发现忘了配镜像源……",
  },
  "story.summary1": { en: "~30 minutes. Every machine. Every time.", zh: "约 30 分钟。每台机器，每次都这样。" },
  "story.with": { en: "With OneInit", zh: "有了 OneInit" },
  "story.onecmd": { en: "One Command", zh: "一条命令" },
  "story.running": { en: "[OK] {cmd} — running...", zh: "[OK] {cmd} — 执行中..." },
  "story.done": { en: "[OK] Done. Your machine is developer-ready.", zh: "[OK] 完成。你的机器已开发就绪。" },
  "story.summary2": { en: "One command. Less than 30 seconds. Every machine.", zh: "一条命令，不到 30 秒。每台机器。" },

  // ===== Comparison =====
  "cmp.badge": { en: "Comparison", zh: "对比" },
  "cmp.title1": { en: "New machine = hours of setup.", zh: "新机器 = 数小时的配置。" },
  "cmp.title2": { en: "Until now.", zh: "直到现在。" },
  "cmp.trad": { en: "Traditional", zh: "传统方式" },
  "cmp.oneinit": { en: "OneInit", zh: "OneInit" },

  // ===== How It Works =====
  "hiw.badge": { en: "How It Works", zh: "工作原理" },
  "hiw.title1": { en: "Install, configure, rollback.", zh: "安装、配置、回滚，" },
  "hiw.title2": { en: "Clean.", zh: "全程干净。" },
  "hiw.s1t": { en: "Install", zh: "安装" },
  "hiw.s1d": {
    en: "Write a YAML recipe with the download URL, SHA256, and install steps. Drop it in ~/.oneinit/recipes/.",
    zh: "写一个 YAML 配方（下载地址 + SHA256 + 安装步骤），放进 ~/.oneinit/recipes/。",
  },
  "hiw.s2t": { en: "Verify", zh: "校验" },
  "hiw.s2d": {
    en: "OneInit downloads the archive, verifies the SHA256 checksum, and extracts it to a sandboxed directory.",
    zh: "OneInit 下载压缩包，校验 SHA256，解压到沙箱目录。",
  },
  "hiw.s3t": { en: "Configure", zh: "配置" },
  "hiw.s3d": {
    en: "Mirror sources are auto-configured. PATH entries are added. Config files are written. All tracked in SQLite.",
    zh: "镜像源自动配置、PATH 自动写入、配置文件自动生成，全部记录在 SQLite。",
  },
  "hiw.s4t": { en: "Rollback", zh: "回滚" },
  "hiw.s4d": {
    en: "Uninstall removes everything: PATH entries, config files, install directory, and manifest record. Complete.",
    zh: "卸载时彻底清除：PATH 条目、配置文件、安装目录、清单记录，干干净净。",
  },

  // ===== Community Recipe =====
  "cr.badge": { en: "Community Registry", zh: "社区配方库" },
  "cr.title1": { en: "Publish YAML.", zh: "发布 YAML，" },
  "cr.title2": { en: "Anyone installs.", zh: "人人都能装。" },
  "cr.desc": {
    en: "Like npm, but for dev tools. Write a recipe. Push to GitHub. One command installs it anywhere.",
    zh: "像 npm 一样，但面向开发工具。写一个配方，推到 GitHub，任何机器一条命令安装。",
  },
  "cr.security": {
    en: "Built-in security: every recipe is SHA256-verified before installation. Users confirm before anything runs.",
    zh: "内置安全机制：每个配方安装前都做 SHA256 校验，任何操作前用户需确认。",
  },

  // ===== Recipes (live) =====
  "rc.badge": { en: "SUPPORTED RECIPES · LIVE", zh: "支持配方 · 实时" },
  "rc.title1": { en: "One command.", zh: "一条命令，" },
  "rc.title2": { en: "{n} tools and counting.", zh: "{n} 个工具，持续增加。" },
  "rc.desc": {
    en: "The recipe list is fetched live from the registry. Builtin recipes work out of the box; remote recipes install with one command.",
    zh: "配方列表实时从注册表拉取。内置配方开箱即用，远程配方一条命令自动安装。",
  },
  "rc.builtin": { en: "BUILTIN", zh: "内置" },
  "rc.remote": { en: "REMOTE", zh: "远程" },
  "rc.fetching": { en: "fetching registry...", zh: "拉取注册表..." },
  "rc.err": { en: "registry unreachable — showing builtin recipes", zh: "注册表不可达——显示内置配方" },
  "rc.cta": {
    en: "Need more tools? Submit a recipe request →",
    zh: "需要更多工具？提交 recipe request →",
  },

  // ===== Features =====
  "feat.c1t": { en: "Auto Mirror Config", zh: "自动镜像源" },
  "feat.c1d": {
    en: "pip uses Tsinghua. npm uses npmmirror. No manual setup. No config files. Works on every install.",
    zh: "pip 走清华源，npm 走淘宝源。无需手动配置，每次安装自动生效。",
  },
  "feat.c2t": { en: "7 Language Detectors", zh: "7 种语言检测器" },
  "feat.c2d": {
    en: "Scan any machine: Python, Node, Git, Rust, Go, Java, Docker. Plus custom detectors via scan_config.yaml.",
    zh: "扫描任意机器：Python、Node、Git、Rust、Go、Java、Docker，还支持 scan_config.yaml 自定义检测器。",
  },
  "feat.c3t": { en: "Environment Migration", zh: "环境迁移" },
  "feat.c3d": {
    en: "Export your entire setup as tar.gz. Import on a new machine. Tools, configs, package lists — everything restored.",
    zh: "整个环境导出为 tar.gz，新机器一键导入。工具、配置、依赖清单全部恢复。",
  },
  "feat.c4t": { en: "Clean Uninstall", zh: "干净卸载" },
  "feat.c4d": {
    en: "SQLite manifest tracks every PATH entry and config file. Uninstall removes everything. No leftovers. No guessing.",
    zh: "SQLite 清单记录每一次 PATH 修改和配置文件。卸载清除一切，无残留、无猜测。",
  },

  // ===== Install Bar =====
  "ib.badge": { en: "Install", zh: "安装" },
  "ib.title1": { en: "One line.", zh: "一行，" },
  "ib.title2": { en: "Done.", zh: "搞定。" },
  "ib.desc": { en: "Pick your method. All install the same binary.", zh: "任选一种方式，安装的是同一个二进制。" },
  "ib.copy": { en: "copy", zh: "复制" },
  "ib.copied": { en: "copied!", zh: "已复制！" },
  "ib.shell": { en: "Shell", zh: "Shell" },
  "ib.npm": { en: "npm", zh: "npm" },
  "ib.source": { en: "Source", zh: "源码" },
  "ib.shell_n": {
    en: "Zero prerequisites. Auto-detects OS and architecture.",
    zh: "零前置依赖，自动识别系统和架构。",
  },
  "ib.npm_n": { en: "Node.js 14+. PATH handled automatically.", zh: "需 Node.js 14+，PATH 自动处理。" },
  "ib.source_n": {
    en: "Rust 1.94+. Binary at target/release/oneinit.",
    zh: "需 Rust 1.94+，产物在 target/release/oneinit。",
  },

  // ===== AI Install =====
  "ai.badge": { en: "AI Install", zh: "AI 安装" },
  "ai.title1": { en: "Let AI do it.", zh: "让 AI 来装。" },
  "ai.title2": { en: "Copy. Paste. Done.", zh: "复制、粘贴、完成。" },
  "ai.desc": {
    en: "Don't want to open a terminal? Ask ChatGPT, Claude, or ZCode to install OneInit for you. One prompt is all it takes.",
    zh: "不想打开终端？让 ChatGPT、Claude 或 ZCode 帮你安装 OneInit。一段提示词就够了。",
  },
  "ai.copy": { en: "copy", zh: "复制" },
  "ai.copied": { en: "copied!", zh: "已复制！" },
  "ai.works": {
    en: "Works with ChatGPT · Claude · ZCode · Copilot · any AI assistant",
    zh: "支持 ChatGPT · Claude · ZCode · Copilot 等任意 AI 助手",
  },

  // ===== Footer =====
  "ft.title1": { en: "One command to", zh: "一条命令" },
  "ft.title2": { en: "init your dev machine", zh: "搞定你的开发环境" },
  "ft.stats": { en: "17 commands. 7 detectors. 26 tests. 7.3MB. Zero runtime.", zh: "17 个命令 · 7 个检测器 · 26 个测试 · 7.3MB · 零运行时" },
  "ft.cta": { en: "Get Started", zh: "开始使用" },
  "ft.github": { en: "View on GitHub", zh: "查看 GitHub" },
  "ft.built": { en: "Built with Rust · No runtime · Single binary", zh: "Rust 构建 · 零运行时 · 单二进制" },
  "ft.support": { en: "💚 Support OneInit — it's made by one person", zh: "💚 支持 OneInit —— 一个人的开源项目" },
  "ft.terms": { en: "Terms of Service", zh: "用户协议" },

  // ===== Terms of Service =====
  "terms.title": { en: "Terms of Service", zh: "用户协议" },
  "terms.updated": { en: "Last updated: August 2026", zh: "最后更新：2026 年 8 月" },
  "terms.p1": { en: "By using OneInit you agree to the following terms.", zh: "使用 OneInit 即表示你同意以下条款。" },
  "terms.s1t": { en: "Automation guidance only", zh: "仅提供自动化指引" },
  "terms.s1d": {
    en: "OneInit automates downloading and installing software from the URLs declared in recipes. It does not host, store, redistribute, or endorse any software copies.",
    zh: "OneInit 仅按配方声明的下载地址自动下载和安装软件。它不托管、存储、分发或背书任何软件副本。",
  },
  "terms.s2t": { en: "Your responsibility", zh: "责任自负" },
  "terms.s2d": {
    en: "You are solely responsible for the software you install and for complying with its license, copyright, and local laws. OneInit is not a party to your relationship with the software publishers.",
    zh: "你需对自己安装的软件负责，并遵守其许可证、版权及相关法律法规。OneInit 不是你和软件发布方之间的协议当事方。",
  },
  "terms.s3t": { en: "Respect licenses", zh: "尊重许可证" },
  "terms.s3d": {
    en: "Before installing, review the license / license_url shown in the [SECURITY] confirmation prompt. If a tool's license does not permit the intended use, do not install it.",
    zh: "安装前请查看 [SECURITY] 确认提示中的 license / license_url。如果某个工具的许可证不允许你的预期用途，请勿安装。",
  },
  "terms.back": { en: "← Back to home", zh: "← 返回首页" },

  // ===== Changelog =====
  "cl.title": { en: "Changelog", zh: "更新日志" },
  "cl.subtitle": {
    en: "Every release, straight from GitHub Releases.",
    zh: "每次版本更新，实时同步自 GitHub Releases。",
  },
  "cl.links": { en: "Links", zh: "链接" },
  "cl.npm": { en: "npm", zh: "npm" },
  "cl.releases": { en: "GitHub Releases", zh: "GitHub Releases" },
  "cl.home": { en: "Homepage", zh: "官网" },
  "cl.prerelease": { en: "Pre-release", zh: "预览版" },
  "cl.install": { en: "Install", zh: "安装" },
  "cl.changelog": { en: "Changelog", zh: "更新日志" },
  "cl.security": { en: "Security Fixes", zh: "安全修复" },
  "cl.fetching": { en: "Fetching releases...", zh: "拉取发行版..." },
  "cl.err": { en: "Failed to load releases — GitHub API rate limited?", zh: "加载失败——GitHub API 限流？" },
  "cl.empty": { en: "No releases yet.", zh: "暂无发行版。" },
  "cl.assets": { en: "Downloads", zh: "下载" },
  "cl.published": { en: "Published", zh: "发布于" },
  "cl.author": { en: "Author", zh: "作者" },
  "cl.by": { en: "by", zh: "由" },

  // ===== Credits / 致谢 =====
  "credit.line": {
    en: "Coded by a developer with the help of DeepSeek, GLM, ChatGPT & friends.",
    zh: "由程序员借助 DeepSeek、GLM、ChatGPT 等 AI 模型辅助完成。",
  },

  // ===== Hero changelog entry =====
  "hero.changelog": { en: "View Changelog", zh: "查看更新日志" },
};
