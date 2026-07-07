#!/bin/bash
# OneInit 开发环境设置
# 在 Git Bash 中使用: source dev.sh
#
# 解决 Git Bash 的 GNU link.exe 覆盖 MSVC link.exe 的问题

export MSVC_BIN="D:/Program Files/Microsoft Visual Studio/18/Enterprise/VC/Tools/MSVC/14.51.36231/bin/Hostx64/x64"

# 将 MSVC link.exe 放在 PATH 最前面，覆盖 Git 的 GNU link
export PATH="${MSVC_BIN}:$PATH"

echo "✅ OneInit 开发环境已加载 (MSVC toolchain)"
