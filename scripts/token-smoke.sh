#!/usr/bin/env bash
set -euo pipefail

API_BASE="${API_BASE:-http://127.0.0.1:3000}"

create_payload='{"symbol":"USDC","name":"USD Coin","decimals":6,"initial_supply":1000000,"token_type":"stablecoin","metadata_json":"{\"issuer\":\"FlowCortex\"}"}'

printf "\n[1/3] Create token...\n"
curl -sS -X POST "${API_BASE}/token/create" \
  -H 'Content-Type: application/json' \
  -d "${create_payload}" | sed -e 's/\\r//g'

printf "\n\n[2/3] List tokens...\n"
curl -sS "${API_BASE}/tokens" | sed -e 's/\\r//g'

printf "\n\n[3/3] Get token...\n"
curl -sS "${API_BASE}/token/USDC" | sed -e 's/\\r//g'

printf "\n\nDone.\n"
