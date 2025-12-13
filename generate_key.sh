#!/bin/bash
# Generate a swarm.key for Axon-Cluster private network

KEY_FILE="swarm.key"

if [ -f "$KEY_FILE" ]; then
    echo "⚠️  Warning: $KEY_FILE already exists!"
    read -p "Overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted. Keeping existing $KEY_FILE"
        exit 0
    fi
fi

echo "🔐 Generating new swarm key..."
echo -e "/key/swarm/psk/1.0.0/\n/base16/" > "$KEY_FILE"
openssl rand -hex 32 >> "$KEY_FILE"

if [ $? -eq 0 ]; then
    echo "✅ Successfully generated $KEY_FILE"
    echo ""
    echo "📋 Key content:"
    cat "$KEY_FILE"
    echo ""
    echo "⚠️  IMPORTANT: Copy this file to all nodes that should join your private network!"
else
    echo "❌ Failed to generate key"
    exit 1
fi
