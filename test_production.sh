#!/bin/bash

# Script de teste para API de produção
# URL: https://kong-security-api.fly.dev
# Data: 04/12/2025

echo "==========================================="
echo "Kong Security API - Production Test"
echo "==========================================="
echo ""

BASE_URL="https://kong-security-api.fly.dev"
TENANT_ID="59a88b51-81ec-4f46-8fd6-2ee8ec196d06"

# Cores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. Health Check
echo -e "${YELLOW}1. Testing API Health...${NC}"
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/health")
HTTP_CODE=$(echo "$HEALTH_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$HEALTH_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✓ API is healthy${NC}"
    echo "$RESPONSE_BODY" | jq '.' 2>/dev/null || echo "$RESPONSE_BODY"
else
    echo -e "${RED}✗ API health check failed (HTTP $HTTP_CODE)${NC}"
    echo "$RESPONSE_BODY"
fi
echo ""

# 2. Login Test
echo -e "${YELLOW}2. Testing Login...${NC}"
echo "Tenant ID: $TENANT_ID"
echo "Email: john@example.com"

LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: $TENANT_ID" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123!"
  }')

HTTP_CODE=$(echo "$LOGIN_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$LOGIN_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✓ Login successful${NC}"
    JWT_TOKEN=$(echo "$RESPONSE_BODY" | jq -r '.token' 2>/dev/null)
    echo "$RESPONSE_BODY" | jq '.' 2>/dev/null || echo "$RESPONSE_BODY"
    echo ""
    echo -e "${GREEN}JWT Token (first 50 chars): ${JWT_TOKEN:0:50}...${NC}"
    
    # 3. Test Protected Endpoint
    echo ""
    echo -e "${YELLOW}3. Testing Protected Endpoint...${NC}"
    PROTECTED_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/auth/protected" \
      -H "Authorization: Bearer $JWT_TOKEN" \
      -H "X-Tenant-ID: $TENANT_ID")
    
    HTTP_CODE=$(echo "$PROTECTED_RESPONSE" | tail -n1)
    RESPONSE_BODY=$(echo "$PROTECTED_RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Protected endpoint accessed successfully${NC}"
        echo "$RESPONSE_BODY" | jq '.' 2>/dev/null || echo "$RESPONSE_BODY"
    else
        echo -e "${RED}✗ Protected endpoint failed (HTTP $HTTP_CODE)${NC}"
        echo "$RESPONSE_BODY"
    fi
    
    # 4. Get My Logs
    echo ""
    echo -e "${YELLOW}4. Getting My Login Logs...${NC}"
    LOGS_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/logs/my-logins" \
      -H "Authorization: Bearer $JWT_TOKEN" \
      -H "X-Tenant-ID: $TENANT_ID")
    
    HTTP_CODE=$(echo "$LOGS_RESPONSE" | tail -n1)
    RESPONSE_BODY=$(echo "$LOGS_RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Logs retrieved successfully${NC}"
        echo "$RESPONSE_BODY" | jq '.[0:3]' 2>/dev/null || echo "$RESPONSE_BODY"
    else
        echo -e "${RED}✗ Get logs failed (HTTP $HTTP_CODE)${NC}"
        echo "$RESPONSE_BODY"
    fi
    
else
    echo -e "${RED}✗ Login failed (HTTP $HTTP_CODE)${NC}"
    echo "$RESPONSE_BODY"
fi

echo ""
echo "==========================================="
echo -e "${GREEN}Test completed!${NC}"
echo "==========================================="
