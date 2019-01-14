/*
Define messages used in the system
*/
use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;


#[derive(Debug)]
pub struct SortingRequest {
    pub values: Vec<i64>,
}

impl Message for SortingRequest {
    type Result = SortingResponse;
}


impl SortingRequest {
    pub fn new(values: Vec<i64>) -> SortingRequest {
        SortingRequest{
            values
        }
    }
}


#[derive(Debug)]
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

impl<A, M> MessageResponse<A, M> for SortingResponse
    where
        A: Actor,
        M: Message<Result = SortingResponse>
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}
