#!/bin/bash

# Migrate existing users from custom auth to Kratos
# This script reads users from engine_search.users and creates identities in Kratos

set -e

DB_HOST="localhost"
DB_PORT="5434"
DB_NAME="engine_search"
DB_USER="postgres"
export PGPASSWORD="postgres"
KRATOS_ADMIN_URL="http://127.0.0.1:4434"

echo "========================================="
echo "Kratos User Migration Script"
echo "========================================="
echo ""

# Check if Kratos is running
echo "Checking Kratos connection..."
if ! curl -s "${KRATOS_ADMIN_URL}/health/ready" > /dev/null 2>&1; then
    echo "❌ Error: Kratos is not running or not accessible at ${KRATOS_ADMIN_URL}"
    echo "Please start Kratos: docker-compose up -d kratos"
    exit 1
fi
echo "✅ Kratos is ready"
echo ""

# Count users to migrate
USER_COUNT=$(psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -A -c \
  "SELECT COUNT(*) FROM users WHERE role = 'user' AND is_active = true")

echo "Found ${USER_COUNT} users to migrate"
echo ""

if [ "$USER_COUNT" -eq 0 ]; then
    echo "No users to migrate. Exiting."
    exit 0
fi

read -p "Do you want to continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Migration cancelled."
    exit 0
fi

echo ""
echo "Starting migration..."
echo "========================================="

# Create temp file for users
TEMP_FILE=$(mktemp)

# Export users to temp file
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -A -F"|" -c \
  "SELECT id, email, password_hash, first_name, last_name, created_at
   FROM users
   WHERE role = 'user' AND is_active = true
   ORDER BY created_at" > "$TEMP_FILE"

# Track migration stats
TOTAL=0
SUCCESS=0
FAILED=0

# Migrate each user
while IFS='|' read -r id email password_hash first_name last_name created_at; do
  TOTAL=$((TOTAL + 1))
  echo -n "[$TOTAL/$USER_COUNT] Migrating: $email ... "

  # Create identity in Kratos
  RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${KRATOS_ADMIN_URL}/admin/identities" \
    -H "Content-Type: application/json" \
    -d "{
      \"schema_id\": \"default\",
      \"traits\": {
        \"email\": \"$email\",
        \"first_name\": \"$first_name\",
        \"last_name\": \"$last_name\"
      },
      \"credentials\": {
        \"password\": {
          \"config\": {
            \"hashed_password\": \"$password_hash\"
          }
        }
      },
      \"metadata_public\": {
        \"legacy_user_id\": \"$id\",
        \"migrated_from_custom_auth\": true,
        \"migrated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"original_created_at\": \"$created_at\"
      },
      \"state\": \"active\"
    }")

  # Extract HTTP status code
  HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)

  if [ "$HTTP_CODE" -eq 201 ] || [ "$HTTP_CODE" -eq 200 ]; then
    echo "✅ Success"
    SUCCESS=$((SUCCESS + 1))
  else
    echo "❌ Failed (HTTP $HTTP_CODE)"
    FAILED=$((FAILED + 1))
    # Show error details
    echo "$RESPONSE" | head -n -1 | jq -r '.error.message // .message // "Unknown error"' 2>/dev/null || echo "Unknown error"
  fi
done < "$TEMP_FILE"

# Cleanup
rm "$TEMP_FILE"

echo ""
echo "========================================="
echo "Migration Complete!"
echo "========================================="
echo "Total users: $TOTAL"
echo "Successfully migrated: $SUCCESS"
echo "Failed: $FAILED"
echo ""

# Verify migrated users in Kratos
echo "Verifying migrated users in Kratos..."
KRATOS_COUNT=$(curl -s "${KRATOS_ADMIN_URL}/admin/identities" | jq '. | length')
echo "Kratos now has ${KRATOS_COUNT} identities"
echo ""

if [ "$FAILED" -gt 0 ]; then
    echo "⚠️  Warning: Some users failed to migrate. Please review the errors above."
    exit 1
else
    echo "✅ All users migrated successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Test login with a migrated user account"
    echo "2. Update backend API routes (see KRATOS_MIGRATION_GUIDE.md)"
    echo "3. Update frontend auth pages (see KRATOS_MIGRATION_GUIDE.md)"
    exit 0
fi
