#[macro_use] extern crate actix;
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
#[macro_use] extern crate log;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate tokio_tcp;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate structopt;

use std::str::FromStr;
use std::{io, net, process};

use actix::prelude::*;
use futures::Future;
use structopt::StructOpt;
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;

mod args;
mod codec;
mod messages;
mod util;


fn main() {
    info!("Starting SortingActor");

    actix::System::run(|| {
        let args = args::SortingActorCliArgs::from_args();
        util::setup_logger(args.debug);

        let supervisor_addr = net::SocketAddr::from_str(&args.supervisor_addr).unwrap();

        Arbiter::spawn(
            TcpStream::connect(&supervisor_addr)
                .and_then(|stream| {
                    SortingActor::create(|ctx| {
                        let my_addr = stream.local_addr().unwrap().to_string();
                        let (r, w) = stream.split();

                        ctx.add_stream(FramedRead::new(r, codec::SortingActorToSupervisorCodec));

                        SortingActor::new(
                            actix::io::FramedWrite::new(w, codec::SortingActorToSupervisorCodec, ctx),
                            my_addr,
                        )
                    });

                    futures::future::ok(())
                })
                .map_err(|e| {
                    error!("Can not connect to server: {}", e);
                    process::exit(1)
                })
        );
    });
}


type WriteStream = actix::io::FramedWrite<WriteHalf<TcpStream>, codec::SortingActorToSupervisorCodec>;

struct SortingActor {
    addr: String,
    framed: WriteStream,
}

impl SortingActor {
    pub fn new(framed: WriteStream, addr: String) -> SortingActor {
        SortingActor {
            framed,
            addr
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
        debug!("[{}] SortingActor ~ START", self.addr)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        debug!("[{}] SortingActor ~ STOPPING", self.addr);

        System::current().stop();

        Running::Stop
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("[{}] SortingActor ~ STOP", self.addr)
    }

}

impl actix::io::WriteHandler<io::Error> for SortingActor {}

impl StreamHandler<messages::SortingRequest, io::Error> for SortingActor {
    fn handle(&mut self, msg: messages::SortingRequest, _: &mut Context<Self>) {
        info!("[SortingActor][{}] Got sorting request: Vec[{}]", self.addr, msg.values.len());

        let sorted_values = self.sort_vec(msg.values);

        info!("[SortingActor][{}] Done sorting", self.addr);

        self.framed.write(messages::SortingResponse::new(sorted_values, -1))
    }
}
