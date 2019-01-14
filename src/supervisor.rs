/*
Define the supervisor actor
*/
use actix::prelude::*;
use futures::Future;

use super::messages;
use super::sorting_actor;
use super::util;


#[derive(Debug)]
pub struct SupervisorActor {
    pub id: String,
    num_actors: usize,
    num_chunks: usize,

    sorting_actors: Vec<Addr<sorting_actor::SortingActor>>,
}

impl SupervisorActor {
    pub fn new(id: &str, num_actors: usize, num_chunks: usize) -> SupervisorActor {
        let id = String::from(id);
        let sorting_actors = vec![];

        SupervisorActor {
            id,
            num_actors,
            num_chunks,
            sorting_actors,
        }
    }

    pub fn spawn_sorting_actors(&mut self) {
        for i in 0..self.num_actors {
            self.sorting_actors.push(
                Arbiter::start(move |_| {
                    sorting_actor::SortingActor::new(&format!("SLAVE-{}", i))
                })
            );
        }
    }

    pub fn sort_values(&self, numbers: Vec<i64>) -> Vec<i64> {
        type Tasks = Vec<Request<sorting_actor::SortingActor, messages::SortingRequest>>;

        let chunks = split_vec(&numbers, self.num_chunks);
        let actor_pool = self.sorting_actors.clone();

        let tasks: Tasks = round_robin_assign(actor_pool, chunks).into_iter().map(|ac| {
            let (actor, chunk) = ac;
            actor.send(messages::SortingRequest::new(chunk.to_vec()))
        }).collect();

        let sorted_values = tasks.into_iter().fold(vec![], |acc, task| {
            let res: messages::SortingResponse = task.wait().unwrap();
            let (merged, duration) = util::measure_time(&|v| merge(&acc, v), &res.values);

            debug!("[Supervisor][{}] Merge took: {} (ms)", self.id, duration);

            merged
        });

        sorted_values
    }
}

impl Actor for SupervisorActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        debug!("[{}] SupervisorActor ~ START", self.id);
        self.spawn_sorting_actors();
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("[{}] SupervisorActor ~ STOP", self.id);
    }
}

impl Handler<messages::SortingRequest> for SupervisorActor {
    type Result = messages::SortingResponse;


    fn handle(&mut self, msg: messages::SortingRequest, _: &mut Context<Self>) -> Self::Result {
        let in_vec = msg.values;

        debug!("[SupervisorActor] Got sorting request: Vec[{}]", in_vec.len());

        let (sorted_values, duration) = util::measure_time(&|vals| self.sort_values(vals), in_vec);

        messages::SortingResponse::new(sorted_values, duration)
    }
}

fn split_vec<T: Clone>(v: &Vec<T>, num_chunks: usize) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = Vec::new();
    let chunk_size = v.len() / num_chunks;

    if chunk_size == 0 {
        panic!("Chunk size is 0")
    }

    for c in v.chunks(chunk_size) {
        result.push(c.to_vec())
    }

    result
}


fn round_robin_assign<A: Clone, V>(actors: Vec<A>, chunks: Vec<V>) -> Vec<(A, V)> {
    actors
        .into_iter()
        .cycle()
        .take(chunks.len())
        .zip(chunks)
        .collect()
}


fn merge(v1: &Vec<i64>, v2: &Vec<i64>) -> Vec<i64> {
    let mut i = 0;
    let mut j = 0;

    let total = v1.len() + v2.len();
    let mut result: Vec<i64> = Vec::with_capacity(total);

    while result.len() != total {
        if i == v1.len() {
            result.extend_from_slice(&v2[j..]);
            break;
        }

        else if j == v2.len() {
            result.extend_from_slice(&v1[i..]);
            break;
        }

        else if v1[i] < v2[j] {
            result.push(v1[i]);
            i += 1;
        }

        else {
            result.push(v2[j]);
            j += 1;
        }
    }

    result
}
