# Dioxus Chat Application

A multi-platform AI chat application built with Dioxus, supporting Web, Desktop, and Mobile platforms with local LLM integration.

## Features

- 🤖 **AI Chat Integration**: Support for local LLM models (Llama, Mistral) and cloud APIs (OpenAI, Anthropic)
- 💬 **Modern Chat Interface**: Clean, responsive chat UI with message history, conversation management
- 📱 **Cross-Platform**: Works on Web, Desktop (native), and Mobile devices
- 🎨 **Shared UI Components**: Consistent design across all platforms using Dioxus components
- 🔧 **Local Model Support**: Run AI models locally for privacy and offline usage
- 📝 **Conversation Management**: Create, switch between, and delete conversations
- 🌙 **Dark Mode Support**: Built-in dark/light theme switching

## Project Structure

```
dioxus-chat/
├─ README.md
├─ Cargo.toml           # Workspace configuration
├─ run.sh               # Development script for easy building/running
└─ packages/
   ├─ ui/               # Shared UI components used across all platforms
   │  ├─ src/
   │  │  ├─ lib.rs
   │  │  ├─ message.rs          # Chat message component
   │  │  ├─ chat_input.rs       # Chat input component
   │  │  ├─ chat_container.rs   # Main chat container
   │  │  ├─ sidebar.rs          # Conversation sidebar
   │  │  ├─ model_selector.rs   # Model selection dropdown
   │  │  └─ ...
   │  └─ assets/
   │     └─ chat.css            # Chat-specific styles
   ├─ api/              # Shared backend logic and LLM integration
   │  ├─ src/
   │  │  ├─ lib.rs
   │  │  └─ chat_service.rs     # LLM service implementation
   │  └─ Cargo.toml
   ├─ web/              # Web-specific implementation
   │  ├─ src/
   │  │  ├─ main.rs
   │  │  └─ views/
   │  │     ├─ mod.rs
   │  │     ├─ chat.rs          # Web chat view
   │  │     └─ ...
   │  └─ assets/
   ├─ desktop/          # Desktop-specific implementation
   │  ├─ src/
   │  │  ├─ main.rs
   │  │  └─ views/
   │  │     ├─ mod.rs
   │  │     ├─ chat.rs          # Desktop chat view (local models)
   │  │     └─ ...
   │  └─ assets/
   └─ mobile/           # Mobile-specific implementation
      ├─ src/
      │  ├─ main.rs
      │  └─ views/
      │     ├─ mod.rs
      │     ├─ chat.rs          # Mobile chat view (responsive)
      │     └─ ...
      └─ assets/
```

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Node.js (for web build dependencies)
- For desktop: platform-specific build tools
- For mobile: Android Studio/iOS development tools

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd dioxus-chat
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   ```

3. **Run the application:**
   
   Use the provided script for easy development:
   ```bash
   # Run web version
   ./run.sh web run
   
   # Run desktop version
   ./run.sh desktop run
   
   # Run mobile version
   ./run.sh mobile run
   ```

   Or run manually:
   ```bash
   # Web
   cd packages/web
   cargo run
   
   # Desktop
   cd packages/desktop
   cargo run
   
   # Mobile
   cd packages/mobile
   cargo run
   ```

## Platform-Specific Features

### Web Platform (`/web`)
- Server-side rendering support
- API integration for cloud LLM services
- Responsive web design
- Progressive Web App (PWA) capabilities

### Desktop Platform (`/desktop`)
- **Local LLM Integration**: Direct integration with local GGUF models
- File system access for model management
- Native window controls and shortcuts
- Offline-first architecture

### Mobile Platform (`/mobile`)
- Touch-optimized interface
- Slide-in navigation drawer
- Mobile-specific gestures and interactions
- Responsive design for various screen sizes

## LLM Model Configuration

### Local Models (Desktop)
The application supports GGUF format models. Place your model files in the `models/` directory:

```bash
# Example model paths
models/llama-2-7b-chat.gguf
models/mistral-7b-instruct.gguf
models/your-custom-model.gguf
```

### Cloud API Models (Web/Mobile)
Configure API keys in the environment or configuration:

```bash
# OpenAI
export OPENAI_API_KEY="your-api-key"

# Other providers can be added similarly
```

## Available Models

### Built-in Support
- **Llama 2 7B Chat**: General-purpose chat model
- **Mistral 7B Instruct**: Instruction-following model
- **GPT-3.5 Turbo**: Fast cloud-based model
- **GPT-4**: Advanced reasoning model

## Development Commands

```bash
# Build all platforms
./run.sh all build

# Clean all build artifacts
./run.sh all clean

# Run specific platform
./run.sh web run
./run.sh desktop run
./run.sh mobile run

# Build specific platform
./run.sh web build
./run.sh desktop build
./run.sh mobile build
```

## Components Overview

### Chat Components (`packages/ui/src/`)

- **Message**: Individual chat message with avatar and timestamp
- **ChatInput**: Multi-line input with send button and keyboard shortcuts
- **ChatContainer**: Main chat interface with message list and input
- **Sidebar**: Conversation list with new chat and management features
- **ModelSelector**: Dropdown for selecting AI models

### API Service (`packages/api/src/`)

- **ChatService**: Core LLM integration supporting multiple providers
- **Model Management**: Dynamic model loading and configuration
- **Streaming Support**: Real-time response streaming

## Customization

### Adding New Models
Edit `packages/api/src/chat_service.rs` to add new model configurations:

```rust
self.models.insert("your-model".to_string(), ModelConfig {
    id: "your-model".to_string(),
    name: "Your Model Name".to_string(),
    provider: "Provider".to_string(),
    // ... other config
});
```

### Customizing UI
Modify components in `packages/ui/src/` and styles in `packages/ui/assets/`.

### Platform-Specific Customization
Each platform crate (`web/`, `desktop/`, `mobile/`) can be customized independently while sharing core UI components.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test across all platforms
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Open an issue on GitHub
- Check the documentation
- Review the example implementations

