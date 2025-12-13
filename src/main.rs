use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use libp2p::{
    PeerId, StreamProtocol, Swarm,
    core::{Transport, upgrade},
    identity, mdns, noise,
    pnet::{PnetConfig, PreSharedKey},
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use std::{collections::HashMap, fs, iter, path::Path, time::Duration};

pub mod cli;
pub mod ollama;
pub mod protocol;

use cli::Mode;
use ollama::OllamaClient;
use protocol::{InferenceCodec, InferenceRequest, InferenceResponse};

/// Network behavior combining mDNS and request-response
#[derive(NetworkBehaviour)]
struct AxonBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<InferenceCodec>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    let args = cli::Args::parse();

    // Load the pre-shared key for private network
    let psk_bytes = load_psk()?;

    match args.mode {
        Mode::Serve { ollama_url, model } => {
            // Use OLLAMA_LOCALHOST env var if ollama_url is the default
            let final_url = if ollama_url == "http://localhost:11434"
                || ollama_url == "http://127.0.0.1:11434"
            {
                std::env::var("OLLAMA_LOCALHOST").unwrap_or(ollama_url)
            } else {
                ollama_url
            };
            run_leader(psk_bytes, final_url, model).await?;
        }
        Mode::Ask { prompt } => {
            run_subordinate(psk_bytes, prompt).await?;
        }
    }

    Ok(())
}

/// Load the pre-shared key from swarm.key file
fn load_psk() -> Result<[u8; 32]> {
    let key_path = Path::new("./swarm.key");
    if !key_path.exists() {
        anyhow::bail!(
            "Error: 'swarm.key' not found!\n\
            Generate it with:\n  \
            echo -e \"/key/swarm/psk/1.0.0/\\n/base16/\" > swarm.key && openssl rand -hex 32 >> swarm.key"
        );
    }

    let psk_string = fs::read_to_string(key_path)?;
    let hex_key = psk_string
        .trim()
        .lines()
        .last()
        .ok_or_else(|| anyhow::anyhow!("Key file is empty"))?;
    let decoded_key = hex::decode(hex_key)?;

    if decoded_key.len() != 32 {
        anyhow::bail!(
            "Invalid key length: expected 32 bytes, got {}",
            decoded_key.len()
        );
    }

    let mut psk_bytes = [0u8; 32];
    psk_bytes.copy_from_slice(&decoded_key);

    Ok(psk_bytes)
}

/// Create a libp2p swarm with private network support
fn create_swarm(psk_bytes: [u8; 32]) -> Result<Swarm<AxonBehaviour>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("🔑 Local PeerId: {}", local_peer_id);
    println!("🔒 Private Network: Enabled");

    // Create transport with private network encryption
    let psk = PreSharedKey::new(psk_bytes);

    let transport = tcp::tokio::Transport::new(tcp::Config::new().nodelay(true))
        .and_then({
            let psk = psk.clone();
            move |socket, _| {
                let pnet_config = PnetConfig::new(psk.clone());
                pnet_config.handshake(socket)
            }
        })
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    // Create request-response behavior
    let cfg = request_response::Config::default().with_request_timeout(Duration::from_secs(120));

    let protocol = StreamProtocol::new("/axon/inference/1.0.0");
    let request_response = request_response::Behaviour::with_codec(
        InferenceCodec,
        iter::once((protocol, ProtocolSupport::Full)),
        cfg,
    );

    // Create mDNS for local network discovery
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

    let behaviour = AxonBehaviour {
        mdns,
        request_response,
    };

    let swarm = Swarm::new(
        transport,
        behaviour,
        local_peer_id,
        libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(Duration::from_secs(60)),
    );

    Ok(swarm)
}

/// Run in Leader mode (server)
async fn run_leader(psk_bytes: [u8; 32], ollama_url: String, model: String) -> Result<()> {
    println!("🚀 Starting Leader Mode (Server)");
    println!("📡 Ollama URL: {}", ollama_url);
    println!("🤖 Model: {}", model);

    let mut swarm = create_swarm(psk_bytes)?;

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let ollama_client = OllamaClient::new(ollama_url);

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, _addr) in peers {
                    println!("🔍 Discovered peer: {}", peer_id);
                }
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    message:
                        request_response::Message::Request {
                            request, channel, ..
                        },
                    ..
                },
            )) => {
                println!("📨 Received inference request: {:?}", request.prompt);

                // Process the inference request with Ollama
                let model_name = request.model.unwrap_or_else(|| model.clone());
                let response = match ollama_client.generate(request.prompt, model_name).await {
                    Ok(text) => InferenceResponse {
                        response: text,
                        success: true,
                        error: None,
                    },
                    Err(e) => InferenceResponse {
                        response: String::new(),
                        success: false,
                        error: Some(format!("{}", e)),
                    },
                };

                println!("✅ Sending response back");
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response)
                    .ok();
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _addr) in peers {
                    println!("❌ Peer expired: {}", peer_id);
                }
            }
            _ => {}
        }
    }
}

/// Run in Subordinate mode (client)
async fn run_subordinate(psk_bytes: [u8; 32], prompt: String) -> Result<()> {
    println!("🚀 Starting Subordinate Mode (Client)");
    println!("💭 Prompt: {}", prompt);

    let mut swarm = create_swarm(psk_bytes)?;

    // Listen on a random port for incoming connections
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut pending_request: Option<OutboundRequestId> = None;
    let mut discovered_leaders: HashMap<PeerId, bool> = HashMap::new();

    println!("🔍 Discovering Leader nodes...");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, _addr) in peers {
                    if !discovered_leaders.contains_key(&peer_id) {
                        println!("🎯 Found Leader: {}", peer_id);
                        discovered_leaders.insert(peer_id, false);

                        // Send the inference request
                        if pending_request.is_none() {
                            println!("📤 Sending inference request to Leader...");
                            let request = InferenceRequest {
                                prompt: prompt.clone(),
                                model: None,
                            };

                            let req_id = swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, request);
                            pending_request = Some(req_id);
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    message: request_response::Message::Response { response, .. },
                    ..
                },
            )) => {
                if response.success {
                    println!("\n✅ Response from Leader:\n");
                    println!("{}", response.response);
                } else {
                    eprintln!(
                        "\n❌ Error from Leader: {}",
                        response.error.unwrap_or_default()
                    );
                }
                return Ok(());
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure { error, .. },
            )) => {
                eprintln!("❌ Request failed: {:?}", error);
                return Err(anyhow::anyhow!("Request failed: {:?}", error));
            }
            SwarmEvent::Behaviour(AxonBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _addr) in peers {
                    println!("❌ Leader disconnected: {}", peer_id);
                    discovered_leaders.remove(&peer_id);
                }
            }
            _ => {}
        }
    }
}
