use std::{
    io::{
        Read,
        Write,
    },
    str::from_utf8,
};

use crate::{
    consts::{
        IPC_DIR,
        IPC_PREFIX,
    },
    errors::{
        IpcError,
        IpcResult,
        PacketError,
    },
    Packet,
    Payload,
};
pub trait IpcClient {
    fn open(&mut self) -> IpcResult<()>;
    fn connect(&mut self) -> IpcResult<()>;
    fn reconnect(&self) -> IpcResult<()>;
    fn read(&mut self) -> IpcResult<((u32, u32), String)>;
    fn write(&mut self, packet: Packet) -> IpcResult<()>;
}

pub enum IpcConnectionStatus {
    Connected,
    Disconnected,
}

pub struct DiscordIpcClient {
    app_id: &'static str,
    status: IpcConnectionStatus,
    #[cfg(windows)]
    source: Option<std::fs::File>,
    #[cfg(unix)]
    source: Option<std::os::unix::net::UnixStream>,
}

impl DiscordIpcClient {
    pub fn new(app_id: &'static str) -> Self {
        Self {
            app_id,
            status: IpcConnectionStatus::Disconnected,
            source: None,
        }
    }
}

#[cfg(windows)]
impl IpcClient for DiscordIpcClient {
    fn open(&mut self) -> IpcResult<()> {
        if self.source.is_some() {
            Ok(())
        } else {
            for i in 0..9 {
                let ipc_path = format!("{}{}{}", IPC_DIR, IPC_PREFIX, i);
                if let Ok(file) = std::fs::OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(&ipc_path)
                {
                    return Ok(self.source = Some(file));
                } else {
                    continue;
                }
            }
            Err(IpcError::OpenError(String::from(
                "Couldn't find an availble discord ipc path",
            )))
        }
    }

    fn connect(&mut self) -> IpcResult<()> {
        if self.source.is_none() {
            return Err(IpcError::ConnectionError(String::from(
                "There is no valid source provided, please open a source first",
            )));
        } else {
            let handshake = Payload::Handshake {
                v: 1,
                client_id: self.app_id.to_owned(),
            };
            let packet = match Packet::new(0x0000, handshake) {
                Ok(p) => p,
                Err(_) => return Err(IpcError::HandshakeError(PacketError::SendError)),
            };
            self.write(packet)?;
            Ok(())
        }
    }

    fn reconnect(&self) -> IpcResult<()> {
        todo!()
    }

    fn read(&mut self) -> IpcResult<((u32, u32), String)> {
        if let Some(ref mut source) = &mut self.source {
            let mut header = vec![0u8; 8];
            source.read(&mut header)?;
            let opcode = u32::from_le_bytes(header[..4].try_into().unwrap());
            let length = u32::from_le_bytes(header[4..].try_into().unwrap());
            let mut response = vec![0u8; length as usize];
            source.read(&mut response)?;
            Ok(((opcode, length), from_utf8(&response).unwrap().to_string()))
        } else {
            Err(IpcError::ReadError(String::from(
                "There is no valid source provided, please open a source first",
            )))
        }
    }

    fn write(&mut self, packet: Packet) -> IpcResult<()> {
        if let Some(ref mut source) = &mut self.source {
            let (header, data) = packet.to_bytes()?;
            source.write_all(&header)?;
            source.write_all(&data)?;
            Ok(())
        } else {
            Err(IpcError::WriteError(String::from(
                "There is no valid source provided, please open a source first",
            )))
        }
    }
}
