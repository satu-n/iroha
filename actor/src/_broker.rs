use dashmap::DashSet;
use std::sync::Arc;
use tokio::sync::watch::Receiver;

pub struct Broker {
    receivers: Arc<DashSet<Receiver<Box<dyn Msg>>>>,
}

// TODO rename
trait Msg {}



