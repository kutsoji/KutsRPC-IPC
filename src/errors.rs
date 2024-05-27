use thiserror::Error;

#[derive(Error, Debug)]
pub enum PayloadError {
    #[error("Failed to convert the payload into a JSON value: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Invalid payload")]
    ToJsonError,
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Failed to create packet due to a payload error: {0}")]
    CreationError(#[from] PayloadError),

    #[error("Failed to send packet")]
    SendError,

    #[error("Failed to receive packet")]
    ReceiveError,
}

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Failed to open an ipc connection: {0}")]
    OpenError(String),

    #[error("Failed to connect to discord ipc: {0}")]
    ConnectionError(String),

    #[error("{0}")]
    HandshakeError(#[from] PacketError),

    #[error("Failed to reconnect to discord ipc: {0}")]
    ReconnectionError(String),

    #[error("Failed to read from discord ipc: {0}")]
    ReadError(String),

    #[error("Failed to write from discord ipc: {0}")]
    WriteError(String),

    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
}

pub(crate) type PayloadResult<T> = Result<T, PayloadError>;
pub(crate) type PacketResult<T> = Result<T, PacketError>;
pub(crate) type IpcResult<T> = Result<T, IpcError>;
