//! Protocol definitions for Axon-Cluster inference requests

use async_trait::async_trait;
use libp2p::{StreamProtocol, request_response::Codec};
use serde::{Deserialize, Serialize};
use std::io;

/// Request sent from Subordinate to Leader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub model: Option<String>,
}

/// Response sent from Leader to Subordinate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub response: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Codec for encoding/decoding inference messages
#[derive(Debug, Clone)]
pub struct InferenceCodec;

#[async_trait]
impl Codec for InferenceCodec {
    type Protocol = StreamProtocol;
    type Request = InferenceRequest;
    type Response = InferenceResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        let mut length_bytes = [0u8; 4];
        io.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut buffer = vec![0u8; length];
        io.read_exact(&mut buffer).await?;

        serde_json::from_slice(&buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        let mut length_bytes = [0u8; 4];
        io.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut buffer = vec![0u8; length];
        io.read_exact(&mut buffer).await?;

        serde_json::from_slice(&buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        let data =
            serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let length = data.len() as u32;
        io.write_all(&length.to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        let data =
            serde_json::to_vec(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let length = data.len() as u32;
        io.write_all(&length.to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.close().await?;

        Ok(())
    }
}
