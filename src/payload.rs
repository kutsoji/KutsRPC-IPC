use std::fmt;

use errors::{
    PayloadError,
    PayloadResult,
};
use serde::{
    de::{
        self,
        MapAccess,
        Visitor,
    },
    Deserialize,
    Deserializer,
    Serialize,
};
use serde_json::{
    json,
    Value,
};

use crate::{
    errors,
    events::Event,
};

#[derive(Debug, Serialize, Clone)]
pub enum Payload {
    Handshake {
        v: u8,
        client_id: String,
    },
    OutGoingCommand {
        cmd: String,
        nonce: i64,
        args: serde_json::Value,
        evt: Option<Event>,
    },
    InComingCommand {
        cmd: String,
        nonce: i64,
        args: Option<serde_json::Value>,
        data: serde_json::Value,
        evt: Option<Event>,
    },
    CriticalError {
        code: u32,
        message: String,
    },
    Empty {},
}

impl Payload {
    pub fn get_length(&self) -> PayloadResult<u32> {
        Ok(self.to_string()?.len() as u32)
    }

    fn to_string(&self) -> PayloadResult<String> {
        Ok(self.to_json()?.to_string())
    }

    pub fn to_json(&self) -> PayloadResult<Value> {
        let full_json_payload = serde_json::to_value(self)?;
        if let Value::Object(map) = full_json_payload {
            if let Some((_, body)) = map.iter().next() {
                Ok(serde_json::to_value(body.clone())?)
            } else {
                Ok(json!(""))
            }
        } else {
            Err(PayloadError::ToJsonError)
        }
    }

    pub fn to_bytes(&self) -> PayloadResult<Vec<u8>> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PayloadVisitor;

        impl<'de> Visitor<'de> for PayloadVisitor {
            type Value = Payload;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid Payload")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut v = None;
                let mut client_id = None;
                let mut cmd = None;
                let mut nonce = None;
                let mut args = None;
                let mut data = None;
                let mut evt = None;
                let mut code = None;
                let mut message = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "v" => {
                            if v.is_some() {
                                return Err(de::Error::duplicate_field("v"));
                            }
                            v = Some(map.next_value()?);
                        }
                        "client_id" => {
                            if client_id.is_some() {
                                return Err(de::Error::duplicate_field("client_id"));
                            }
                            client_id = Some(map.next_value()?);
                        }
                        "cmd" => {
                            if cmd.is_some() {
                                return Err(de::Error::duplicate_field("cmd"));
                            }
                            cmd = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value::<Option<i64>>()?.unwrap_or_default());
                        }
                        "args" => {
                            if args.is_some() {
                                return Err(de::Error::duplicate_field("args"));
                            }
                            args = Some(map.next_value()?);
                        }
                        "data" => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        "evt" => {
                            if evt.is_some() {
                                return Err(de::Error::duplicate_field("evt"));
                            }
                            evt = Some(map.next_value::<Option<Event>>()?);
                        }
                        "code" => {
                            if code.is_some() {
                                return Err(de::Error::duplicate_field("code"));
                            }
                            code = Some(map.next_value()?);
                        }
                        "message" => {
                            if message.is_some() {
                                return Err(de::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                        _ => {
                            let _: Value = map.next_value()?;
                        }
                    }
                }

                if let (Some(v), Some(client_id)) = (v, client_id) {
                    Ok(Payload::Handshake { v, client_id })
                } else if let (Some(cmd), Some(nonce), Some(data), Some(evt)) =
                    (cmd.clone(), nonce, data, evt.clone())
                {
                    Ok(Payload::InComingCommand {
                        cmd,
                        nonce,
                        args,
                        data,
                        evt,
                    })
                } else if let (Some(cmd), Some(nonce), Some(args), Some(evt)) =
                    (cmd, nonce, args, evt)
                {
                    Ok(Payload::OutGoingCommand {
                        cmd,
                        nonce,
                        args,
                        evt,
                    })
                } else if let (Some(code), Some(message)) = (code, message) {
                    Ok(Payload::CriticalError { code, message })
                } else {
                    Ok(Payload::Empty {})
                }
            }
        }

        deserializer.deserialize_map(PayloadVisitor)
    }
}
