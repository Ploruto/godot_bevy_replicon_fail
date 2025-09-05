use std::{
    net::{Ipv4Addr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
};
use shared::{PingEvent, PongEvent, PingPongCounter, SERVER_PORT};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, RepliconPlugins, RepliconRenetPlugins))
        .replicate::<PingPongCounter>()
        .add_client_event::<PingEvent>(Channel::Ordered)
        .add_server_event::<PongEvent>(Channel::Ordered)
        .add_observer(spawn_counter_on_client_connect)
        .add_systems(Startup, start_server)
        .add_systems(Update, handle_ping_events)
        .run();
}

fn start_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    info!("Starting replicon server on port {SERVER_PORT}");
    
    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, SERVER_PORT)).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: 0,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    commands.insert_resource(server);
    commands.insert_resource(transport);
}

fn spawn_counter_on_client_connect(
    trigger: Trigger<OnAdd, ConnectedClient>,
    mut commands: Commands,
) {
    info!("Client connected: {}", trigger.target());
    commands.spawn(PingPongCounter { count: 0 });
}

fn handle_ping_events(
    mut ping_events: EventReader<FromClient<PingEvent>>,
    mut pong_events: EventWriter<ToClients<PongEvent>>,
    mut counters: Query<&mut PingPongCounter>,
) {
    for FromClient { client_entity, event } in ping_events.read() {
        info!("Received ping from client {}: {}", client_entity, event.message);
        
        if let Ok(mut counter) = counters.single_mut() {
            counter.count += 1;
            
            let response = format!("Pong! Counter: {}", counter.count);
            pong_events.write(ToClients {
                mode: SendMode::Broadcast,
                event: PongEvent { response: response.clone() },
            });
            
            info!("Sent pong: {}", response);
        }
    }
}
