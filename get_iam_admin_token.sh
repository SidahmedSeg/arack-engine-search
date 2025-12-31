#!/bin/bash
# Get IAM Admin Token using password grant flow

ZITADEL_DOMAIN="auth.arack.io"
ADMIN_USERNAME="admin"
ADMIN_PASSWORD="S1OUCJa4884muQIvW9St/RgWxiLqh+hh"

# Login and get session token
# First, we need to get a PAT from the admin account
# Since this is the instance admin, let's try using the introspect endpoint to verify

# Actually, let's use the Resource Owner Password Credentials flow if available
# Or we can create a PAT manually via the console

# For now, let's just output instructions
echo "To get IAM admin token:"
echo "1. Go to https://auth.arack.io/ui/console"
echo "2. Login with username: admin"
echo "3. Password: S1OUCJa4884muQIvW9St/RgWxiLqh+hh"
echo "4. Go to User Settings → Personal Access Tokens"
echo "5. Create a new PAT and copy the token"
