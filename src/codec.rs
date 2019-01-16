#![allow(dead_code)]
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use std::io;
use tokio_io::codec::{Decoder, Encoder};


/// Supervisor -> SortingActor request
#[derive(Serialize, Deserialize, Debug, Message)]
pub struct SortingRequest {
    pub values: Vec<i64>,
}

impl SortingRequest {
    pub fn new(values: Vec<i64>) -> SortingRequest {
        SortingRequest{
            values
        }
    }
}


/// SortingActor -> Supervisor response
#[derive(Serialize, Deserialize, Debug, Message)]
pub struct SortingResponse {
    pub values: Vec<i64>,
    pub duration: i64
}

impl SortingResponse {
    pub fn new(values: Vec<i64>, duration: i64) -> SortingResponse {
        SortingResponse{
            values,
            duration
        }
    }
}


/// Codec for SortingActor -> Supervisor transport
pub struct SortingActorToSupervisorCodec;

impl Encoder for SortingActorToSupervisorCodec {
    type Item = SortingResponse;
    type Error = io::Error;

    fn encode(&mut self, msg: SortingResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 8);
        dst.put_u64_be(msg_ref.len() as u64);
        dst.put(msg_ref);

        Ok(())
    }
}

impl Decoder for SortingActorToSupervisorCodec {
    type Item = SortingRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 8 {
                return Ok(None);
            }
            BigEndian::read_u64(src.as_ref()) as usize
        };

        if src.len() >= size + 8 {
            src.split_to(8);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<SortingRequest>(&buf)?))
        } else {
            Ok(None)
        }
    }
}


/// Codec for Supervisor -> SortingActor transport
pub struct SupervisorToSortingActorCodec;

impl Encoder for SupervisorToSortingActorCodec {
    type Item = SortingRequest;
    type Error = io::Error;

    fn encode(&mut self, msg: SortingRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 8);
        dst.put_u64_be(msg_ref.len() as u64);
        dst.put(msg_ref);

        Ok(())
    }
}

impl Decoder for SupervisorToSortingActorCodec {
    type Item = SortingResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 8 {
                return Ok(None);
            }
            BigEndian::read_u64(src.as_ref()) as usize
        };


        if src.len() >= size + 8 {
            src.split_to(8);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<SortingResponse>(&buf)?))
        } else {
            Ok(None)
        }
    }
}
