#!/bin/bash

# API 测试脚本
# 确保服务器运行在 http://localhost:8080

BASE_URL="http://localhost:8080"

echo "=== 测试游戏服务器 API ==="
echo ""

# 1. 健康检查
echo "1. 健康检查"
curl -s "$BASE_URL/health" | jq
echo ""

# 2. 注册新用户
echo "2. 注册新用户"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testplayer",
    "email": "test@example.com",
    "password": "password123"
  }')
echo "$REGISTER_RESPONSE" | jq
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.token')
echo ""

# 3. 登录
echo "3. 用户登录"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "testplayer",
    "password": "password123"
  }')
echo "$LOGIN_RESPONSE" | jq
echo ""

# 4. 获取当前用户信息
echo "4. 获取当前用户信息"
curl -s -X GET "$BASE_URL/api/me" \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# 5. 创建游戏会话
echo "5. 创建游戏会话"
GAME_SESSION=$(curl -s -X POST "$BASE_URL/api/game/session" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "game_type": "puzzle"
  }')
echo "$GAME_SESSION" | jq
SESSION_ID=$(echo "$GAME_SESSION" | jq -r '.id')
echo ""

# 6. 更新游戏分数
echo "6. 更新游戏分数"
curl -s -X PUT "$BASE_URL/api/game/session/$SESSION_ID/score" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "score": 1500
  }' | jq
echo ""

# 7. 结束游戏会话
echo "7. 结束游戏会话"
curl -s -X POST "$BASE_URL/api/game/session/$SESSION_ID/end" \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

# 8. 获取排行榜
echo "8. 获取排行榜"
curl -s -X GET "$BASE_URL/api/leaderboard" | jq
echo ""

echo "=== API 测试完成 ==="