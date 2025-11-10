# MCP (Model Context Protocol) Module

The MCP module provides integration with the Model Context Protocol, enabling Puetce to communicate with external MCP servers and dynamically load tools from them.

## Configuration

MCP servers are configured via `mcp_servers.json`. This is how you would add the filesystem server:

```json
{
  "mcp_servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/dir"],
      "env": null
    }
  ]
}
```

## Integration

The MCP module integrates with:
- **ToolManager** (`app::tool_manager`): Manages both MCP and integrated tools
- **AnthropicClient**: Converts MCP tools to tool definitions for the Claude API
- **PuetceApp**: Registers and executes MCP tools during bot startup/shutdown