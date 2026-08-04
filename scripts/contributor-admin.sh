#!/usr/bin/env bash
# 贡献者管理员命令（包装网站 API，替代页面管理面板）
#
# 用法（Windows 推荐直接传 --token，无需设环境变量）:
#   bash scripts/contributor-admin.sh add <login> --token <ADMIN_TOKEN> [--tags a,b,c] [--contrib N]
#   bash scripts/contributor-admin.sh remove <login> --token <ADMIN_TOKEN>
#   bash scripts/contributor-admin.sh list
#
# 兼容: --token 可放在任意位置；未传时回退读取 ADMIN_TOKEN 环境变量。
# 安全: 脚本不会保存/打印 token；请勿把 token 写进任何会被提交的文件。
#
# 依赖: curl（list 美化输出需要 python3 或 python）

set -u
BASE="${BASE:-https://oneinit.bg4jts.cn}"
TOKEN="${ADMIN_TOKEN:-}"

# 全局参数解析：--token 可在任意位置（子命令前后均可）
POS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --token)
      [ $# -ge 2 ] || { echo "错误: --token 需要一个值" >&2; exit 1; }
      TOKEN="$2"
      shift 2
      ;;
    *)
      POS+=("$1")
      shift
      ;;
  esac
done
set -- "${POS[@]}"

cmd="${1:-}"
login="${2:-}"

usage() {
  echo "用法:"
  echo "  contributor-admin.sh add <login> --token <ADMIN_TOKEN> [--tags a,b,c] [--contrib N]"
  echo "  contributor-admin.sh remove <login> --token <ADMIN_TOKEN>"
  echo "  contributor-admin.sh list"
  echo
  echo "参数: --token 管理令牌（也可用 ADMIN_TOKEN 环境变量）; BASE 环境变量可改站点（默认 https://oneinit.bg4jts.cn）"
  exit 1
}

# "a,b,c" -> ["a","b","c"]（不依赖 tr/sed 引号转义）
build_tags() {
  local tags="$1" first=1 t
  printf '['
  IFS=',' read -ra arr <<< "$tags"
  for t in "${arr[@]}"; do
    t="$(echo "$t" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    [ -n "$t" ] || continue
    [ "$first" = 0 ] && printf ','
    printf '"%s"' "$t"
    first=0
  done
  printf ']'
}

case "$cmd" in
  add)
    [ -z "$login" ] && usage
    if [ -z "$TOKEN" ]; then echo "错误: 请用 --token <ADMIN_TOKEN> 或设置 ADMIN_TOKEN 环境变量" >&2; exit 1; fi
    tags=""; contrib=""
    while [ $# -gt 2 ]; do
      case "$3" in
        --tags)   tags="$4";   shift 2 ;;
        --contrib) contrib="$4"; shift 2 ;;
        *) shift ;;
      esac
    done
    body="{\"login\":\"$login\""
    if [ -n "$tags" ]; then
      body="$body,\"tags\":$(build_tags "$tags")"
    fi
    if [ -n "$contrib" ]; then
      body="$body,\"contributions\":$contrib"
    fi
    body="$body}"
    curl -s -X POST "$BASE/api/v1/contributors" \
      -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "$body"
    echo
    ;;
  remove)
    [ -z "$login" ] && usage
    if [ -z "$TOKEN" ]; then echo "错误: 请用 --token <ADMIN_TOKEN> 或设置 ADMIN_TOKEN 环境变量" >&2; exit 1; fi
    curl -s -X DELETE "$BASE/api/v1/contributors/$login" -H "Authorization: Bearer $TOKEN"
    echo
    ;;
  list)
    # 找可用的 python（跳过 Windows Store 的 python3 stub：实际执行验证）
    py=""
    for c in python3 python; do
      if command -v "$c" >/dev/null 2>&1 && "$c" -c 'pass' >/dev/null 2>&1; then
        py="$c"
        break
      fi
    done
    if [ -n "$py" ]; then
      curl -s "$BASE/api/v1/contributors" | "$py" -c "
import json, sys
d = json.load(sys.stdin)
for i, c in enumerate(d.get('contributors', []), 1):
    tags = ','.join(c.get('tags', [])) or '-'
    print(f\"{i:>2}. {c['login']:<20} {c['contributions']:>5}  [{tags}]  {c['html_url']}\")
"
    else
      curl -s "$BASE/api/v1/contributors"
      echo
    fi
    ;;
  *)
    usage
    ;;
esac
