#!/usr/bin/env bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== FreeLLMAPI Auto-Setup for skills-bank ===${NC}"

# 1. Dependency Checks
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed. Please install Node.js (v18+) to run FreeLLMAPI.${NC}"
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: git is not installed. Please install git.${NC}"
    exit 1
fi

# 2. Locate or clone FreeLLMAPI
FREELLM_DIR=""
if [ -d "./freellmapi" ]; then
    FREELLM_DIR="./freellmapi"
    echo -e "${GREEN}Found FreeLLMAPI inside skills-bank directory.${NC}"
elif [ -d "../freellmapi" ]; then
    FREELLM_DIR="../freellmapi"
    echo -e "${GREEN}Found FreeLLMAPI as a sibling directory.${NC}"
else
    echo -e "${YELLOW}FreeLLMAPI directory not found. Cloning to ./freellmapi...${NC}"
    git clone https://github.com/tashfeenahmed/freellmapi.git freellmapi
    FREELLM_DIR="./freellmapi"
fi

# 3. Configure FreeLLMAPI
echo -e "${BLUE}Configuring FreeLLMAPI...${NC}"
cd "$FREELLM_DIR"

if [ ! -f ".env" ]; then
    if [ -f ".env.example" ]; then
        cp .env.example .env
    else
        touch .env
    fi
fi

# Generate encryption key if not present
if ! grep -q "ENCRYPTION_KEY=" .env; then
    HEX_KEY=$(node -e "console.log(require('crypto').randomBytes(32).toString('hex'))")
    echo "ENCRYPTION_KEY=$HEX_KEY" >> .env
    echo -e "${GREEN}Generated new ENCRYPTION_KEY in FreeLLMAPI config.${NC}"
fi

# 4. Install dependencies
echo -e "${BLUE}Installing dependencies in FreeLLMAPI...${NC}"
npm install

# 5. Initialize DB and get/generate the Unified API Key
echo -e "${BLUE}Initializing database and generating Unified API Key...${NC}"
UNIFIED_KEY=$(DEV_MODE=true npx tsx -e "
import { initDb, getUnifiedApiKey } from './server/src/db/index.ts';
initDb();
console.log(getUnifiedApiKey());
" 2>/dev/null | grep "^freellmapi-")

if [ -z "$UNIFIED_KEY" ]; then
    # Fallback to .js in imports just in case
    UNIFIED_KEY=$(DEV_MODE=true npx tsx -e "
    import { initDb, getUnifiedApiKey } from './server/src/db/index.js';
    initDb();
    console.log(getUnifiedApiKey());
    " 2>/dev/null | grep "^freellmapi-")
fi

if [ -z "$UNIFIED_KEY" ]; then
    echo -e "${RED}Warning: Could not automatically generate/retrieve the unified key using tsx.${NC}"
    echo -e "${YELLOW}You can find/generate it by running 'npm run dev' inside FreeLLMAPI directory.${NC}"
else
    echo -e "${GREEN}Successfully generated/retrieved Unified API Key: ${YELLOW}$UNIFIED_KEY${NC}"
fi

# 6. Configure skills-bank .env
cd - > /dev/null # Go back to skills-bank root

echo -e "${BLUE}Configuring skills-bank .env...${NC}"
SKILLS_ENV=".env"

# Ensure .env exists
touch "$SKILLS_ENV"

# Helper to upsert env var
upsert_env_var() {
    local key=$1
    local val=$2
    if grep -q "^$key=" "$SKILLS_ENV"; then
        # Replace existing
        sed -e "s|^$key=.*|$key=$val|" "$SKILLS_ENV" > "$SKILLS_ENV.tmp" && mv "$SKILLS_ENV.tmp" "$SKILLS_ENV"
    else
        # Append new
        echo "$key=$val" >> "$SKILLS_ENV"
    fi
}

upsert_env_var "LLM_PROVIDER" "freellmapi"
upsert_env_var "LLM_API_URL" "http://localhost:3001/v1"
upsert_env_var "LLM_BATCH_SIZE" "1"
upsert_env_var "LLM_CONCURRENCY" "1"
upsert_env_var "LLM_MAX_RETRIES" "3"
upsert_env_var "LLM_TIMEOUT_SECS" "20"

if [ -n "$UNIFIED_KEY" ]; then
    upsert_env_var "FREELLMAPI_API_KEY" "\"$UNIFIED_KEY\""
fi

echo -e "${GREEN}skills-bank .env configured successfully!${NC}"

echo -e "\n${GREEN}===================================================${NC}"
echo -e "${GREEN}Setup Complete! Next Steps:${NC}"
echo -e "1. Start the FreeLLMAPI proxy server:"
echo -e "   ${YELLOW}cd $FREELLM_DIR && npm run dev${NC}"
echo -e "2. Open the FreeLLMAPI Dashboard:"
echo -e "   ${BLUE}http://localhost:5173${NC}"
echo -e "3. Add your provider API keys under the 'Keys' tab."
echo -e "4. Run aggregation in skills-bank:"
echo -e "   ${YELLOW}./target/release/skills-bank aggregate${NC}"
echo -e "${GREEN}===================================================${NC}\n"
