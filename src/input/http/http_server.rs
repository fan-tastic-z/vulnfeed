use std::{io, net::SocketAddr, time::Duration};

use mea::{shutdown::ShutdownRecv, waitgroup::WaitGroup};
use poem::{
    Endpoint, EndpointExt, Route, get,
    listener::{Acceptor, Listener, TcpAcceptor, TcpListener},
    post,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Assets;

use crate::input::http::{handlers::ding_bot_config, spa_endpoint::SpaFileEndpoint};

use crate::{
    cli::Ctx,
    domain::ports::VulnService,
    input::http::{
        handlers::{login, plugin, sec_notice, sync_data_task, vuln_information},
        middleware::auth::AuthMiddleware,
    },
    utils::runtime::{self, Runtime},
};

pub(crate) type ServerFuture<T> = runtime::JoinHandle<Result<T, io::Error>>;

#[derive(Debug)]
pub struct ServerState {
    advertise_addr: SocketAddr,
    server_fut: ServerFuture<()>,
    shutdown_rx_server: ShutdownRecv,
}

impl ServerState {
    pub fn advertise_addr(&self) -> SocketAddr {
        self.advertise_addr
    }
    pub async fn await_shutdown(self) {
        self.shutdown_rx_server.is_shutdown().await;
        log::info!("http server is shutting down");

        match self.server_fut.await {
            Ok(_) => log::info!("http server stopped"),
            Err(err) => log::error!(err:?;"http server failed."),
        }
    }
}

pub async fn make_acceptor_and_advertise_addr(
    listen_addr: &str,
    advertise_addr: Option<&str>,
) -> Result<(TcpAcceptor, SocketAddr), io::Error> {
    log::info!("listening on {}", listen_addr);

    let acceptor = TcpListener::bind(&listen_addr).into_acceptor().await?;
    let listen_addr = acceptor.local_addr()[0]
        .as_socket_addr()
        .cloned()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "failed to get local listen addr",
            )
        })?;

    let advertise_addr = match advertise_addr {
        None => {
            if listen_addr.ip().is_unspecified() {
                let ip = local_ip_address::local_ip().map_err(io::Error::other)?;
                let port = listen_addr.port();
                SocketAddr::new(ip, port)
            } else {
                listen_addr
            }
        }
        Some(advertise_addr) => advertise_addr
            .parse::<SocketAddr>()
            .map_err(io::Error::other)?,
    };

    Ok((acceptor, advertise_addr))
}

pub async fn start_server<S: VulnService + Send + Sync + 'static>(
    ctx: Ctx<S>,
    rt: &Runtime,
    shutdown_rx: ShutdownRecv,
    acceptor: TcpAcceptor,
    advertise_addr: SocketAddr,
) -> Result<ServerState, io::Error> {
    let wg = WaitGroup::new();
    let shutdown_rx_server = shutdown_rx;
    let server_fut = {
        let wg_clone = wg.clone();
        let shutdown_clone = shutdown_rx_server.clone();
        let route = Route::new()
            .nest("/", SpaFileEndpoint::<Assets>::new())
            .nest("/api", api_routes::<S>())
            .data(ctx.clone());
        let listen_addr = acceptor.local_addr()[0].clone();
        let signal = async move {
            log::info!("server has started on [{listen_addr}]");
            drop(wg_clone);

            shutdown_clone.is_shutdown().await;
            log::info!("server is closing");
        };
        rt.spawn(async move {
            poem::Server::new_with_acceptor(acceptor)
                .run_with_graceful_shutdown(route, signal, Some(Duration::from_secs(10)))
                .await
        })
    };
    wg.await;
    Ok(ServerState {
        advertise_addr,
        server_fut,
        shutdown_rx_server,
    })
}

fn api_routes<S: VulnService + Send + Sync + 'static>() -> impl Endpoint {
    Route::new()
        .nest("/login", post(login::login::<S>::default()))
        .nest(
            "/",
            Route::new()
                .nest(
                    "/vulns",
                    Route::new()
                        .at(
                            "",
                            get(vuln_information::list_vuln_information::<S>::default()),
                        )
                        .at(
                            "/:id",
                            get(vuln_information::get_vuln_information_detail::<S>::default()),
                        ),
                )
                .nest("/plugins", Route::new().at("", get(plugin::list_plugins)))
                .nest("/notices", Route::new().at("", get(plugin::list_notice)))
                .nest(
                    "/sync_data_task",
                    Route::new().at(
                        "",
                        post(sync_data_task::create_or_update_sync_data_task::<S>::default())
                            .get(sync_data_task::get_sync_data_task::<S>::default()),
                    ),
                )
                .nest(
                    "/ding_bot_config",
                    Route::new().at(
                        "",
                        post(ding_bot_config::create_or_update_ding_bot_config::<S>::default())
                            .get(ding_bot_config::get_ding_bot_config::<S>::default()),
                    ),
                )
                .nest(
                    "/sec_notice",
                    Route::new().at("", get(sec_notice::list_sec_notice::<S>::default())),
                )
                .with(AuthMiddleware::<S>::default()),
        )
}
