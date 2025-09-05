use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Event, Serialize, Deserialize, Debug, Clone)]
pub struct PingEvent {
    pub message: String,
}

#[derive(Event, Serialize, Deserialize, Debug, Clone)]  
pub struct PongEvent {
    pub response: String,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
#[require(Replicated)]
pub struct PingPongCounter {
    pub count: u32,
}

pub const SERVER_PORT: u16 = 5000;