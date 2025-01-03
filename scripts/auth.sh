#!/bin/bash

# Configuration
API_URL="http://localhost:8080/api"
EMAIL="user@example.com"
PASSWORD="password123"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Registering new user...${NC}"

# Register user (if not already registered)
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\",
    \"password\": \"$PASSWORD\",
    \"password_confirmation\": \"$PASSWORD\"
  }")

echo -e "\nRegister Response: $REGISTER_RESPONSE"
echo -e "\n${BLUE}Logging in...${NC}"

# Login and extract token
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\",
    \"password\": \"$PASSWORD\"
  }")

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo "Failed to get token. Login response: $LOGIN_RESPONSE"
    exit 1
fi

echo -e "${GREEN}Successfully logged in!${NC}"
echo -e "\nYour token: $TOKEN"

# Save token to file for later use
echo $TOKEN > .token

echo -e "\n${BLUE}Making test scraping request...${NC}"

# Example scraping request using the token
SCRAPE_RESPONSE=$(curl -s -X POST "$API_URL/scraper" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
  }')

echo -e "\nScraping Response: $SCRAPE_RESPONSE"

echo -e "\n${GREEN}Done! You can now use the token for other requests.${NC}"
echo -e "The token has been saved to .token file for later use." 