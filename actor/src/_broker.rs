use core::any::TypeId;
use std::sync::Arc;

use dashmap::DashSet;
use tokio::sync::watch::{Receiver, Sender};

pub struct Broker {
    receivers: Arc<DashMap<Receiver<Box<dyn Msg>>>>,
}

// TODO rename
trait Msg {}

type MsgId = TypeId;

trait CtrlMsg {
    fn handle(&self, broker: &Broker);
}

struct Publish(Receiver<Box<dyn Msg>>);

struct Subscribe(MsgId);

trait Publisher<T: Msg> {
    fn broker(&self) -> &Broker;
    fn publish(&self, msg: T) {
        self.sender().send(msg);
    }
}

trait Subscriber<T: Msg> {
    fn broker(&self) -> &Broker;
    fn subscribe(&self, msg: T);
    fn handle(&self, msg: T);
}

impl Broker {
    /// Construct [`Broker`].
    pub fn new() -> Self {
        Self(Arc::new(DashSet::new()))
    }
}

impl CtrlMsg for Publish {
    fn handle(&self, broker: &Broker) {
        broker.receivers.insert(self.0.clone());
    }
}
impl CtrlMsg for Subscribe {}
