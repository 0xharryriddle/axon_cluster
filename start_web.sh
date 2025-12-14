#!/bin/bash
# Start script for Axon Cluster Web UI

set -e

echo "🧠⚡ Starting Axon Cluster Web UI"
echo "=================================="
echo ""

# Check if swarm.key exists
if [ ! -f "swarm.key" ]; then
    echo "⚠️  swarm.key not found. Generating..."
    ./generate_key.sh
    echo ""
fi

# Check if .env exists for WSL users
if [ -f "/proc/version" ] && grep -qi microsoft /proc/version; then
    if [ ! -f ".env" ]; then
        echo "💡 Detected WSL environment. Creating .env file..."
        WINDOWS_IP=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
        echo "OLLAMA_LOCALHOST=http://$WINDOWS_IP:11434" > .env
        echo "✅ Created .env with Windows IP: $WINDOWS_IP"
        echo ""
    fi
fi

# Build Rust backend if needed
if [ ! -f "target/release/axon_cluster" ]; then
    echo "🔨 Building Rust backend..."
    cargo build --release
    echo ""
fi

# Install npm dependencies if needed
if [ ! -d "web-app/node_modules" ]; then
    echo "📦 Installing frontend dependencies..."
    cd web-app && npm install && cd ..
    echo ""
fi

# Start backend in background
echo "🚀 Starting Rust backend (Leader with HTTP API)..."
./target/release/axon_cluster web &
BACKEND_PID=$!

# Give backend time to start
sleep 2

# Start frontend
echo "🌐 Starting React frontend..."
echo ""
echo "=================================="
echo "✅ Web UI ready!"
echo "Backend:  http://localhost:3000/api"
echo "Frontend: http://localhost:5173"
echo "=================================="
echo ""

cd web-app && npm run dev

# Cleanup on exit
trap "kill $BACKEND_PID 2>/dev/null" EXIT
