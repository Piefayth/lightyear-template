#![cfg(not(target_family = "wasm"))]
use crate::{
    launch_options::{ClientLaunchOptions, ServerLaunchOptions, SharedLaunchOptions},
    launch_options::{
        SerializableClientLaunchOptions, SerializableServerLaunchOptions,
        SerializableSharedLaunchOptions,
    },
};
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use lightyear::{
    client::config::{ClientConfig, NetcodeConfig as ClientNetcodeConfig},
    connection::client::NetConfig as ClientNetConfig,
    prelude::{
        LinkConditionerConfig, SharedConfig, TickConfig,
        client::{
            Authentication, ClientTransport, InterpolationConfig, IoConfig as ClientIoConfig,
            PredictionConfig,
        },
        server::{
            Identity, IoConfig as ServerIoConfig, NetConfig as ServerNetConfig, ServerTransport,
        },
    },
    server::config::{NetcodeConfig as ServerNetcodeConfig, ServerConfig},
};
use mygame_client::app::build_client_app;
use mygame_server::app::{ServerMode, build_server_app};
use ron::de::from_str;
use std::{
    error::Error,
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

const DEFAULT_CLIENT_CONFIG_PATH: &str = "./crates/mygame-launcher/options/client_options.ron";
const DEFAULT_SERVER_CONFIG_PATH: &str = "./crates/mygame-launcher/options/server_options.ron";
const DEFAULT_SHARED_CONFIG_PATH: &str = "./crates/mygame-launcher/options/shared_options.ron";

#[derive(Parser)]
#[command(name = "mygame")]
#[command(version = "0.1")]
#[command(about = "Manages and launches various server and client configurations for mygame.")]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,

    #[arg(long, default_value_t = false)]
    headless: bool,

    #[arg(short, long, default_value_t = 0)]
    client_id: u64,

    #[arg(long, value_name = "FILE")]
    shared_options: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    client_options: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    server_options: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Client,
    Server,
}

fn load_config<T, S>(path: Option<PathBuf>, default_path: &str) -> Option<T>
where
    T: From<S>,
    S: serde::de::DeserializeOwned,
{
    let config_path = path.unwrap_or_else(|| PathBuf::from(default_path));

    if !config_path.exists() {
        return None;
    }

    let config_str = match fs::read_to_string(&config_path) {
        Ok(str) => str,
        Err(_) => {
            println!("Warning: Failed to read config from {:?}", config_path);
            return None;
        }
    };

    let serializable_config: S = match from_str(&config_str) {
        Ok(config) => config,
        Err(e) => {
            println!(
                "Warning: Failed to parse config from {:?}: {}",
                config_path, e
            );
            return None;
        }
    };

    Some(T::from(serializable_config))
}

fn load_shared_options(path: Option<PathBuf>) -> SharedLaunchOptions {
    load_config::<SharedLaunchOptions, SerializableSharedLaunchOptions>(
        path,
        DEFAULT_SHARED_CONFIG_PATH,
    )
    .unwrap_or_default()
}

fn load_client_options(path: Option<PathBuf>) -> ClientLaunchOptions {
    load_config::<ClientLaunchOptions, SerializableClientLaunchOptions>(
        path,
        DEFAULT_CLIENT_CONFIG_PATH,
    )
    .unwrap_or_default()
}

fn load_server_options(path: Option<PathBuf>) -> ServerLaunchOptions {
    load_config::<ServerLaunchOptions, SerializableServerLaunchOptions>(
        path,
        DEFAULT_SERVER_CONFIG_PATH,
    )
    .unwrap_or_default()
}

pub fn run() {
    let cli = Cli::parse();

    let shared_launch_options = load_shared_options(cli.shared_options);

    let shared_config = SharedConfig {
        server_replication_send_interval: shared_launch_options.server_replication_send_interval,
        client_replication_send_interval: shared_launch_options.client_replication_send_interval,
        tick: TickConfig {
            tick_duration: shared_launch_options.simulation_update_frequency,
        },
    };

    match cli.mode {
        Mode::Client => {
            if cli.client_id == 0 {
                panic!(
                    "No --client_id specified. To connect with multiple clients, specify a unique client id with --client_id <id>"
                )
            }

            let client_launch_options = load_client_options(cli.client_options);

            let mut server_launch_options = load_server_options(cli.server_options);

            // Always set server to headless in client mode
            server_launch_options.headless = true;

            let (from_server_send, from_server_recv) = crossbeam_channel::unbounded();
            let (to_server_send, to_server_recv) = crossbeam_channel::unbounded();

            let local_transport_config =
                ClientIoConfig::from_transport(ClientTransport::LocalChannel {
                    recv: from_server_recv.clone(),
                    send: to_server_send.clone(),
                });

            let local_auth = Authentication::Manual {
                server_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
                client_id: cli.client_id,
                private_key: shared_launch_options.key,
                protocol_id: shared_launch_options.protocol_id,
            };

            let local_netcode = ClientNetConfig::Netcode {
                auth: local_auth,
                config: ClientNetcodeConfig {
                    token_expire_secs: -1,
                    client_timeout_secs: 5,
                    ..default()
                },
                io: local_transport_config,
            };

            let local_client_config = ClientConfig {
                shared: shared_config,
                net: local_netcode,
                prediction: PredictionConfig::default()
                    .with_correction_ticks_factor(client_launch_options.correction_ticks_factor),
                interpolation: InterpolationConfig {
                    min_delay: client_launch_options.min_delay,
                    send_interval_ratio: 0.,
                },
                ..default()
            };

            let remote_transport_config =
                ClientIoConfig::from_transport(ClientTransport::UdpSocket(SocketAddr::new(
                    IpAddr::V4(client_launch_options.listen_addr),
                    client_launch_options.listen_port,
                )));

            let remote_auth = Authentication::Manual {
                server_addr: SocketAddr::new(
                    IpAddr::V4(client_launch_options.server_addr),
                    client_launch_options.server_port,
                ),
                client_id: cli.client_id,
                private_key: shared_launch_options.key,
                protocol_id: shared_launch_options.protocol_id,
            };

            let remote_netcode = ClientNetConfig::Netcode {
                auth: remote_auth,
                config: ClientNetcodeConfig {
                    token_expire_secs: -1,
                    client_timeout_secs: 5,
                    ..default()
                },
                io: remote_transport_config,
            };

            let remote_client_config = ClientConfig {
                shared: shared_config,
                net: remote_netcode,
                prediction: PredictionConfig::default()
                    .with_correction_ticks_factor(client_launch_options.correction_ticks_factor),
                interpolation: InterpolationConfig {
                    min_delay: client_launch_options.min_delay,
                    send_interval_ratio: 0.,
                },
                ..default()
            };

            let server_netcode_config = ServerNetcodeConfig::default()
                .with_protocol_id(shared_launch_options.protocol_id)
                .with_key(shared_launch_options.key);

            let maybe_webtransport_identity = load_certificate_from_files(
                Path::new(&server_launch_options.webtransport_cert_path),
                Path::new(&server_launch_options.webtransport_key_path),
            );

            let mut net_configs = vec![
                ServerNetConfig::Netcode {
                    // normal udp sockets for desktop
                    config: server_netcode_config.clone(),
                    io: ServerIoConfig::from_transport(ServerTransport::UdpSocket(
                        (
                            server_launch_options.listen_addr,
                            server_launch_options.udp_listen_port,
                        )
                            .into(),
                    ))
                    .with_conditioner(server_launch_options.conditioner.clone()),
                },
                ServerNetConfig::Netcode {
                    // channels, for client host
                    config: server_netcode_config.clone(),
                    io: ServerIoConfig::from_transport(ServerTransport::Channels {
                        channels: vec![(
                            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 12027), // port doesn't matter?
                            to_server_recv,
                            from_server_send,
                        )],
                    })
                    .with_conditioner(server_launch_options.conditioner.clone()),
                },
            ];

            match maybe_webtransport_identity {
                Ok(webtransport_identity) => {
                    net_configs.push(ServerNetConfig::Netcode {
                        config: server_netcode_config.clone(),
                        io: ServerIoConfig::from_transport(ServerTransport::WebTransportServer {
                            server_addr: SocketAddr::new(
                                IpAddr::V4(server_launch_options.listen_addr),
                                server_launch_options.webtransport_listen_port,
                            ),
                            certificate: webtransport_identity,
                        })
                        .with_conditioner(server_launch_options.conditioner.clone()),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "WARNING - Skipping WebTransport setup; unable to load cert.pem and key.pem due to {}",
                        e
                    );
                }
            }

            let server_config = ServerConfig {
                shared: shared_config,
                net: net_configs,
                ..default()
            };

            build_client_app(
                remote_client_config,
                local_client_config,
                client_launch_options.asset_path,
                server_config,
            )
            .run();
        }
        Mode::Server => {
            let server_launch_options = load_server_options(cli.server_options);

            let headless = cli.headless || server_launch_options.headless;

            let server_netcode_config = ServerNetcodeConfig::default()
                .with_protocol_id(shared_launch_options.protocol_id)
                .with_key(shared_launch_options.key);

            let mut net_configs = vec![ServerNetConfig::Netcode {
                // normal udp sockets for desktop
                config: server_netcode_config.clone(),
                io: ServerIoConfig::from_transport(ServerTransport::UdpSocket(
                    (
                        server_launch_options.listen_addr,
                        server_launch_options.udp_listen_port,
                    )
                        .into(),
                ))
                .with_conditioner(server_launch_options.conditioner.clone()),
            }];

            match load_certificate_from_files(
                Path::new(&server_launch_options.webtransport_cert_path),
                Path::new(&server_launch_options.webtransport_key_path),
            ) {
                Ok(webtransport_identity) => {
                    println!(
                        "Launching Server with Certificate Digest: {}",
                        webtransport_identity.certificate_chain().as_slice()[0]
                            .hash()
                            .to_string()
                            .replace(":", "")
                    );

                    net_configs.push(ServerNetConfig::Netcode {
                        // webtransport
                        config: server_netcode_config.clone(),
                        io: ServerIoConfig::from_transport(ServerTransport::WebTransportServer {
                            server_addr: SocketAddr::new(
                                IpAddr::V4(server_launch_options.listen_addr),
                                server_launch_options.webtransport_listen_port,
                            ),
                            certificate: webtransport_identity,
                        })
                        .with_conditioner(server_launch_options.conditioner.clone()),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "WARNING - Skipping WebTransport setup; unable to load cert.pem and key.pem due to {}",
                        e
                    );
                }
            }

            let server_config = ServerConfig {
                shared: shared_config,
                net: net_configs,
                ..default()
            };

            let mode = if headless {
                ServerMode::Headless
            } else {
                ServerMode::Windowed
            };

            build_server_app(server_config, server_launch_options.asset_path, mode).run();
        }
    }
}

pub fn load_certificate_from_files(
    cert_path: &Path,
    key_path: &Path,
) -> Result<Identity, Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;

    let identity = rt.block_on(async { Identity::load_pemfiles(cert_path, key_path).await })?;

    Ok(identity)
}
