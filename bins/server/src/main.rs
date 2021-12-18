#![allow(clippy::type_complexity)]

mod ecs;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    network::{NetworkBundle, ServerConfig},
    prelude::{Application, GameDataBuilder},
    LogLevelFilter, Logger,
};
use laminar::Config as LaminarConfig;

use std::time::Duration;

use gv_core::{
    ecs::resources::world::{
        DummyFramedUpdate, FramedUpdates, ReceivedClientActionUpdates, ServerWorldUpdates,
    },
    net::EncodedMessage,
};
use gv_game::{
    build_game_logic_systems, ecs::systems::NetConnectionManagerSystem, states::LoadingState,
};

use crate::ecs::{resources::LastBroadcastedFrame, systems::*};

fn main() -> amethyst::Result<()> {
    let cli_matches = clap::App::new("grumpy_visitors")
        .version("0.1")
        .author("Vladyslav Batyrenko <mvlabat@gmail.com>")
        .about("A prototype of a top-down EvilInvasion-like 2D arcade/action")
        .arg(
            clap::Arg::with_name("addr")
                .short("a")
                .long("addr")
                .value_name("ADDR")
                .help("Specifies the address for UdpSocket")
                .default_value("127.0.0.1:3455")
                .takes_value(true),
        )
        .get_matches();

    let socket_addr = cli_matches
        .value_of("addr")
        .expect("Expected a default value");

    Logger::from_config_formatter(Default::default(), |out, message, record| {
        out.finish(format_args!(
            "[{level}][SERVER][{target}] {message}",
            level = record.level(),
            target = record.target(),
            message = message,
        ))
    })
    .level_for("gfx_backend_vulkan", LogLevelFilter::Warn)
    .level_for("gv_game::ecs::systems", LogLevelFilter::Debug)
    .level_for(
        "gv_game::ecs::systems::net_connection_manager",
        LogLevelFilter::Info,
    )
    .level_for("gv_game::utils::net", LogLevelFilter::Info)
    .level_for("gv_server", LogLevelFilter::Debug)
    .start();

    let mut builder = Application::build("./", LoadingState::default())?;
    builder
        .world
        .insert(FramedUpdates::<DummyFramedUpdate>::default());
    builder
        .world
        .insert(FramedUpdates::<ReceivedClientActionUpdates>::default());
    builder.world.insert(ServerWorldUpdates::default());
    builder.world.insert(LastBroadcastedFrame(0));

    let server_config = ServerConfig {
        udp_socket_addr: socket_addr.parse()?,
        laminar_config: LaminarConfig {
            receive_buffer_max_size: 14_500,
            ..LaminarConfig::default()
        },
        ..ServerConfig::default()
    };
    let mut game_data_builder = GameDataBuilder::default()
        .with_bundle(NetworkBundle::<EncodedMessage>::from_config(server_config))?
        .with(
            NetConnectionManagerSystem::default(),
            "net_connection_manager_system",
            &["net_socket"],
        )
        .with(
            ServerNetworkSystem::new(),
            "game_network_system",
            &["net_socket"],
        );
    game_data_builder = build_game_logic_systems(game_data_builder, &mut builder.world, true)?
        .with(
            GameUpdatesBroadcastingSystem::default(),
            "game_updates_broadcasting_system",
            &["action_system"],
        )
        .with_bundle(TransformBundle::new().with_dep(&["world_position_transform_system"]))?;

    let mut game = builder
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60,
        )
        .build(game_data_builder)?;
    game.run();

    Ok(())
}