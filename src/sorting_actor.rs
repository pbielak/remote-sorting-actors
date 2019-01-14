/*
Define the sorting actor
*/
use actix::prelude::*;

use super::messages;
use super::util;



#[derive(Debug)]
pub struct SortingActor {
    pub id: String
}

impl SortingActor {
    pub fn new(id: &str) -> SortingActor {
        SortingActor {
            id: String::from(id)
        }
    }

    pub fn sort_vec<T: Clone + Ord>(&self, v: Vec<T>) -> Vec<T> {
        let mut vals = v.clone();
        vals.sort();
        vals
    }
}

impl Actor for SortingActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        debug!("[{}] SortingActor ~ START", self.id)
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("[{}] SortingActor ~ STOP", self.id)
    }
}

impl Handler<messages::SortingRequest> for SortingActor {
    type Result = messages::SortingResponse;


    fn handle(&mut self, msg: messages::SortingRequest, _: &mut Self::Context) -> Self::Result {
        debug!("[SortingActor][{}] Got sorting request: Vec[{}]", self.id, msg.values.len());

        let (vals, duration) = util::measure_time(&|vals| self.sort_vec(vals), msg.values);

        debug!("[SortingActor][{}] Done sorting - duration {} (ms)", self.id, duration);

        messages::SortingResponse::new(vals, duration)
    }
}
