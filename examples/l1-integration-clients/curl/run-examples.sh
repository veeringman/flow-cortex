#!/bin/bash
#
# FlowCortex L1 cURL Examples
# Quick reference for testing FlowCortex L1 REST API
#

BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"

echo "FlowCortex L1 cURL Examples"
echo "Base URL: $BASE_URL"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper function to pretty-print JSON
pretty_json() {
  echo "$1" | python3 -m json.tool 2>/dev/null || echo "$1"
}

# 1. Create Account
echo -e "${BLUE}1. Create Account${NC}"
RESPONSE=$(curl -s -X POST "$BASE_URL/account" \
  -H "Content-Type: application/json" \
  -d '{"account":"alice"}')
pretty_json "$RESPONSE"
echo ""

# 2. Get Balance
echo -e "${BLUE}2. Get Balance${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/balance/admin/Proof")
pretty_json "$RESPONSE"
echo ""

# 3. Get FloweR Balance
echo -e "${BLUE}3. Get FloweR Balance${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/balance/admin/FloweR")
pretty_json "$RESPONSE"
echo ""

# 4. Mint Tokens
echo -e "${BLUE}4. Mint Tokens${NC}"
RESPONSE=$(curl -s -X POST "$BASE_URL/mint" \
  -H "Content-Type: application/json" \
  -d '{
    "caller":"admin",
    "to":"alice",
    "token":"proof",
    "amount":1000
  }')
pretty_json "$RESPONSE"
echo ""

# 5. Transfer Tokens
echo -e "${BLUE}5. Transfer Tokens${NC}"
RESPONSE=$(curl -s -X POST "$BASE_URL/transfer" \
  -H "Content-Type: application/json" \
  -d '{
    "from":"alice",
    "to":"bob",
    "token":"proof",
    "amount":100
  }')
pretty_json "$RESPONSE"
echo ""

# 6. Get Pool
echo -e "${BLUE}6. Get Pending Transaction Pool${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/pool")
pretty_json "$RESPONSE"
echo ""

# 7. List Blocks
echo -e "${BLUE}7. List Blocks${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/blocks")
pretty_json "$RESPONSE"
echo ""

# 8. Create Block
echo -e "${BLUE}8. Create Block${NC}"
RESPONSE=$(curl -s -X POST "$BASE_URL/block")
pretty_json "$RESPONSE"
echo ""

# 9. Get Snapshot
echo -e "${BLUE}9. Get Snapshot${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/snapshot")
pretty_json "$RESPONSE"
echo ""

# 10. List Anchors
echo -e "${BLUE}10. List Anchors${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/anchors")
pretty_json "$RESPONSE"
echo ""

# 11. Upload Capsule
echo -e "${BLUE}11. Upload Capsule${NC}"
# Create sample base64 encoded code
SAMPLE_CODE=$(echo -n "sample capsule code" | base64)
RESPONSE=$(curl -s -X POST "$BASE_URL/capsule" \
  -H "Content-Type: application/json" \
  -d "{
    \"id\":\"my_capsule_1\",
    \"code\":\"$SAMPLE_CODE\"
  }")
pretty_json "$RESPONSE"
echo ""

# 12. List Capsules
echo -e "${BLUE}12. List Capsules${NC}"
RESPONSE=$(curl -s -X GET "$BASE_URL/capsule")
pretty_json "$RESPONSE"
echo ""

# 13. Get Anchor (if available)
echo -e "${BLUE}13. Get First Anchor (if available)${NC}"
ANCHORS=$(curl -s -X GET "$BASE_URL/anchors")
ANCHOR_ID=$(echo "$ANCHORS" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['anchors'][0] if data.get('anchors') else '')" 2>/dev/null)
if [ -n "$ANCHOR_ID" ]; then
  RESPONSE=$(curl -s -X GET "$BASE_URL/anchor/$ANCHOR_ID")
  pretty_json "$RESPONSE"
else
  echo "No anchors available"
fi
echo ""

echo -e "${GREEN}All examples completed!${NC}"
