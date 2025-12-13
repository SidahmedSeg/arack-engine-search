#!/bin/bash

# Test script for the search engine crawler

API_URL="http://127.0.0.1:3000"

echo "==================================="
echo "Search Engine Test Script"
echo "==================================="
echo ""

# Test 1: Health Check
echo "1. Testing health check..."
curl -s "${API_URL}/health" | jq '.' || echo "Health check failed"
echo ""

# Test 2: Start a crawl
echo "2. Starting crawl for example.com..."
CRAWL_RESPONSE=$(curl -s -X POST "${API_URL}/api/crawl" \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "max_depth": 1
  }')
echo "$CRAWL_RESPONSE" | jq '.'
echo ""

# Test 3: Wait for indexing
echo "3. Waiting 5 seconds for indexing..."
sleep 5
echo ""

# Test 4: Search for content
echo "4. Searching for 'example'..."
curl -s "${API_URL}/api/search?q=example&limit=5" | jq '.'
echo ""

# Test 5: Get index statistics
echo "5. Getting index statistics..."
curl -s "${API_URL}/api/stats" | jq '.'
echo ""

echo "==================================="
echo "Tests completed!"
echo "==================================="
