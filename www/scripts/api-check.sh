#!/usr/bin/env bash
# 一键检查 oneinit.bg4jts.cn 的 API / 页面是否正常
#
# 用法:
#   bash scripts/api-check.sh                # 检查线上站点
#   BASE=https://oneinit.bg4jts.cn bash scripts/api-check.sh
#
# 退出码: 0 = 全部通过, 1 = 有失败项

set -u
BASE="${BASE:-https://oneinit.bg4jts.cn}"

pass=0
fail=0

check() {
  local name="$1" url="$2" expect="$3"
  local code
  code=$(curl -s -o /tmp/api-check-body -w "%{http_code}" --max-time 20 "$url" 2>/dev/null)
  if [ "$code" = "$expect" ]; then
    echo "  [OK]   $name  → HTTP $code"
    pass=$((pass + 1))
  else
    echo "  [FAIL] $name  → HTTP ${code:-无响应} (期望 $expect)"
    head -c 200 /tmp/api-check-body 2>/dev/null; echo
    fail=$((fail + 1))
  fi
}

echo "== oneinit.bg4jts.cn API 检查 =="
echo "BASE: $BASE"
echo

# 1. 健康检查（期望 200，且 body 里 ok=true）
check "GET /api/v1/health        (健康)" "$BASE/api/v1/health" "200"
if curl -s --max-time 20 "$BASE/api/v1/health" 2>/dev/null | grep -q '"ok":true'; then
  echo "  [OK]   health 依赖项全部正常 (ok:true)"
  pass=$((pass + 1))
else
  echo "  [FAIL] health ok 不为 true（依赖降级）"
  fail=$((fail + 1))
fi

# 2. 统计接口（期望 200，且含 total_recipes）
check "GET /api/v1/stats          (统计)" "$BASE/api/v1/stats" "200"
if curl -s --max-time 20 "$BASE/api/v1/stats" 2>/dev/null | grep -q '"total_recipes"'; then
  echo "  [OK]   stats 返回 total_recipes"
  pass=$((pass + 1))
else
  echo "  [FAIL] stats 缺少 total_recipes"
  fail=$((fail + 1))
fi

# 3. 配方目录（期望 200，且含 recipes 数组）
check "GET /api/v1/recipes        (目录)" "$BASE/api/v1/recipes" "200"
if curl -s --max-time 20 "$BASE/api/v1/recipes" 2>/dev/null | grep -q '"recipes"'; then
  echo "  [OK]   recipes 返回目录"
  pass=$((pass + 1))
else
  echo "  [FAIL] recipes 缺少 recipes 字段"
  fail=$((fail + 1))
fi

# 4. 上传接口校验（非法 YAML → 期望 400）
code=$(printf 'foo: bar\n' | curl -s -o /tmp/api-check-body -w "%{http_code}" --max-time 20 \
  -X POST "$BASE/api/v1/recipes" -H "Content-Type: application/yaml" --data-binary @- 2>/dev/null)
if [ "$code" = "400" ]; then
  echo "  [OK]   POST /api/v1/recipes 非法输入 → 400"
  pass=$((pass + 1))
else
  echo "  [FAIL] POST 非法输入 → HTTP ${code:-无响应} (期望 400) — 提示: 若为 503 说明服务端未配置 GITHUB_TOKEN"
  head -c 200 /tmp/api-check-body 2>/dev/null; echo
  fail=$((fail + 1))
fi

# 5. 展示页（期望 200）
check "GET /recipes                (展示页)" "$BASE/recipes" "200"

echo
echo "== 结果: $pass 通过, $fail 失败 =="
[ "$fail" -eq 0 ]
