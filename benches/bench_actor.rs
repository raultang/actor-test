use std::collections::HashMap;
use actix::{Message, Actor, Handler, ResponseActFuture, Context, fut, System, Addr, Arbiter, ActorFuture, AsyncContext, SystemRunner};
use std::time::{Instant, Duration};
use metrics::timing;
use log::debug;
use futures01::Future;
use actor_test::{init_log, into_metrics};

#[tokio::main]
async fn main() {

    init_log();
    into_metrics();

    let mut system = System::builder().stop_on_panic(true).name("test").build();

    //one_actor_in_main_thread(&mut system);
    //one_sender_one_actor();
    //one_sender_two_actor();
    //two_sender_two_actor();
    three_actor_in_main_thread(&mut system);

    //two_sender_two_actor_main_thread();

    system.run();
}

/// Result:
/// ```
/// actor qps: 208262
/// actor max: 28671999
/// actor min: 3538
/// actor p50: 3807
/// actor p90: 6215
/// actor p95: 6451
/// actor p99: 10735
/// actor p999: 26447
/// ```
fn one_actor_in_main_thread(system: &mut SystemRunner) {
    let request = Request {
        m1: "11111".to_string(),
        m2: "22222".to_string()
    };
    let actor2_addr = Actor2 { map: Default::default() }.start();
    loop {
        let start = Instant::now();
        system.block_on(
            actor2_addr.send(request.clone())
                .map_err(|e| {})
                .and_then(|res| {
                    futures01::future::ok(())
                })
        );
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// actor qps: 313572
/// actor max: 115711
/// actor min: 2978
/// actor p50: 3169
/// actor p90: 3485
/// actor p95: 3527
/// actor p99: 5767
/// actor p999: 16351
/// ```
fn one_actor_in_main_thread_with_empty_payload(system: &mut SystemRunner) {
    let actor2_addr = Actor2 { map: Default::default() }.start();
    loop {
        let start = Instant::now();
        system.block_on(actor2_addr.send(EmptyRequest));
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// actor qps: 109138
/// actor max: 103807
/// actor min: 8344
/// actor p50: 8799
/// actor p90: 9703
/// actor p95: 9847
/// actor p99: 19327
/// actor p999: 33055
/// ```
fn two_actor_in_main_thread(system: &mut SystemRunner) {
    let request = Request {
        m1: "11111".to_string(),
        m2: "22222".to_string()
    };
    let actor2_addr = Actor2 { map: Default::default() }.start();
    let actor1_addr = Actor1 { map: Default::default(), addr: actor2_addr.clone() }.start();
    loop {
        let start = Instant::now();
        system.block_on(actor1_addr.send(request.clone()));
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// actor qps: 74297
/// actor max: 184063
/// actor min: 13208
/// actor p50: 13591
/// actor p90: 15335
/// actor p95: 15599
/// actor p99: 24735
/// actor p999: 39519
/// ```
fn three_actor_in_main_thread(system: &mut SystemRunner) {
    let request = Request {
        m1: "11111".to_string(),
        m2: "22222".to_string()
    };
    let actor2_addr = Actor2 { map: Default::default() }.start();
    let actor1_addr = Actor1 { map: Default::default(), addr: actor2_addr.clone() }.start();
    let actor3_addr = Actor3 { map: Default::default(), addr: actor1_addr.clone() }.start();

    let actor3_addr_cloned = actor3_addr.clone();
    let sender_addr = Sender3::start_in_arbiter(&Arbiter::new(), |_| Sender3 { addr: actor3_addr_cloned });
    let actor3_addr_cloned = actor3_addr.clone();
    let sender_addr = Sender3::start_in_arbiter(&Arbiter::new(), |_| Sender3 { addr: actor3_addr_cloned });
    let actor3_addr_cloned = actor3_addr.clone();
    let sender_addr = Sender3::start_in_arbiter(&Arbiter::new(), |_| Sender3 { addr: actor3_addr_cloned });

    loop {
        let start = Instant::now();
        system.block_on(actor3_addr.send(request.clone()));
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// actor qps: 47665
/// actor max: 1511423
/// actor min: 3388
/// actor p50: 21599
/// actor p90: 30175
/// actor p95: 38495
/// actor p99: 56703
/// actor p999: 75455
/// ```
fn one_actor_in_different_thread(system: &mut SystemRunner) {
    let request = Request {
        m1: "11111".to_string(),
        m2: "22222".to_string()
    };
    let actor2_addr = Actor2::start_in_arbiter(&Arbiter::new(), |_| Actor2 { map: Default::default() });
    loop {
        let start = Instant::now();
        system.block_on(actor2_addr.send(request.clone()));
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// actor qps: 24794
/// actor max: 1400831
/// actor min: 29376
/// actor p50: 40415
/// actor p90: 58495
/// actor p95: 75199
/// actor p99: 89023
/// actor p999: 120447
/// ```
fn two_actor_in_different_thread(system: &mut SystemRunner) {
    let request = Request {
        m1: "11111".to_string(),
        m2: "22222".to_string()
    };
    let actor2_addr = Actor2::start_in_arbiter(&Arbiter::new(), |_| Actor2 { map: Default::default() });
    let actor2_addr_cloned = actor2_addr.clone();
    let actor1_addr = Actor1::start_in_arbiter(&Arbiter::new(), |_| Actor1 { map: Default::default(), addr: actor2_addr_cloned });
    loop {
        let start = Instant::now();
        system.block_on(actor1_addr.send(request.clone()));
        timing!("actor", Instant::now() - start);
    }
}

/// Result:
/// ```
/// sender qps: 1073
/// sender max: 521727
/// sender min: 23376
/// sender p50: 93055
/// sender p90: 112511
/// sender p95: 116095
/// sender p99: 148607
/// sender p999: 268543
/// ```
fn one_sender_one_actor() {
    let actor2_addr = Actor2::start_in_arbiter(&Arbiter::new(), |_| Actor2 { map: Default::default() });

    let sender_addr = Sender2::start_in_arbiter(&Arbiter::new(), |_| Sender2 { addr: actor2_addr });
}

/// Result:
/// ```
/// sender qps: 1083
/// sender max: 468223
/// sender min: 46432
/// sender p50: 139647
/// sender p90: 159231
/// sender p95: 177791
/// sender p99: 227711
/// sender p999: 345855
/// ```
fn one_sender_two_actor() {
    let actor2_addr = Actor2::start_in_arbiter(&Arbiter::new(), |_| Actor2 { map: Default::default() });
    let actor2_addr_cloned = actor2_addr.clone();
    let actor1_addr = Actor1::start_in_arbiter(&Arbiter::new(), |_| Actor1 { map: Default::default(), addr: actor2_addr_cloned });

    let sender_addr = Sender1::start_in_arbiter(&Arbiter::new(), |_| Sender1 { addr: actor1_addr });
}

/// Result:
/// ```
/// sender qps: 2138
/// sender max: 622591
/// sender min: 30784
/// sender p50: 128383
/// sender p90: 167807
/// sender p95: 176767
/// sender p99: 222975
/// sender p999: 338943
/// ```
fn two_sender_two_actor() {
    let actor2_addr = Actor2::start_in_arbiter(&Arbiter::new(), |_| Actor2 { map: Default::default() });
    let actor2_addr_cloned = actor2_addr.clone();
    let actor1_addr = Actor1::start_in_arbiter(&Arbiter::new(), |_| Actor1 { map: Default::default(), addr: actor2_addr_cloned });
    let actor1_addr_cloned = actor1_addr.clone();
    let sender_addr = Sender1::start_in_arbiter(&Arbiter::new(), |_| Sender1 { addr: actor1_addr_cloned });
    let actor1_addr_cloned = actor1_addr.clone();
    let sender_addr = Sender1::start_in_arbiter(&Arbiter::new(), |_| Sender1 { addr: actor1_addr_cloned });

}

fn two_sender_two_actor_main_thread() {

    let actor2_addr = Actor2 { map: Default::default() }.start();
    let actor2_addr_cloned = actor2_addr.clone();
    let actor1_addr = Actor1 { map: Default::default(), addr: actor2_addr_cloned }.start();
    let actor1_addr_cloned = actor1_addr.clone();
    let sender_addr = Sender1 { addr: actor1_addr_cloned }.start();
    let actor1_addr_cloned = actor1_addr.clone();
    let sender_addr = Sender1 { addr: actor1_addr_cloned }.start();

}

pub struct Sender1 {
    addr: Addr<Actor1>,
}

pub struct Sender2 {
    addr: Addr<Actor2>,
}

pub struct Sender3 {
    addr: Addr<Actor3>,
}

pub struct Actor1 {
    map: HashMap<String, String>,
    addr: Addr<Actor2>,
}

pub struct Actor2 {
    map: HashMap<String, String>,
}

pub struct Actor3 {
    map: HashMap<String, String>,
    addr: Addr<Actor1>,
}

#[derive(Debug, Clone)]
pub struct Request {
    m1: String,
    m2: String,
}

pub struct Response;

impl Message for Request {
    type Result = Result<Response, ()>;
}

#[derive(Debug, Clone)]
pub struct EmptyRequest;

impl Message for EmptyRequest {
    type Result = Result<Response, ()>;
}

impl Actor for Actor1 {
    type Context = Context<Self>;

    /// Start this actor.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
    }
}

impl Handler<Request> for Actor1 {

    type Result = ResponseActFuture<Self, Response, ()>;

    fn handle(&mut self, req: Request, _: &mut Self::Context) -> Self::Result {
        let request = req.clone();
        let fut = fut::wrap_future(self.addr.send(req))
            .map_err(|e, _, _| println!("{:?}", e))
            .and_then(|res, act: &mut Self, _ | {
                act.map.insert(request.m1, request.m2);
                fut::ok(Response)
        });

        Box::new(fut)
    }
}

impl Actor for Actor2 {
    type Context = Context<Self>;

    /// Start this actor.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
    }
}

impl Handler<Request> for Actor2 {

    type Result = ResponseActFuture<Self, Response, ()>;

    fn handle(&mut self, req: Request, _: &mut Self::Context) -> Self::Result {
        self.map.insert(req.m1, req.m2);

        Box::new(fut::ok(Response))
    }

}

impl Handler<EmptyRequest> for Actor2 {

    type Result = ResponseActFuture<Self, Response, ()>;

    fn handle(&mut self, req: EmptyRequest, _: &mut Self::Context) -> Self::Result {

        Box::new(fut::ok(Response))

    }
}

impl Actor for Actor3 {
    type Context = Context<Self>;

    /// Start this actor.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
    }
}

impl Handler<Request> for Actor3 {

    type Result = ResponseActFuture<Self, Response, ()>;

    fn handle(&mut self, req: Request, _: &mut Self::Context) -> Self::Result {

        let request = req.clone();
        let fut = fut::wrap_future(self.addr.send(req))
            .map_err(|e, _, _| println!("{:?}", e))
            .and_then(|res, act: &mut Self, _ | {
                act.map.insert(request.m1, request.m2);
                fut::ok(Response)
            });

        Box::new(fut)

    }

}

impl Actor for Sender1 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let request = Request {
            m1: "11111".to_string(),
            m2: "22222".to_string()
        };

        ctx.run_interval(Duration::from_millis(1), move|act, ctx| {
            let start = Instant::now();
            let fut = act.addr.send(request.clone())
                .map_err(|e| println!("{:?}", e))
                .and_then(move |re| {
                    timing!("sender", Instant::now() - start);
                    futures01::future::ok(())
                });
            ctx.spawn(fut::wrap_future(fut));
            //timing!("sender", Instant::now() - start);
        });

    }
}

impl Actor for Sender2 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let request = Request {
            m1: "11111".to_string(),
            m2: "22222".to_string()
        };

        ctx.run_interval(Duration::from_millis(1), move|act, ctx| {
            let start = Instant::now();
            let fut = act.addr.send(request.clone())
                .map_err(|e| println!("{:?}", e))
                .and_then(move |re| {
                    timing!("sender", Instant::now() - start);
                    futures01::future::ok(())
                });
            ctx.spawn(fut::wrap_future(fut));
            //timing!("sender", Instant::now() - start);
        });

    }
}

impl Actor for Sender3 {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let request = Request {
            m1: "11111".to_string(),
            m2: "22222".to_string()
        };

        ctx.run_interval(Duration::from_millis(1), move|act, ctx| {
            let start = Instant::now();
            let fut = act.addr.send(request.clone())
                .map_err(|e| println!("{:?}", e))
                .and_then(move |re| {
                    timing!("sender", Instant::now() - start);
                    futures01::future::ok(())
                });
            ctx.spawn(fut::wrap_future(fut));
//            timing!("actor", Instant::now() - start);
        });

    }
}