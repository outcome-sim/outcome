//! Message definitions.

#![allow(unused)]

use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::*;

pub mod coord_worker;
pub mod server_client;

mod query;

pub use server_client::*;

use crate::socket::{pack, unpack, Encoding};
use crate::{error::Error, Result, TaskId};

/// Enumeration of all available message types.
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum MessageType {
    Bytes,
    Heartbeat,
    Disconnect,
    Connect,

    PingRequest,
    PingResponse,

    RegisterClientRequest,
    RegisterClientResponse,

    IntroduceCoordRequest,
    IntroduceCoordResponse,
    IntroduceWorkerToCoordRequest,
    IntroduceWorkerToCoordResponse,

    ExportSnapshotRequest,
    ExportSnapshotResponse,

    RegisterRequest,
    RegisterResponse,

    StatusRequest,
    StatusResponse,

    NativeQueryRequest,
    NativeQueryResponse,
    QueryRequest,
    QueryResponse,

    DataTransferRequest,
    DataTransferResponse,
    TypedDataTransferRequest,
    TypedDataTransferResponse,

    JsonPullRequest,
    JsonPullResponse,
    DataPullRequest,
    DataPullResponse,
    TypedDataPullRequest,
    TypedDataPullResponse,

    ScheduledDataTransferRequest,
    ScheduledDataTransferResponse,

    TurnAdvanceRequest,
    TurnAdvanceResponse,

    SpawnEntitiesRequest,
    SpawnEntitiesResponse,
}

/// Self-described message structure wrapping a byte payload.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Message {
    /// Integer identifier allowing for custom message filtering
    pub task_id: TaskId,
    /// Describes what is stored within the payload
    pub type_: MessageType,
    /// Byte representation of the message payload
    pub payload: Vec<u8>,
}

/// Takes a payload struct and turns it directly into a serialized message.
pub(crate) fn msg_bytes_from_payload<P>(
    payload: P,
    task_id: TaskId,
    encoding: &Encoding,
) -> Result<Vec<u8>>
where
    P: Serialize,
    P: Payload,
{
    match encoding {
        Encoding::Bincode => {
            // let msg_bytes = prefix_with_msg_code(payload_bytes, type_);
            let msg = Message {
                task_id,
                type_: payload.type_(),
                payload: pack_payload(payload, encoding)?,
            };
            Ok(bincode::serialize(&msg)?)
        }
        #[cfg(feature = "msgpack_encoding")]
        Encoding::MsgPack => {
            let type_ = payload.type_();
            let payload_bytes = pack_payload(payload, encoding)?;
            let msg = Message {
                task_id,
                type_,
                payload: payload_bytes,
            };
            let msg_bytes = pack(msg, encoding)?;
            // let msg_bytes = prefix_with_msg_code(payload_bytes, type_);
            Ok(msg_bytes)
        }
        _ => unimplemented!(),
    }
}

impl Message {
    /// Creates a complete `Message` from a payload struct.
    pub fn from_payload<P>(payload: P, encoding: &Encoding) -> Result<Message>
    where
        P: Clone,
        P: Serialize,
        P: Payload,
    {
        let msg_type = payload.type_();
        let bytes = pack_payload(payload, encoding)?;
        Ok(Message {
            task_id: 0,
            type_: msg_type,
            payload: bytes,
        })
    }

    /// Deserializes from bytes.
    pub fn from_bytes(mut bytes: Vec<u8>, encoding: &Encoding) -> Result<Message> {
        unpack(&bytes, encoding)
    }

    /// Serializes into bytes.
    pub fn to_bytes(mut self, encoding: &Encoding) -> Result<Vec<u8>> {
        Ok(pack(self, encoding)?)
    }

    /// Unpacks message payload into a payload struct of provided type.
    pub fn unpack_payload<'de, P: Payload + Deserialize<'de>>(
        &'de self,
        encoding: &Encoding,
    ) -> Result<P> {
        let unpacked = unpack(&self.payload, encoding)?;
        Ok(unpacked)
    }
}

// /// Prefixes a payload with a one byte code representing message type.
// pub(crate) fn prefix_with_msg_code(mut payload: Vec<u8>, type_: MessageType) -> Vec<u8> {
//     payload.insert(0, type_ as u8);
//     payload
// }
//
// pub(crate) fn prefix_with_task_id(mut payload: Vec<u8>, task_id: TaskId) -> Vec<u8> {
//     payload.insert(0, task_id.to_be_bytes());
//     payload
// }

/// Packs a payload struct to bytes.
pub(crate) fn pack_payload<P: Payload + Serialize>(
    payload: P,
    encoding: &Encoding,
) -> Result<Vec<u8>> {
    let packed = pack(payload, encoding)?;
    Ok(packed)
}

pub trait Payload: Clone {
    /// Allows payload message structs to state their message type.
    fn type_(&self) -> MessageType;
}

/// Version of the `Var` struct used for untagged ser/deser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VarJson {
    String(String),
    Int(outcome::Int),
    Float(outcome::Float),
    Bool(bool),
    Byte(u8),
    StringList(Vec<String>),
    IntList(Vec<outcome::Int>),
    FloatList(Vec<outcome::Float>),
    BoolList(Vec<bool>),
    ByteList(Vec<u8>),
    #[cfg(feature = "grids")]
    StringGrid(Vec<Vec<String>>),
    #[cfg(feature = "grids")]
    IntGrid(Vec<Vec<outcome::Int>>),
    #[cfg(feature = "grids")]
    FloatGrid(Vec<Vec<outcome::Float>>),
    #[cfg(feature = "grids")]
    BoolGrid(Vec<Vec<bool>>),
    #[cfg(feature = "grids")]
    ByteGrid(Vec<Vec<u8>>),
}

impl From<outcome::Var> for VarJson {
    fn from(var: outcome::Var) -> Self {
        match var {
            outcome::Var::String(v) => VarJson::String(v),
            outcome::Var::Int(v) => VarJson::Int(v),
            outcome::Var::Float(v) => VarJson::Float(v),
            outcome::Var::Bool(v) => VarJson::Bool(v),
            outcome::Var::Byte(v) => VarJson::Byte(v),
            _ => unimplemented!(),
        }
    }
}
impl Into<outcome::Var> for VarJson {
    fn into(self) -> outcome::Var {
        match self {
            VarJson::String(v) => outcome::Var::String(v),
            VarJson::Int(v) => outcome::Var::Int(v),
            VarJson::Float(v) => outcome::Var::Float(v),
            VarJson::Bool(v) => outcome::Var::Bool(v),
            VarJson::Byte(v) => outcome::Var::Byte(v),
            _ => unimplemented!(),
        }
    }
}
