//! Events for streaming API.
#![allow(clippy::unused_self)]

use iroha_macro::FromVariant;
use iroha_schema::prelude::*;
use iroha_version::prelude::*;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod data;
mod pipeline;

declare_versioned_with_scale!(VersionedEventSocketMessage 1..2, Debug, Clone, FromVariant, IntoSchema);

impl VersionedEventSocketMessage {
    /// Converts from `&VersionedEventSocketMessage` to V1 reference
    pub const fn as_v1(&self) -> &EventSocketMessage {
        match self {
            Self::V1(v1) => v1,
        }
    }

    /// Converts from `&mut VersionedEventSocketMessage` to V1 mutable reference
    pub fn as_mut_v1(&mut self) -> &mut EventSocketMessage {
        match self {
            Self::V1(v1) => v1,
        }
    }

    /// Performs the conversion from `VersionedEventSocketMessage` to V1
    pub fn into_v1(self) -> EventSocketMessage {
        match self {
            Self::V1(v1) => v1,
        }
    }
}

/// Message type used for communication over web socket event stream.
#[allow(variant_size_differences)]
#[version_with_scale(n = 1, versioned = "VersionedEventSocketMessage")]
#[derive(Debug, Clone, IntoSchema, FromVariant, Decode, Encode, Deserialize, Serialize)]
pub enum EventSocketMessage {
    /// Request sent by client to subscribe to events.
    SubscriptionRequest(SubscriptionRequest),
    /// Answer sent by peer.
    /// The message means that all event connection is initialized and will be supplying
    /// events starting from the next one.
    SubscriptionAccepted,
    /// Event, sent by peer.
    Event(Event),
    /// Acknowledgment of receiving event sent from client.
    EventReceived,
}

//TODO: Sign request?
/// Subscription Request to listen to events
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, IntoSchema)]
pub struct SubscriptionRequest(pub EventFilter);

/// Event.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, FromVariant, IntoSchema)]
pub enum Event {
    /// Pipeline event.
    Pipeline(pipeline::Event),
    /// Data event.
    Data(data::Event),
}

/// Event filter.
#[derive(Debug, Decode, Encode, Deserialize, Serialize, Clone, FromVariant, IntoSchema)]
pub enum EventFilter {
    /// Listen to pipeline events with filter.
    Pipeline(pipeline::EventFilter),
    /// Listen to data events with filter.
    Data(data::EventFilter),
}

impl EventFilter {
    /// Apply filter to `event`: check if `event` is accepted.
    pub fn apply(&self, event: &Event) -> bool {
        match (event, self) {
            (Event::Pipeline(event), EventFilter::Pipeline(filter)) => filter.apply(event),
            (Event::Data(event), EventFilter::Data(filter)) => filter.apply(event),
            _ => false,
        }
    }
}

/// Exports common structs and enums from this module.
pub mod prelude {
    pub use super::{
        data::prelude::*, pipeline::prelude::*, Event, EventFilter, EventSocketMessage,
        SubscriptionRequest, VersionedEventSocketMessage,
    };
}
