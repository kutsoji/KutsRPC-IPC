use serde_json::json;

use crate::{
    consts::IPC_PREFIX,
    errors::{
        IpcError,
        IpcResult,
        PacketError,
    },
    events::{
        Event,
        EventHandler,
    },
    Activity,
    Header,
    Packet,
    Payload,
};
use std::{
    io::{
        Read,
        Write,
    },
    str::from_utf8,
};
pub trait IpcClient {
    fn open(&mut self) -> IpcResult<()>;
    fn read(&mut self) -> IpcResult<Packet>;
    fn write(&mut self, packet: Packet) -> IpcResult<()>;
}

pub struct DiscordIpcClient {
    app_id: &'static str,
    connected: bool,
    event_handler: EventHandler,
    #[cfg(windows)]
    source: Option<std::fs::File>,
    #[cfg(unix)]
    source: Option<std::os::unix::net::UnixStream>,
}

impl DiscordIpcClient {
    pub fn new(app_id: &'static str) -> Self {
        Self {
            app_id,
            connected: false,
            source: None,
            event_handler: EventHandler::new(),
        }
    }

    pub fn connect(&mut self) -> IpcResult<()> {
        if self.source.is_none() {
            Err(IpcError::ConnectionError(String::from(
                "There is no valid source provided, please open a source first",
            )))
        } else if self.connected {
            Err(IpcError::ConnectionError(String::from("Already connected")))
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

    pub fn reconnect(&self) -> IpcResult<()> {
        todo!()
    }

    pub fn set_activity(&mut self, activity: Activity) -> IpcResult<()> {
        self.write(Packet::new(
            0x0001,
            Payload::OutGoingCommand {
                cmd: "SET_ACTIVITY".into(),
                nonce: uuid::Uuid::new_v4().as_u128() as i64,
                args: json!({
                    "pid": std::process::id(),
                    "activity": serde_json::to_value(activity)?
                }),
                evt: None,
            },
        )?)?;
        Ok(())
    }

    pub fn clear_activity(&mut self) -> IpcResult<()> {
        self.write(Packet::new(
            0x0001,
            Payload::OutGoingCommand {
                cmd: "SET_ACTIVITY".into(),
                nonce: uuid::Uuid::new_v4().as_u128() as i64,
                args: json!({
                    "pid": std::process::id()
                }),
                evt: None,
            },
        )?)?;
        Ok(())
    }

    pub fn on<Callback>(&self, ev: Event, mut cb: Callback) -> IpcResult<()>
    where
        Callback: FnMut(Payload) + Send + 'static,
    {
        if self.connected {
            self.event_handler.listen(
                move |event| {
                    cb(event);
                },
                ev,
            );
            Ok(())
        } else {
            Err(IpcError::EventError)
        }
    }

    pub fn disconnect(&mut self) -> IpcResult<()> {
        self.write(Packet::new(0x0002, Payload::Empty {})?)?;
        self.connected = false;
        self.event_handler.stop();
        Ok(())
    }
}

#[cfg(windows)]
impl IpcClient for DiscordIpcClient {
    fn open(&mut self) -> IpcResult<()> {
        if self.source.is_some() {
            Ok(())
        } else {
            for i in 0..9 {
                let ipc_path = format!("{}{}{}", crate::consts::IPC_DIR, IPC_PREFIX, i);
                if let Ok(file) = std::fs::OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(&ipc_path)
                {
                    return {
                        self.source = Some(file);
                        Ok(())
                    };
                } else {
                    continue;
                }
            }
            Err(IpcError::OpenError(String::from(
                "Couldn't find an available discord ipc path",
            )))
        }
    }

    fn read(&mut self) -> IpcResult<Packet> {
        if let Some(ref mut source) = &mut self.source {
            let mut header = vec![0u8; 8];
            source.read_exact(&mut header)?;
            let opcode = u32::from_le_bytes(header[..4].try_into().unwrap());
            let length = u32::from_le_bytes(header[4..].try_into().unwrap());
            let mut response = vec![0u8; length as usize];
            source.read_exact(&mut response)?;
            Ok(Packet {
                header: Header { opcode, length },
                payload: serde_json::from_str::<Payload>(from_utf8(&response).unwrap())?,
            })
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

            let res = self.read()?;
            match res.payload {
                Payload::InComingCommand {
                    cmd: _,
                    nonce: _,
                    args: _,
                    data: _,
                    evt: Some(ref event),
                } => {
                    let ev_clone = event.clone();
                    self.event_handler.emit(ev_clone.clone(), res.payload);
                    if ev_clone == Event::Ready {
                        self.connected = true;
                    }
                }
                Payload::CriticalError { code: _, message } => {
                    return Err(IpcError::CriticalError(message));
                }
                Payload::Empty {} => {
                    if res.header.opcode == 0x0003 {
                        self.write(Packet::new(0x0004, Payload::Empty {})?)?
                    }
                }
                _ => (),
            }

            Ok(())
        } else {
            Err(IpcError::WriteError(String::from(
                "There is no valid source provided, please open a source first",
            )))
        }
    }
}
