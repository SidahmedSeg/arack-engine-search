#!/bin/bash
# Zitadel Actions V2 Configuration Script
# This script sets up event-based webhooks for user.human.added events
#
# Phase 3: Actions V2 Migration - Fixes the bug where admin-created users
# don't trigger Actions V1 "Post Creation" hooks

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Zitadel Actions V2 Configuration Script${NC}"
echo -e "${BLUE}  Event-based webhooks for user.human.added${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Configuration
ZITADEL_DOMAIN="auth.arack.io"
ZITADEL_URL="https://${ZITADEL_DOMAIN}"
WEBHOOK_ENDPOINT="http://search-service:3000/internal/auth/zitadel/v2/user-created"

# Check if token is provided
if [ -z "$1" ]; then
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}  How to Get Your Admin Token${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "1. Go to Zitadel Console: https://auth.arack.io/ui/console"
    echo "2. Login with your admin account"
    echo "3. Navigate to: Organization → Service Users"
    echo "4. Click on your service user (or create one if needed)"
    echo "5. Click 'Generate Key' → Download the JSON"
    echo "6. Extract the token from the JSON and run:"
    echo ""
    echo -e "${GREEN}   ./setup_zitadel_actions_v2.sh YOUR_TOKEN_HERE${NC}"
    echo ""
    echo "OR generate a PAT (Personal Access Token):"
    echo ""
    echo "1. Go to: User → Personal Access Tokens"
    echo "2. Click 'New'"
    echo "3. Set expiration and click 'Add'"
    echo "4. Copy the token and run:"
    echo ""
    echo -e "${GREEN}   ./setup_zitadel_actions_v2.sh YOUR_PAT_HERE${NC}"
    echo ""
    exit 1
fi

ADMIN_TOKEN="$1"

echo -e "${GREEN}✓${NC} Admin token provided"
echo ""

# Step 1: Create Target (Webhook Endpoint)
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Step 1: Creating Target (Webhook Endpoint)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Endpoint: $WEBHOOK_ENDPOINT"
echo ""

TARGET_RESPONSE=$(curl -s -X POST "${ZITADEL_URL}/v2beta/actions/targets" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  --data-raw '{
    "name": "Search Service V2 Webhook",
    "restWebhook": {
      "interruptOnError": false
    },
    "endpoint": "'"${WEBHOOK_ENDPOINT}"'",
    "timeout": "10s"
  }')

# Check if target creation was successful
if echo "$TARGET_RESPONSE" | grep -q '"id"'; then
    TARGET_ID=$(echo "$TARGET_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${GREEN}✓${NC} Target created successfully"
    echo "  Target ID: $TARGET_ID"
    echo ""
else
    echo -e "${RED}✗${NC} Failed to create target"
    echo "Response: $TARGET_RESPONSE"
    exit 1
fi

# Step 2: Create Execution for user.human.added event
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Step 2: Creating Execution (user.human.added)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Event: user.human.added"
echo "Target ID: $TARGET_ID"
echo ""

EXECUTION_RESPONSE=$(curl -s -X POST "${ZITADEL_URL}/v2beta/actions/executions" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  --data-raw '{
    "condition": {
      "event": {
        "event": "user.human.added"
      }
    },
    "targets": ["'"${TARGET_ID}"'"]
  }')

# Check if execution creation was successful
if echo "$EXECUTION_RESPONSE" | grep -q '"targets"'; then
    echo -e "${GREEN}✓${NC} Execution created successfully"
    echo ""
else
    echo -e "${RED}✗${NC} Failed to create execution"
    echo "Response: $EXECUTION_RESPONSE"

    # Even if execution fails, we should show the target ID for manual setup
    echo ""
    echo -e "${YELLOW}Target was created successfully with ID: $TARGET_ID${NC}"
    echo "You can manually configure the execution in Zitadel Console"
    exit 1
fi

# Success summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✓ Zitadel Actions V2 Configuration Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Configuration Summary:"
echo "  • Target ID: $TARGET_ID"
echo "  • Endpoint: $WEBHOOK_ENDPOINT"
echo "  • Event: user.human.added"
echo "  • Interrupt on Error: false"
echo "  • Timeout: 10s"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "  1. Deploy the updated backend with V2 webhook handler"
echo "  2. Create a test user in Zitadel Console"
echo "  3. Verify webhook is called and user_preferences created"
echo ""
echo -e "${BLUE}Test Commands:${NC}"
echo ""
echo "  # Check webhook logs"
echo "  docker logs search_engine_search_service --tail 50 | grep 'Zitadel Actions V2'"
echo ""
echo "  # Verify database record"
echo "  docker exec search_engine_postgres psql -U postgres -d engine_search \\"
echo "    -c \"SELECT zitadel_user_id, username FROM user_preferences WHERE zitadel_user_id IS NOT NULL ORDER BY created_at DESC LIMIT 3;\""
echo ""
echo -e "${GREEN}Actions V2 is now configured and will trigger for ALL user creation methods!${NC}"
echo ""
