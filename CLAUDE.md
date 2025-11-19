# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a multi-platform AI chat application built with Dioxus, supporting Web, Desktop, and Mobile platforms with local LLM integration. The project uses a workspace structure with shared UI components and API logic.

## Development Commands

```bash
# Build all packages in the workspace
cargo build

# Run specific platform packages
cd packages/web && cargo run    # Web version
cd packages/desktop && cargo run    # Desktop version  
cd packages/mobile && cargo run    # Mobile version

# Run tests
cargo test

# Clean build artifacts
cargo clean
```

Note: The README mentions a `run.sh` script, but it has been deleted from the repository. Use cargo commands directly.

## Architecture

### Workspace Structure
- **`packages/ui/`**: Shared UI components used across all platforms
- **`packages/api/`**: Shared backend logic and LLM integration services
- **`packages/web/`**: Web-specific implementation with server-side rendering
- **`packages/desktop/`**: Desktop-specific implementation with local LLM support
- **`packages/mobile/`**: Mobile-specific implementation with touch-optimized interface

### Core Components

#### UI Package (`packages/ui/src/`)
The UI package exports simplified chat components that are known to work:
- `SimpleChatContainer`: Main chat interface
- `SimpleSidebar`: Conversation list and management
- `SimpleModelSelector`: Model selection dropdown
- `SimpleChatMessage`: Individual chat message display
- `SimpleConversationItem`: Conversation list item

#### API Package (`packages/api/src/`)
The API package provides:
- `ChatService`: Core LLM integration supporting multiple providers
- `ChatProvider` trait: Extensible interface for different AI providers
- Built-in support for thinking/reasoning content and tool calls
- Server functions for echo, models, chat, streaming chat, and tools endpoints

### Key Files

- **`packages/api/src/lib.rs`**: Main API exports, provider traits, server functions
- **`packages/ui/src/lib.rs`**: UI component exports, simplified chat components
- **`packages/web/src/main.rs`**: Web application entry point with routing

## Current State

The codebase is in a transition state:
- Complex legacy components have been replaced with simplified versions
- MCP (Model Context Protocol) and providers modules are temporarily disabled to avoid compilation issues
- Focus is on simplified chat functionality that works across platforms
- The project supports streaming responses, thinking content, and tool calling

## Development Notes

- Use the simplified components from the UI package rather than attempting to use disabled legacy components
- The ChatService supports thinking content and tool calls out of the box
- Server functions are configured for both web and desktop deployment
- Dioxus 0.7.1 is used with fullstack features for web platform