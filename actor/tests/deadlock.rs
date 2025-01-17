#![allow(clippy::expect_used, clippy::exit, clippy::unwrap_used)]

use iroha_actor::prelude::*;

#[derive(Default, Debug)]
struct DeadlockActor(Option<Addr<Self>>);
struct Msg;
struct Address<A: Actor>(Addr<A>);

impl Message for Msg {
    type Result = ();
}
impl<A: Actor> Message for Address<A> {
    type Result = ();
}
impl Actor for DeadlockActor {}

#[async_trait::async_trait]
impl Handler<Msg> for DeadlockActor {
    type Result = ();
    async fn handle(&mut self, _: Msg) {
        if let Some(addr) = &self.0 {
            let _ = addr.send(Msg).await;
        }
    }
}

#[async_trait::async_trait]
impl Handler<Address<Self>> for DeadlockActor {
    type Result = ();
    async fn handle(&mut self, Address(addr): Address<Self>) {
        self.0 = Some(addr);
    }
}

/// Basic deadlock test.
#[cfg(feature = "deadlock_detection")]
#[tokio::test(flavor = "multi_thread")]
async fn async_test() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // This test should panic.
        std::process::exit(0);
    }));
    let addr1 = DeadlockActor::start_default().await;
    let addr2 = DeadlockActor::start_default().await;
    addr1.send(Address(addr2.clone())).await.unwrap();
    addr2.send(Address(addr1.clone())).await.unwrap();
    addr1.send(Msg).await.unwrap();
    unreachable!()
}
