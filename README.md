# XZepr MCP server

## Overview

XZepr MCP server is a rust MCP server for XZepr High-Performance Event Tracking Server. The server is designed to handle events and metadata, providing a robust and efficient way to manage data with AI.

## Features

- **Event Handling**: The server can fetch and process events from a specified URL.
- **Metadata Management**: It supports metadata storage and retrieval using a flexible key-value structure.
- **Error Handling**: The server includes comprehensive error handling to ensure reliability.

## Development

MCP Inspector is a tool for inspecting and debugging MCP events. It provides a user-friendly interface to visualize and interact with the events processed by the server.

```bash
export JANUS_APIKEY=your_janus_api_key
cargo run
```

In another terminal, you can run the MCP Inspector to visualize the events:

```bash
npx @modelcontextprotocol/inspector
```

Use Streamable HTTP to connect to the MCP server:

URL:

```text
http://localhost:8000/mcp
```

1. MCP Transport will be StreamableHTTP for remote clients

use rmcp::{
ServerHandler,
handler::server::{router::tool::ToolRouter, tool::Parameters},
model::\*,
tool, tool_handler, tool_router,
transport::streamable_http_server::{
StreamableHttpService, session::local::LocalSessionManager,
},
};

2. OpenAPI: Do health check endpoints need OpenAPI documentation? YES

3. API Versioning: Are XZepr endpoints external (document as-is) or internal (add versioning)? XZepr api endpoints are versioned

4. Caching: Do performance targets require response caching? Maybe. Make this a FUTURE feature

Did we add the other priorities to the plan?

Priority 2 (SHOULD FIX):
Add resilience patterns (retry, circuit breaker, connection pooling)
Define observability schema (metric names, log format, spans)
Add Dockerfile to Phase 4 deliverables
Specify timestamp format as RFC-3339 in data models
Add validation rules to data models
Structured Logging for tracing tool usage

Priority 3 (NICE TO HAVE):
Consider splitting Phase 2 into sub-phases
Add graceful shutdown to Phase 4
Add performance caveat about XZepr dependency
Metrics Promethius
