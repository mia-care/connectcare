#!/bin/bash

# Generate HMAC signature
SECRET="test_secret"
BODY='{"webhookEvent":"jira:issue_created","issue":{"id":"12345","key":"TEST-123"}}'
SIGNATURE=$(echo -n "$BODY" | openssl dgst -sha256 -hmac "$SECRET" | sed 's/^.* //')

# Send request
curl -X POST http://localhost:8080/jira/webhook \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature: sha256=$SIGNATURE" \
  -d "$BODY" \
  -v
