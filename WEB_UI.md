# Axon Cluster Web UI

A modern, ChatGPT-like interface for interacting with your private AI inference P2P network.

## Architecture: The Sidecar Pattern

The Web UI uses the **sidecar pattern** to bridge browser limitations with P2P protocols:

```
Browser (React)  ←→  HTTP API (Axum)  ←→  P2P Swarm (libp2p)
   Port 5173           Port 3000           Private Network
```

### How It Works

1. **Frontend**: React app with Tailwind CSS runs in your browser
2. **HTTP Sidecar**: Axum server provides REST API at `localhost:3000`
3. **Command Channel**: Uses `tokio::mpsc` to send commands from HTTP → P2P
4. **Event Loop**: `tokio::select!` handles both HTTP requests and P2P events concurrently
5. **Response Delivery**: Oneshot channels return P2P responses to HTTP handlers

This pattern enables browser access while maintaining the security and performance benefits of P2P networking.

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Node.js 18+ with npm
- Ollama running locally (default: `localhost:11434`)

### One-Command Start

```bash
./start_web.sh
```

This script will:

- Generate `swarm.key` if missing
- Detect WSL and configure `.env` automatically
- Build Rust backend
- Install npm dependencies
- Start both backend and frontend

Access the UI at: **http://localhost:5173**

## Manual Setup

### 1. Build Backend

```bash
cargo build --release
```

### 2. Install Frontend Dependencies

```bash
cd web-app
npm install
cd ..
```

### 3. Start Backend (Leader Mode with HTTP)

```bash
./target/release/axon_cluster web
```

Backend API will be available at `http://localhost:3000/api`

### 4. Start Frontend (separate terminal)

```bash
cd web-app
npm run dev
```

Frontend will open at `http://localhost:5173`

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# Ollama API endpoint (required for WSL users)
OLLAMA_LOCALHOST=http://192.168.1.100:11434

# Optional: change default model
OLLAMA_MODEL=llama2
```

### WSL Configuration

If running in WSL with Ollama on Windows:

```bash
# Get Windows IP
cat /etc/resolv.conf | grep nameserver | awk '{print $2}'

# Create .env
echo "OLLAMA_LOCALHOST=http://<WINDOWS_IP>:11434" > .env
```

## API Reference

### Health Check

```bash
GET http://localhost:3000/api/health
```

Response:

```json
{
  "status": "ok"
}
```

### Ask Question

```bash
POST http://localhost:3000/api/ask
Content-Type: application/json

{
  "prompt": "What is Rust?"
}
```

Response:

```json
{
  "response": "Rust is a systems programming language..."
}
```

## UI Components

### ChatWindow

- Main interface with message history
- Auto-scrolls to latest message
- Health check polling every 5 seconds

### MessageBubble

- User messages: Blue, right-aligned
- AI responses: Gray, left-aligned
- Timestamps and "thinking" indicator

### InputArea

- Text input with send button
- Disabled during processing
- Submit on Enter key

### StatusIndicator

- Green pulse: Connected to swarm
- Red static: Disconnected

## Troubleshooting

### Backend Won't Start

**Error**: `Address already in use (port 3000)`

```bash
# Find and kill process using port 3000
lsof -ti:3000 | xargs kill -9
```

### Frontend Can't Connect

**Error**: `Failed to fetch` or CORS errors

1. Verify backend is running: `curl http://localhost:3000/api/health`
2. Check CORS configuration in [src/http_server.rs](src/http_server.rs#L74-L79)
3. Ensure frontend is requesting correct URL

### WSL Ollama Connection

**Error**: `Connection refused` to Ollama

1. Get Windows IP: `cat /etc/resolv.conf | grep nameserver`
2. Update `.env`: `OLLAMA_LOCALHOST=http://<IP>:11434`
3. Restart backend

### P2P Network Issues

The web UI currently requires at least one P2P peer. To test:

1. Start web UI: `./start_web.sh`
2. In another terminal, start a subordinate: `./target/release/axon_cluster serve`
3. Web UI should now forward requests to subordinate nodes

## Development

### Frontend Development

```bash
cd web-app
npm run dev    # Development server with hot reload
npm run build  # Production build
npm run preview # Preview production build
```

### Backend Development

```bash
cargo watch -x run  # Auto-reload on changes
cargo test          # Run tests
cargo clippy        # Linter
```

### Project Structure

```
axon_cluster/
├── src/
│   ├── main.rs          # Event loop with tokio::select!
│   ├── http_server.rs   # Axum HTTP API
│   ├── cli.rs           # CLI with 'web' mode
│   └── ...
├── web-app/
│   ├── src/
│   │   ├── components/
│   │   │   ├── ChatWindow.jsx
│   │   │   ├── MessageBubble.jsx
│   │   │   ├── InputArea.jsx
│   │   │   └── StatusIndicator.jsx
│   │   ├── App.jsx
│   │   └── index.css
│   ├── package.json
│   └── vite.config.js
└── start_web.sh
```

## Next Steps

- [ ] Implement peer discovery tracking in web mode
- [ ] Add request forwarding to subordinate nodes
- [ ] WebSocket support for real-time updates
- [ ] Multi-model selection in UI
- [ ] Conversation history persistence
- [ ] Deploy with Docker Compose

## Related Documentation

- [README.md](README.md) - Main project overview
- [QUICKSTART.md](QUICKSTART.md) - CLI-based quick start
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Technical implementation details
- [WSL_SETUP.md](WSL_SETUP.md) - WSL-specific troubleshooting

---

Built with ❤️ using React, Vite, Tailwind CSS, Axum, and libp2p
