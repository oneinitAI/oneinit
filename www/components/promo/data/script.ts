export interface ScriptLine {
  type: "user" | "ai" | "terminal" | "error" | "success" | "system";
  prefix?: string;
  text: string;
  speed?: number;
  delay?: number;
}

export interface Scene {
  id: string;
  lines: ScriptLine[];
  lineInterval: number;
}

export const scenes: Scene[] = [
  // Phase 1: New machine, no Python at all
  {
    id: "phase1-no-python",
    lineInterval: 800,
    lines: [
      { type: "user", text: "帮我写一个数据分析脚本，读 CSV 画图表", speed: 28 },
      {
        type: "ai",
        text: `好的，这里用 **pandas + matplotlib** 读取数据并生成折线图：

\`\`\`python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("data.csv")
df.plot(x="date", y="value", kind="line")
plt.title("Data Analysis")
plt.savefig("output.png")
print("Chart saved!")
\`\`\`

保存为 \`analysis.py\`，用 \`python analysis.py\` 运行即可。`,
        speed: 4,
        delay: 500,
      },
      {
        type: "terminal",
        text: "> python analysis.py",
        speed: 25,
        delay: 600,
      },
      {
        type: "error",
        text: `\`\`\`
'python' is not recognized as an internal
or external command, operable program
or batch file.
\`\`\``,
        speed: 8,
        delay: 500,
      },
      { type: "user", text: "运行不了... 提示 python 不是命令", speed: 25, delay: 400 },
      {
        type: "ai",
        text: `你的机器上**没有安装 Python**。请先前往 [python.org](https://python.org) 下载安装包。

安装时记得勾选 **"Add Python to PATH"**。`,
        speed: 18,
        delay: 300,
      },
    ],
  },

  // Phase 2: Manual Python installation — PATH nightmare
  {
    id: "phase2-manual-install",
    lineInterval: 750,
    lines: [
      {
        type: "system",
        text: "Step 1: 打开浏览器 -> python.org -> 找到 Downloads 页面",
        speed: 12,
        delay: 400,
      },
      {
        type: "system",
        text: "Step 2: 下载 Python 安装包 (25 MB)... 等待中...",
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: "Step 3: 运行安装程序 -> Next -> Next -> Next -> Finish",
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: "漏掉了: 没勾选 Add Python to PATH",
        speed: 15,
        delay: 600,
      },
      {
        type: "terminal",
        text: "> python --version",
        speed: 25,
        delay: 400,
      },
      {
        type: "error",
        text: `\`\`\`
'python' is not recognized as an internal
or external command, operable program
or batch file.
\`\`\``,
        speed: 8,
        delay: 500,
      },
      { type: "user", text: "装完了啊？为什么还是不行？", speed: 22, delay: 400 },
      {
        type: "ai",
        text: `安装时**没有勾选 Add Python to PATH**。现在需要手动配置系统环境变量。`,
        speed: 15,
        delay: 300,
      },
      {
        type: "system",
        text: "控制面板 -> 系统 -> 高级系统设置 -> 环境变量",
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: "系统变量 Path -> 编辑 -> 新建 -> 粘贴 C:\\\\Python312",
        speed: 8,
        delay: 400,
      },
      {
        type: "system",
        text: "再新建一个 -> 粘贴 C:\\\\Python312\\\\Scripts -> 确定 -> 确定 -> 确定",
        speed: 8,
        delay: 400,
      },
      {
        type: "terminal",
        text: "> python --version",
        speed: 25,
        delay: 400,
      },
      { type: "success", text: "Python 3.12.5", speed: 20, delay: 300 },
      { type: "user", text: "...花了 20 分钟，就为了装一个 Python", speed: 18, delay: 600 },
    ],
  },

  // Phase 3: pip is unbearably slow — mirror configuration pain
  {
    id: "phase3-pip-slow",
    lineInterval: 700,
    lines: [
      {
        type: "system",
        text: "现在需要安装项目依赖: pandas, matplotlib",
        speed: 15,
        delay: 400,
      },
      {
        type: "terminal",
        text: "> pip install pandas matplotlib",
        speed: 22,
        delay: 400,
      },
      {
        type: "terminal",
        text: "Downloading... 20 KB/s | ETA: 15 min | Timeout risk: HIGH",
        speed: 12,
        delay: 600,
      },
      { type: "user", text: "下载速度只有 20KB/s ？这要等到什么时候", speed: 20, delay: 400 },
      {
        type: "ai",
        text: `默认 pip 走 **pypi.org** 官方源，国内访问极慢。你需要手动配置清华镜像源。

创建文件 \`%APPDATA%\\pip\\pip.ini\`，写入配置。`,
        speed: 12,
        delay: 400,
      },
      {
        type: "system",
        text: "Step 1: 打开文件管理器 -> 地址栏输入 %APPDATA%",
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: "Step 2: 新建文件夹 pip -> 进入 -> 新建文件 pip.ini",
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: "Step 3: 用记事本打开 -> 粘贴镜像配置 -> 保存 -> 关闭",
        speed: 10,
        delay: 400,
      },
      { type: "user", text: "PATH要手动配，pip镜像要手动写配置文件... 我只是想写个脚本而已", speed: 16, delay: 500 },
      { type: "user", text: "不搞了。环境配置比写代码难一百倍", speed: 18, delay: 600 },
    ],
  },

  // Phase 4: OneInit to the rescue
  {
    id: "phase4-oneinit",
    lineInterval: 500,
    lines: [
      {
        type: "terminal",
        text: "> oneinit install python3.12",
        speed: 22,
        delay: 600,
      },
      { type: "success", text: "[OK] Python 3.12.5 installed", speed: 18, delay: 200 },
      { type: "success", text: "[OK] PATH configured (user-level, no admin)", speed: 18, delay: 150 },
      { type: "success", text: "[OK] pip mirror set to Tsinghua mirror", speed: 18, delay: 150 },
      { type: "success", text: "[OK] pip 24.0 ready", speed: 18, delay: 150 },
      {
        type: "terminal",
        text: "> python analysis.py",
        speed: 22,
        delay: 400,
      },
      {
        type: "success",
        text: "Chart saved! output.png generated.",
        speed: 18,
        delay: 300,
      },
      {
        type: "system",
        text: "OneInit is AI-native. Every command supports --json output for AI agents.",
        speed: 12,
        delay: 400,
      },
      {
        type: "user",
        text: "帮我在这台机器上装好 Python 开发环境",
        speed: 25,
        delay: 500,
      },
      {
        type: "ai",
        text: `\`\`\`json
oneinit install python3.12 --json
→ {"status":"success","package":"python3.12",
   "path":"~/.oneinit/envs/python3.12",
   "mirror":"tsinghua","duration_ms":3420}
\`\`\`

Python 3.12.5 已安装，PATH 已配置，pip 已就绪。`,
        speed: 12,
        delay: 300,
      },
      {
        type: "user",
        text: "把我的开发环境导出，我要换新电脑",
        speed: 25,
        delay: 500,
      },
      {
        type: "ai",
        text: `\`\`\`json
oneinit capture --json
→ {"python":"3.12.5","node":"20.15.0",
   "git":"2.45.0","rust":"1.80.0"}

oneinit export --include-envs -o backup.tar.gz
→ {"file":"backup.tar.gz","size":"156MB",
   "tools":4,"status":"success"}
\`\`\`

环境已导出。新电脑上运行 \`oneinit import backup.tar.gz\` 即可恢复。`,
        speed: 10,
        delay: 400,
      },
      {
        type: "system",
        text: `OneInit also supports:
- oneinit init --preset python    (batch install Python stack)
- oneinit init --preset full      (Python + Node + Rust + Go + Git + Docker)
- oneinit capture                 (scan current environment)
- oneinit export / import         (migrate between machines)
- All commands support --json for AI agent integration`,
        speed: 6,
        delay: 500,
      },
    ],
  },
];
