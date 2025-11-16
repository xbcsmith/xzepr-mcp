# Configure OIDC Authentication with Keycloak

## Overview

This guide explains how to configure XZepr MCP server to use OpenID Connect (OIDC) authentication with Keycloak, which is integrated with the XZepr backend.

## Prerequisites

- XZepr backend with Keycloak configured
- Keycloak realm and client configured for XZepr
- Access to Keycloak admin console or CLI
- XZepr MCP server installed

## Authentication Architecture

```text
┌─────────────────┐
│   MCP Client    │ (Claude Desktop, VSCode, etc.)
│  (uses token)   │
└────────┬────────┘
         │ 1. MCP request with JWT token
         ▼
┌─────────────────────────────────────────┐
│        XZepr MCP Server                 │
│  2. Validate token (optional)           │
│     - Fetch JWKS from Keycloak          │
│     - Verify signature                  │
│     - Check claims (exp, iss, aud)      │
└────────┬────────────────────────────────┘
         │ 3. Forward request with token
         ▼
┌─────────────────────────────────────────┐
│           XZepr API                     │
│  4. Validate token with Keycloak        │
│  5. Authorize request                   │
└─────────────────────────────────────────┘
```

## Configuration Methods

### Method 1: Environment Variables

```bash
# Keycloak configuration
export OIDC_ISSUER_URL="https://keycloak.example.com/realms/xzepr"
export OIDC_CLIENT_ID="xzepr-mcp"
export OIDC_AUDIENCE="xzepr-mcp"  # REQUIRED - tokens must be issued for this audience
export OIDC_REQUIRED_SCOPES="xzepr:read,xzepr:write"
export OIDC_JWKS_CACHE_TTL=3600

# XZepr API configuration
export XZEPR_URL="https://xzepr.example.com"

# Start server
./xzepr-mcp
```

### Method 2: Configuration File

Create `config/production.yaml`:

```yaml
xzepr:
  url: "https://xzepr.example.com"
  timeout: 30

server:
  host: "0.0.0.0"
  port: 8000
  mcp_path: "/mcp"

oidc:
  issuer_url: "https://keycloak.example.com/realms/xzepr"
  client_id: "xzepr-mcp"
  jwks_cache_ttl: 3600
  audience: "xzepr-mcp" # REQUIRED - tokens must be issued for this audience
  required_scopes:
    - "xzepr:read"
    - "xzepr:write"

logging:
  level: "info"
  json: true
```

Run with config file:

```bash
./xzepr-mcp --config config/production.yaml
```

### Method 3: CLI Arguments

```bash
./xzepr-mcp \
  --oidc-issuer-url https://keycloak.example.com/realms/xzepr \
  --oidc-client-id xzepr-mcp \
  --oidc-audience xzepr-mcp \
  --xzepr-url https://xzepr.example.com
```

## Keycloak Setup

### Step 1: Create Keycloak Client

1. Log in to Keycloak admin console
2. Navigate to your realm (e.g., `xzepr`)
3. Go to **Clients** → **Create client**
4. Configure client:

```yaml
Client ID: xzepr-mcp
Client type: OpenID Connect
Access Type: confidential (if using client secret)
Valid Redirect URIs: http://localhost:8000/* (for development)
Web Origins: *
```

### Step 2: Configure Client Scopes

Add required scopes:

- `openid` (required)
- `profile` (recommended)
- `email` (recommended)
- `xzepr-api` (custom scope for XZepr)

### Step 3: Get Client Credentials

1. Go to **Clients** → `xzepr-mcp` → **Credentials** tab
2. Copy the **Client Secret** (if using confidential client)
3. Note the **Client ID**: `xzepr-mcp`

### Step 4: Create Test User

1. Go to **Users** → **Add user**
2. Set username, email, and enable user
3. Go to **Credentials** tab
4. Set password (temporary or permanent)
5. Assign roles as needed for XZepr access

## Obtaining Authentication Tokens

### Option 1: Using Direct Grant (Password Flow)

For CLI tools and testing:

```bash
# Get access token
TOKEN=$(curl -X POST \
  "https://keycloak.example.com/realms/xzepr/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=xzepr-mcp" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=password" \
  -d "username=user@example.com" \
  -d "password=YOUR_PASSWORD" \
  -d "scope=openid profile email" \
  | jq -r '.access_token')

echo $TOKEN
```

Note: Direct grant must be enabled in Keycloak client settings.

### Option 2: Using Authorization Code Flow (Web Browser)

For interactive authentication:

1. Open browser to:

   ```
   https://keycloak.example.com/realms/xzepr/protocol/openid-connect/auth?client_id=xzepr-mcp&response_type=code&redirect_uri=http://localhost:8000/callback&scope=openid
   ```

2. Log in with credentials
3. Extract authorization code from redirect URL
4. Exchange code for token:

```bash
curl -X POST \
  "https://keycloak.example.com/realms/xzepr/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=xzepr-mcp" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=authorization_code" \
  -d "code=AUTHORIZATION_CODE" \
  -d "redirect_uri=http://localhost:8000/callback"
```

### Option 3: Using Keycloak Admin CLI

```bash
# Configure kcadm
kcadm.sh config credentials \
  --server https://keycloak.example.com \
  --realm xzepr \
  --user admin

# Get token for user
kcadm.sh get-token \
  --realm xzepr \
  --client xzepr-mcp \
  --user user@example.com \
  --password YOUR_PASSWORD
```

### Option 4: Custom Token Tool (Recommended for End Users)

Create a simple token acquisition tool for users:

```bash
#!/bin/bash
# xzepr-get-token.sh

KEYCLOAK_URL="${KEYCLOAK_URL:-https://keycloak.example.com}"
REALM="${REALM:-xzepr}"
CLIENT_ID="${CLIENT_ID:-xzepr-mcp}"

echo "XZepr MCP Token Tool"
echo "===================="
echo ""
read -p "Username: " USERNAME
read -sp "Password: " PASSWORD
echo ""

TOKEN=$(curl -s -X POST \
  "${KEYCLOAK_URL}/realms/${REALM}/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=${CLIENT_ID}" \
  -d "grant_type=password" \
  -d "username=${USERNAME}" \
  -d "password=${PASSWORD}" \
  -d "scope=openid profile email" \
  | jq -r '.access_token')

if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
    echo "Token obtained successfully!"
    echo ""
    echo "Copy this token and use it in your MCP client configuration:"
    echo ""
    echo "$TOKEN"
    echo ""
    echo "To use with XZepr MCP:"
    echo "export XZEPR_TOKEN=\"$TOKEN\""
else
    echo "Failed to obtain token. Check credentials and try again."
    exit 1
fi
```

## MCP Client Configuration

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "xzepr": {
      "type": "http",
      "url": "http://localhost:8000/mcp",
      "env": {
        "XZEPR_TOKEN": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
      }
    }
  }
}
```

### VSCode MCP Extension

Edit `.vscode/mcp.json`:

```json
{
  "mcp.servers": {
    "xzepr": {
      "type": "http",
      "url": "http://localhost:8000/mcp"
    }
  },
  "mcp.inputs": [
    {
      "type": "promptString",
      "id": "xzepr_token",
      "description": "XZepr API Token (from Keycloak)",
      "password": true,
      "hint": "Run: ./xzepr-get-token.sh"
    }
  ]
}
```

## Token Validation Mode

### Mandatory Local Validation (Production Security)

**SECURITY CRITICAL**: MCP server ALWAYS validates tokens before use. This cannot be disabled.

**Configuration**:

```yaml
oidc:
  issuer_url: "https://keycloak.example.com/realms/xzepr"
  client_id: "xzepr-mcp"
  jwks_cache_ttl: 3600
  audience: "xzepr-mcp" # REQUIRED - tokens must be issued for this audience
  required_scopes:
    - "xzepr:read"
    - "xzepr:write"
```

**Security Features**:

- Prevents token passthrough vulnerabilities (MCP Security Best Practice)
- Validates token was issued FOR xzepr-mcp (audience validation)
- Validates token contains required scopes (least privilege)
- Early detection of invalid/expired tokens (better UX)
- Reduces XZepr API load from invalid requests
- Defense in depth (MCP validates, XZepr validates again)

**Performance**:

- Cached JWKS keeps validation fast (<5ms)
- First request fetches keys from Keycloak
- Subsequent requests use cached keys (1 hour TTL)

**Why This Matters**:
Token validation prevents attackers from using tokens issued for OTHER services to access XZepr via the MCP server. This is a critical security control mandated by MCP security specifications.

## Token Validation Details

The MCP server ALWAYS performs the following validations:

1. **Signature Verification**:

   - Fetches JWKS (JSON Web Key Set) from Keycloak
   - Caches keys for configured TTL (default 1 hour)
   - Verifies JWT signature using public key

2. **Claim Validation** (MANDATORY):

   - `iss` (issuer): Must match configured issuer URL
   - `aud` (audience): Must match configured audience (REQUIRED - typically "xzepr-mcp")
   - `exp` (expiration): Token must not be expired
   - `iat` (issued at): Token must have valid issue time
   - `nbf` (not before): Token must be valid for use
   - `scope`: Must contain all required scopes (e.g., "xzepr:read", "xzepr:write")

3. **Error Handling**:
   - Invalid signature → 401 Unauthorized
   - Expired token → 401 Unauthorized (clear message to refresh)
   - Wrong issuer → 401 Unauthorized
   - Missing claims → 400 Bad Request

## Troubleshooting

### Issue: "Invalid token signature"

**Cause**: JWT signature cannot be verified

**Solutions**:

1. Check `OIDC_ISSUER_URL` matches Keycloak realm exactly
2. Ensure Keycloak is accessible from MCP server
3. Verify JWKS endpoint is reachable:
   ```bash
   curl https://keycloak.example.com/realms/xzepr/protocol/openid-connect/certs
   ```
4. Check token was issued by correct Keycloak instance
5. Verify token `aud` claim matches `OIDC_AUDIENCE` configuration

### Issue: "Token expired"

**Cause**: JWT has passed expiration time

**Solutions**:

1. Obtain new token using token acquisition method
2. Check system clock synchronization (NTP)
3. Increase token lifetime in Keycloak (Client → Advanced → Access Token Lifespan)

### Issue: "JWKS fetch failed"

**Cause**: Cannot download keys from Keycloak

**Solutions**:

1. Verify Keycloak URL is correct and accessible
2. Check network connectivity from MCP server
3. Verify Keycloak is running and healthy
4. Check firewall rules allow outbound HTTPS

### Issue: "Audience claim mismatch"

**Cause**: Token `aud` claim doesn't match configuration

**CRITICAL**: This is a security control and cannot be bypassed. Tokens MUST be issued for the xzepr-mcp audience.

**Solutions**:

1. Verify `OIDC_AUDIENCE` is set to "xzepr-mcp"
2. Configure Keycloak client audience mapper to include "xzepr-mcp"
3. Inspect token claims to see actual audience:
   ```bash
   echo $TOKEN | cut -d. -f2 | base64 -d | jq '.aud'
   ```
4. Obtain new token with correct audience from Keycloak
5. DO NOT remove audience validation - this is a required security control

**Keycloak Audience Mapper Configuration**:

1. Go to Client → xzepr-mcp → Client Scopes → Dedicated
2. Add Mapper → Audience
3. Set Included Client Audience: xzepr-mcp
4. Save and obtain new token

### Issue: "Client authentication failed"

**Cause**: Wrong client credentials when obtaining token

**Solutions**:

1. Verify `client_id` is correct
2. Check `client_secret` if using confidential client
3. Ensure client is enabled in Keycloak
4. Verify client access type matches authentication method

## Token Lifecycle Management

### Token Expiration

Keycloak tokens typically expire after 5-15 minutes. Users must obtain fresh tokens.

**Best Practices**:

- Set token lifespan appropriate for use case
- Provide clear error messages on expiration
- Document token refresh process for users

### Token Refresh (Future Enhancement)

Currently, token refresh is user's responsibility. Future versions may support:

```yaml
oidc:
  enable_refresh: true
  refresh_token_ttl: 3600
```

### Revoking Tokens

To revoke a user's access:

1. In Keycloak: **Users** → Select user → **Sessions** → **Sign out**
2. Or revoke via API:
   ```bash
   curl -X POST \
     "https://keycloak.example.com/realms/xzepr/protocol/openid-connect/logout" \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "client_id=xzepr-mcp" \
     -d "refresh_token=REFRESH_TOKEN"
   ```

## Security Best Practices

### Production Deployment

1. **Always use HTTPS**:

   ```yaml
   oidc:
     issuer_url: "https://keycloak.example.com/realms/xzepr"
   ```

2. **Set required audience** (MANDATORY):

   ```yaml
   oidc:
     audience: "xzepr-mcp" # Tokens MUST be issued for this audience
   ```

3. **Configure required scopes**:

   ```yaml
   oidc:
     required_scopes:
       - "xzepr:read"
       - "xzepr:write"
   ```

4. **Use confidential clients** for server-to-server communication

5. **Rotate client secrets** regularly

6. **Monitor authentication failures**:
   ```bash
   # Check logs for auth errors
   grep "authentication failed" /var/log/xzepr-mcp.log
   ```

### Token Storage

- Store tokens in environment variables or secure credential stores
- Never commit tokens to version control
- Use short-lived tokens (5-15 minutes)
- Rotate tokens frequently

### Network Security

- Restrict Keycloak access to authorized networks
- Use VPN or private networks for production
- Enable TLS for all connections
- Implement rate limiting on token endpoints

## Testing OIDC Configuration

### Test Token Validation

```bash
# Get token
TOKEN=$(./xzepr-get-token.sh)

# Test MCP server authentication
curl -v http://localhost:8000/api/v1/health \
  -H "Authorization: Bearer $TOKEN"

# Should return 200 OK if token is valid
```

### Verify JWKS Caching

```bash
# Check logs for JWKS fetch
tail -f /var/log/xzepr-mcp.log | grep "JWKS"

# Should see initial fetch, then cache hits
```

### Inspect Token Claims

```bash
# Decode token (requires jq)
echo $TOKEN | cut -d. -f2 | base64 -d | jq

# Expected output:
{
  "exp": 1699564800,
  "iat": 1699564500,
  "jti": "uuid-here",
  "iss": "https://keycloak.example.com/realms/xzepr",
  "aud": "xzepr-api",
  "sub": "user-id",
  "typ": "Bearer",
  "azp": "xzepr-mcp",
  "preferred_username": "user@example.com",
  "email": "user@example.com",
  "scope": "openid profile email xzepr:read xzepr:write"
}
```

## Reference

### OIDC Configuration Options

| Option            | Type         | Default | Description                                     |
| ----------------- | ------------ | ------- | ----------------------------------------------- |
| `issuer_url`      | String       | -       | Keycloak realm URL (required)                   |
| `client_id`       | String       | -       | OIDC client ID (required)                       |
| `client_secret`   | String       | -       | Client secret (optional)                        |
| `jwks_cache_ttl`  | Integer      | 3600    | JWKS cache TTL in seconds                       |
| `audience`        | String       | -       | Required audience claim (REQUIRED for security) |
| `required_scopes` | String Array | []      | Required token scopes for authorization         |

### Keycloak Endpoints

For realm `xzepr` at `https://keycloak.example.com`:

- **OIDC Discovery**: `https://keycloak.example.com/realms/xzepr/.well-known/openid-configuration`
- **Token Endpoint**: `https://keycloak.example.com/realms/xzepr/protocol/openid-connect/token`
- **JWKS Endpoint**: `https://keycloak.example.com/realms/xzepr/protocol/openid-connect/certs`
- **Authorization**: `https://keycloak.example.com/realms/xzepr/protocol/openid-connect/auth`
- **User Info**: `https://keycloak.example.com/realms/xzepr/protocol/openid-connect/userinfo`

## Related Documentation

- [XZepr Keycloak Integration](../explanation/xzepr_keycloak_integration.md)
- [Security Considerations](../explanation/security_architecture.md)
- [Deployment Guide](deploy_production.md)
- [Troubleshooting Guide](troubleshooting.md)

---

**Last Updated**: 2024-01-XX
**Status**: Draft
