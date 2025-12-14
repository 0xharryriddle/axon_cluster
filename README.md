# Axon-Cluster 🧠⚡

A private P2P network that allows low-power "Subordinate" nodes (laptops) to offload AI inference tasks to a high-power "Leader" node (GPU Desktop) securely.

## Demo

![Axon-Cluster Demo](/assets/demo.gif)

## Features

✅ **Private Network (PNET)**: Only nodes with the correct `swarm.key` can join  
✅ **End-to-End Encryption**: All traffic encrypted with Noise protocol  
✅ **Automatic Discovery**: Nodes find each other via mDNS on local WiFi  
✅ **Leader-Subordinate Architecture**: Distributed AI inference workload  
✅ **Ollama Integration**: Seamless connection to local Ollama API

## Architecture

### Leader Mode (`serve`)

- Listens for inference requests from Subordinates
- Forwards prompts to local Ollama API
- Returns AI-generated responses
- Runs on high-power GPU Desktop

### Subordinate Mode (`ask`)

- Discovers Leader nodes on the network
- Sends prompts for AI inference
- Receives and displays responses
- Runs on low-power laptops/devices

## Setup

### 1. Generate the Pre-Shared Key (swarm.key)

All nodes must have the same `swarm.key` file to join the private network.

```bash
# Generate swarm.key
echo -e "/key/swarm/psk/1.0.0/\n/base16/" > swarm.key && openssl rand -hex 32 >> swarm.key
```

**Important**: Copy this `swarm.key` to all devices that should be part of your private network.

### 1.5. Configure Ollama URL (WSL Users)

If running in WSL with Ollama on Windows, create a `.env` file:

```bash
# Create .env file with your Windows host IP
echo "OLLAMA_LOCALHOST=http://172.23.160.1:11434" > .env

# To find your Windows IP from WSL:
cat /etc/resolv.conf | grep nameserver | awk '{print $2}'
```

The application will automatically use `OLLAMA_LOCALHOST` from `.env` when you run `serve` without `--ollama-url`.

### 2. Install Ollama (Leader Node Only)

On the Leader node, install Ollama:

```bash
# Linux/macOS
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model (e.g., llama2)
ollama pull llama2
```

### 3. Build Axon-Cluster

```bash
cargo build --release
```

## Usage

### Web UI (Recommended)

The easiest way to interact with Axon Cluster:

```bash
# One-command start (builds, installs dependencies, starts everything)
./start_web.sh
```

Then open **http://localhost:5173** in your browser for a ChatGPT-like interface.

📚 **[Complete Web UI Documentation →](WEB_UI.md)**

### CLI Mode

#### Running the Leader (Server)

On your GPU Desktop or powerful machine:

```bash
# Start Leader with default settings (llama2 model)
./target/release/axon_cluster serve

# Specify a different model
./target/release/axon_cluster serve --model mistral

# Specify a different Ollama URL
./target/release/axon_cluster serve --ollama-url http://192.168.1.100:11434 --model llama2
```

**Output:**

```
🚀 Starting Leader Mode (Server)
📡 Ollama URL: http://localhost:11434
🤖 Model: llama2
🔑 Local PeerId: 12D3KooW...
🔒 Private Network: Enabled
👂 Listening on: /ip4/0.0.0.0/tcp/54321
```

#### Running a Subordinate (Client)

On your laptop or low-power device:

```bash
# Send a prompt to the Leader
./target/release/axon_cluster ask "Explain quantum computing in simple terms"
```

**Output:**

```
🚀 Starting Subordinate Mode (Client)
💭 Prompt: Explain quantum computing in simple terms
🔑 Local PeerId: 12D3KooW...
🔒 Private Network: Enabled
👂 Listening on: /ip4/0.0.0.0/tcp/0
🔍 Discovering Leader nodes...
🎯 Found Leader: 12D3KooW...
📤 Sending inference request to Leader...

✅ Response from Leader:

Quantum computing is a new type of computing that uses quantum mechanics...
[AI-generated response continues]
```

## Security Features

### 1. Pre-Shared Key (PSK)

- Only nodes with the correct `swarm.key` can connect
- Unauthorized nodes are rejected immediately
- Key uses 256-bit hex encoding

### 2. Noise Protocol Encryption

- All network traffic is encrypted
- Uses libp2p-noise with X25519 key exchange
- Perfect forward secrecy

### 3. Private Network Isolation

- Nodes operate in an isolated P2P network
- Cannot accidentally connect to public libp2p networks
- mDNS discovery limited to local network

## Network Protocol

### Request Format

```json
{
  "prompt": "Your AI prompt here",
  "model": "llama2" // Optional, uses Leader's default if not specified
}
```

### Response Format

```json
{
  "response": "AI-generated response text",
  "success": true,
  "error": null
}
```

### Protocol Specifications

- **Protocol Name**: `/axon/inference/1.0.0`
- **Encoding**: JSON with length-prefix framing
- **Request Timeout**: 120 seconds
- **Discovery**: mDNS on local network

## Troubleshooting

### "swarm.key not found"

Generate the key file using the command in Setup section.

### "No Leader found"

- Ensure Leader is running with `serve` command
- Verify both nodes have the **same** `swarm.key`
- Check that both devices are on the same WiFi network
- Disable any firewalls blocking mDNS (port 5353 UDP)

### "Ollama API error" or "Connection refused"

#### If running in WSL with Ollama on Windows:

**Problem**: WSL cannot connect to `localhost:11434` because Ollama on Windows only listens on `127.0.0.1`.

**Solution 1 - Configure Ollama to listen on all interfaces** (Recommended):

1. **On Windows**, set the `OLLAMA_HOST` environment variable:

   - Open System Properties → Environment Variables
   - Add User Variable: `OLLAMA_HOST` = `0.0.0.0:11434`
   - Restart Ollama (or reboot Windows)

2. **In WSL**, find the Windows host IP:

   ```bash
   cat /etc/resolv.conf | grep nameserver | awk '{print $2}'
   # Example output: 10.255.255.254
   ```

3. **Start Leader with Windows host IP**:
   ```bash
   ./target/release/axon_cluster serve --ollama-url http://10.255.255.254:11434
   ```

**Solution 2 - Run Ollama in WSL** (Alternative):

Install and run Ollama directly in WSL:

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama serve
# In another terminal
./target/release/axon_cluster serve
```

#### General Ollama troubleshooting:

- Verify Ollama is running: `ollama list`
- Check the model is pulled: `ollama pull qwen:0.5b`
- Test Ollama directly:

  ```bash
  # From WSL to Windows
  curl http://10.255.255.254:11434/api/generate -d '{"model":"qwen:0.5b","prompt":"test","stream":false}'

  # Local test
  curl http://localhost:11434/api/generate -d '{"model":"qwen:0.5b","prompt":"test","stream":false}'
  ```

### Connection Timeout

- Increase timeout in code if needed for slow models
- Ensure Leader has sufficient resources (GPU/RAM)
- Check network latency

## Documentation

- **[WEB_UI.md](WEB_UI.md)** - Complete web interface guide (sidecar pattern, API reference, troubleshooting)
- **[QUICKSTART.md](QUICKSTART.md)** - CLI-based quick start tutorial
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)** - Technical implementation details
- **[WSL_SETUP.md](WSL_SETUP.md)** - WSL-specific configuration and troubleshooting

## Dependencies

- **libp2p**: P2P networking with PSK, Noise, mDNS
- **tokio**: Async runtime with channels (mpsc, oneshot)
- **axum**: HTTP server for web UI
- **tower-http**: CORS middleware
- **reqwest**: HTTP client for Ollama API
- **serde/serde_json**: Serialization
- **clap**: CLI argument parsing
- **anyhow**: Error handling

## Performance Tips

1. **Leader Node**: Use a machine with:

   - NVIDIA GPU (for Ollama acceleration)
   - At least 16GB RAM for larger models
   - Fast SSD for model storage

2. **Network**:

   - Use 5GHz WiFi for better throughput
   - Wired Ethernet for lowest latency

3. **Models**:
   - `llama2:7b` - Fast, good for most tasks
   - `mistral:7b` - Better quality, similar speed
   - `codellama` - Optimized for code generation

## License

This project is provided as-is for educational and research purposes.

## Contributing

Contributions welcome! Please ensure:

- Code compiles without warnings
- Security features remain intact
- Tests pass (if applicable)

## Future Enhancements

- [ ] Multiple Leader support with load balancing
- [ ] Streaming responses for real-time output
- [ ] Web dashboard for monitoring
- [ ] Docker containerization
- [ ] Authentication layer (beyond PSK)
- [ ] Rate limiting and quotas
- [ ] Metrics and observability
