/*
Define the supervisor actor
*/
#[macro_use] extern crate actix;
extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate structopt;


use std::time::Duration;
use std::net;
use std::str::FromStr;
use std::path::PathBuf;
use std::io;

use actix::prelude::*;
use futures::Stream;
use structopt::StructOpt;
use tokio_io::codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::{TcpListener, TcpStream};

mod codec;
mod util;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct CliArgs {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(short = "n", long = "nb-actors")]
    pub n: usize,

    #[structopt(short = "k", long = "nb-chunks")]
    pub k: usize,

    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    #[structopt(long = "addr")]
    pub addr: String,
}


type WriteStream = actix::io::FramedWrite<WriteHalf<TcpStream>, codec::SupervisorToSortingActorCodec>;

pub struct SupervisorActor {
    pub addr: String,
    num_actors: usize,
    num_chunks: usize,

    sorting_actors: Vec<WriteStream>,

    processed_chunks: usize,
    sorted_values: Vec<i64>,
}

impl SupervisorActor {
    pub fn new(addr: String, num_actors: usize, num_chunks: usize) -> SupervisorActor {
        let sorting_actors = vec![];
        let sorted_values = vec![];
        let processed_chunks = 0;

        SupervisorActor {
            addr,
            num_actors,
            num_chunks,
            sorting_actors,
            processed_chunks,
            sorted_values,
        }
    }

    pub fn sort_values(&mut self, values: Vec<i64>, ctx: &mut Context<Self>) {
        if self.sorting_actors.len() < self.num_actors {
            println!("Not enough sorting actors");

            ctx.run_later(Duration::new(1, 0), |act, ctx| {
                act.sort_values(values, ctx);
            });

            return
        }

        let chunks = split_vec(&values, self.num_chunks);
        let assignments = round_robin_assign((0..self.sorting_actors.len()).collect(), chunks);

        for (actor_idx, chunk) in assignments {
            self.sorting_actors[actor_idx].write(codec::SortingRequest::new(chunk.to_vec()))
        }

        println!("Sent to workers");
    }
}

impl Actor for SupervisorActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        println!("[{}] SupervisorActor ~ START", self.addr);
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        println!("[{}] SupervisorActor ~ STOPPING", self.addr);

        System::current().stop();

        Running::Stop
    }


    fn stopped(&mut self, _: &mut Self::Context) {
        println!("[{}] SupervisorActor ~ STOP", self.addr);
    }
}

impl Handler<codec::SortingRequest> for SupervisorActor {
    type Result = ();


    fn handle(&mut self, msg: codec::SortingRequest, ctx: &mut Context<Self>) {
        let in_vec = msg.values;

        println!("[SupervisorActor] Got sorting request: Vec[{}]", in_vec.len());

        self.sort_values(in_vec, ctx);
    }
}

impl actix::io::WriteHandler<io::Error> for SupervisorActor {}

#[derive(Debug, Message)]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);


impl Handler<TcpConnect> for SupervisorActor {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, ctx: &mut Context<Self>) {
        println!("TCPConnect from: {:?}", msg.1);
        let (r, w) = msg.0.split();
        SupervisorActor::add_stream(FramedRead::new(r, codec::SupervisorToSortingActorCodec), ctx);

        self.sorting_actors.push(actix::io::FramedWrite::new(w, codec::SupervisorToSortingActorCodec, ctx));
    }
}

impl StreamHandler<codec::SortingResponse, io::Error> for SupervisorActor {
    fn handle(&mut self, msg: codec::SortingResponse, _: &mut Context<Self>) {
        println!("Supervisor got: {:?} took {}", msg.values, msg.duration);

        if self.sorted_values.is_empty() {
            self.sorted_values = msg.values;
        }
        else {
            self.sorted_values = merge(&self.sorted_values, &msg.values);
        }

        self.processed_chunks += 1;

        if self.processed_chunks == self.num_chunks {
            println!("Done with sorting: {:?}", self.sorted_values);
        }
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


fn main() {
    actix::System::run(|| {
        let args = CliArgs::from_args();

        let myaddr = args.addr;
        let n = args.n;
        let k = args.k;

        let addr = net::SocketAddr::from_str(&myaddr).unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let supervisor = SupervisorActor::create(move |ctx| {
            ctx.add_message_stream(listener
                .incoming()
                .map_err(|_| ())
                .map(|st| {
                    let sorting_actor_addr = st.peer_addr().unwrap();
                    TcpConnect(st, sorting_actor_addr)
                })
            );

            SupervisorActor::new(myaddr, n, k)
        });

        let sort_req = codec::SortingRequest::new(vec![6, 5, 4, 3, 2, 1]);
        supervisor.do_send(sort_req);
    });
}