# MCP (Model Context Protocol) Module

The MCP module provides a simple and extensible interface for adding and configuring local MCP servers

## Configuration

MCP servers are configured by adding entries to `mcp_servers.json`. This is an example of the format using the filesystem MCP server:

```json
{
  "mcp_servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/dir"],
      "env": {}
    }
  ]
}
```

## Integration

The MCP module integrates with:
- **ToolManager** (`app::tool_manager`): Manages both MCP and integrated tools
- **AnthropicClient**: Converts MCP tools to tool definitions for the Claude API
- **backend.rs**: Handles registering MCP servers
- **PuetceApp**: Executes MCP tools during bot startup/shutdown