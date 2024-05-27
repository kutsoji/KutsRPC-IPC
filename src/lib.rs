mod consts;
mod errors;
mod ipc;
mod payload;
use errors::PacketResult;
pub use ipc::{
    DiscordIpcClient,
    IpcClient,
};
pub use payload::*;

#[derive(Debug)]
pub struct Header {
    pub opcode: u32,
    pub length: u32,
}

impl Header {
    pub fn to_bytes(&self) -> Vec<u8> {
        [self.opcode.to_le_bytes(), self.length.to_le_bytes()].concat()
    }
}

#[derive(Debug)]
pub struct Packet {
    header: Header,
    payload: Payload,
}

impl Packet {
    pub fn new(opcode: u32, payload: Payload) -> PacketResult<Self> {
        let length = payload.get_length()?;
        Ok(Self {
            header: Header { opcode, length },
            payload,
        })
    }

    pub fn to_bytes(&self) -> PacketResult<(Vec<u8>, Vec<u8>)> {
        let header = self.header.to_bytes();
        let data = self.payload.to_bytes()?;
        Ok((header, data))
    }
}
