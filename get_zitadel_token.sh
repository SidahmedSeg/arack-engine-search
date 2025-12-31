#!/bin/bash
# Get Zitadel Access Token from Service Account Key
# This script generates a JWT and exchanges it for an access token

set -e

ZITADEL_DOMAIN="auth.arack.io"
KEY_ID="353106685046292493"
USER_ID="353043841739128836"

# Save the private key to a temporary file
cat > /tmp/zitadel_key.pem <<'EOF'
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA4jfeIlNEIpQiy3vOUB4b/pU/EBvcZKF3xwz/+6b8gs9/E3a4
Nai/kmFN99kggWmmX/6d1e9cC4dP0dpbUg5NuyPh8LkWhDntJBnEiuyJOg4sM56a
LYhC9BLsN55r/SZiwkbP1WDD4gRmmFproJPcivpP8UFmWqx58E2Iv+txFx3jR241
OgzwA6QZ/ZViXNT6uHMWI76H7k6HdrcsrtvZQ/w11IuNmPXB14xsmV7eyTDEYyR2
WdWViMnl26nZUNHNkgEkta81/LDpGyu55VOcfBfvVQSVI5bX+ZPg4vrJPOACf2Mo
bIk90H/LRgwpswLKpBg24ej0H0QBjaYKrk5pBwIDAQABAoIBABwR2ppfwqCXyQl+
v0bptYeNdVnGWz6RWo90aX8MZWDF5nq/zHO8EKlVMZuDcakdNuvKaENXhBBaJelg
MtynsbV66lo4XTbjCS3llKG4X/64K3vsPi1QLx6iCnWMTtIGpVJ9/uP3MdclvKVC
8v/l2QPVs324I780j5zlwgYubMX1T6vVEhZVdPw45OLS9RtlSWzypFbM0sGoN/We
nNiwKOObiDjrNT3DQhe+iLjFDVqR6S2l2IFDUEq7pBUxZcLyjHBht3uBeTpFi4r2
daY6wJfKJsgHJT9pfOLbtf8lq/dwEQLtMNL2cVeEofkCQE13MWD5A3RoRkzTfdzf
BVri6iECgYEA+xdQOHp+gPkO6hk6OhevWsbnZ51taVLIhh1JYSH80The11W7u3QB
Vn9APVTgLX98bKsn8clmPWsyyfg2aEE+XHRFVYsAa/iZwqGoQ4A9sRT9jkXb/R3X
ZhZLkbw43XB6shKpklCTn4OlvnbcNyyb6C4ZWB2NbUJnbPQjFs5mbTUCgYEA5qQR
ded887XnYSLxwzTT4BfzJjs7CL6wI2qZaZrRIVzyf3kJjRu/KKMznXiyYTaWYY31
hjqwBPFEioxEILYpTwZewcfNvi/zNI/dWgposANpgNlUP6uglaOMB0klDMFQRWAQ
Oo8TU5EKGvyjYHbNrG9fUC2jF2MJ0+1ZE4wIkMsCgYEAkzG4ilNs44idsKh3VOTb
nFisATbtk+e+u7hhcvqsra3hE1UkB6Daw+03KH0gKivpMf8oHHrXX7v++x1yL63H
tAVJO/uPlLwYz1tbO67q+2t1tLjJXNnokuURCe8QWuf8VXXRSH/J+qH/QOeG8yVI
hFS58MjqRpm97cY058K5kt0CgYBYFGgmotTxLZLDYdj6N36CoiLVguE0ob4aiGc2
EU20dA6X591h6irCljDr/mABCBu9/by6GkeGW61VS+Poqih0aXZegrNr4lv6wsZB
0J/SODteXaDy/9Q/0Ul0rtZbXKgPFnScKG+1BjrZu4mZGUXKiG3Z7NPP5p6mNIpE
EMJkqQKBgQCjhckAc+W8Eou6OQb9PkYec1lytqZs47mBydVoJbP0Cc8ELPAtGZVU
1PFQM7Me0a0BD9DI/08kwgS/Kl6krdJGxiootxE1SXcb5KwoeR3YlutqVQL4M8b3
a7t+PN4X1ko64KVJmqUHSuP72rnyFxVQFvV2Kbsb36VvSzg3qQiRFg==
-----END RSA PRIVATE KEY-----
EOF

# Create JWT header
HEADER=$(echo -n '{"alg":"RS256","kid":"'${KEY_ID}'"}' | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')

# Create JWT payload
NOW=$(date +%s)
EXP=$((NOW + 3600))
PAYLOAD=$(echo -n '{
  "iss":"'${USER_ID}'",
  "sub":"'${USER_ID}'",
  "aud":"https://'${ZITADEL_DOMAIN}'",
  "iat":'${NOW}',
  "exp":'${EXP}'
}' | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')

# Create signature
SIGNATURE=$(echo -n "${HEADER}.${PAYLOAD}" | openssl dgst -sha256 -sign /tmp/zitadel_key.pem | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')

# Complete JWT
JWT="${HEADER}.${PAYLOAD}.${SIGNATURE}"

# Exchange JWT for access token with expanded scopes
TOKEN_RESPONSE=$(curl -s -X POST "https://${ZITADEL_DOMAIN}/oauth/v2/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer" \
  --data-urlencode "assertion=${JWT}" \
  --data-urlencode "scope=openid profile email urn:zitadel:iam:org:project:id:zitadel:aud urn:zitadel:iam:org:projects:roles")

# Extract access token
ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$ACCESS_TOKEN" ]; then
  echo "Failed to get access token"
  echo "Response: $TOKEN_RESPONSE"
  rm /tmp/zitadel_key.pem
  exit 1
fi

# Clean up
rm /tmp/zitadel_key.pem

# Output the access token
echo "$ACCESS_TOKEN"
