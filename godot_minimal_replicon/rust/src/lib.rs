use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use godot::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::server::ServerPlugin;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{ConnectionConfig, RenetClient},
};
use godot_bevy::prelude::*;
use shared::{PingEvent, PongEvent, PingPongCounter, SERVER_PORT};

#[bevy_app]
fn build_app(app: &mut App) {
	// Add plugins first
	app.add_plugins(GodotDefaultPlugins);
	
	// Add replicon plugins with explicit client configuration
	app.add_plugins(RepliconPlugins.build().disable::<ServerPlugin>());
	app.add_plugins(RepliconRenetPlugins);

	// Register replication and events in same order as server
	app.replicate::<PingPongCounter>();
	app.add_client_event::<PingEvent>(Channel::Ordered);
	app.add_server_event::<PongEvent>(Channel::Ordered);

	// Add systems after registration  
	app.add_systems(PostStartup, connect_to_server);
	app.add_systems(Update, (ping_system, handle_pong_events));
}

fn connect_to_server(mut commands: Commands, channels: Res<RepliconChannels>) {
	godot_print!("Connecting to replicon server...");
	godot_print!("Available channels: {} server, {} client", 
		channels.server_configs().len(), 
		channels.client_configs().len());
	
	let server_channels_config = channels.server_configs();
	let client_channels_config = channels.client_configs();

	let client = RenetClient::new(ConnectionConfig {
		server_channels_config,
		client_channels_config,
		..Default::default()
	});

	let current_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap();
	let client_id = current_time.as_millis() as u64;
	let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), SERVER_PORT);
	let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
	let authentication = ClientAuthentication::Unsecure {
		client_id,
		protocol_id: 0,
		server_addr,
		user_data: None,
	};
	let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

	commands.insert_resource(client);
	commands.insert_resource(transport);
	
	godot_print!("Client resources inserted, attempting connection to {}:{}", 
		server_addr.ip(), server_addr.port());
}

fn ping_system(
	mut timer: Local<f32>, 
	time: Res<Time>,
	mut ping_events: EventWriter<PingEvent>,
	client: Option<Res<RenetClient>>
) {
	// Only send pings if client exists and is connected
	if let Some(client) = client {
		if client.is_connected() {
			*timer += time.delta_secs();
			if *timer > 2.0 {
				*timer = 0.0;
				let ping_msg = format!("Ping from Godot client at {:?}", SystemTime::now());
				ping_events.write(PingEvent { message: ping_msg.clone() });
				godot_print!("Sent: {}", ping_msg);
			}
		}
	}
}

fn handle_pong_events(
	mut pong_events: EventReader<PongEvent>,
	counters: Query<&PingPongCounter>
) {
	for event in pong_events.read() {
		godot_print!("Received: {}", event.response);
		
		if let Ok(counter) = counters.single() {
			godot_print!("Current counter value: {}", counter.count);
		}
	}
}
