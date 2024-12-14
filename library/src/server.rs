use crate::error::Error;
use crate::game::instance::Instance;
use crate::input::crossterm_wrapper_next;

use crate::input::on_term_event;

use crate::network;

use crate::network::tls::ClientPki;

use crate::network::tls::ServerPki;
use crate::service;
use crate::Result;
use crossterm::event::EventStream;

use hyper::server::conn::http1::{self};
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use log::debug;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub enum InstanceConfig {
    UserInstance(Arc<Mutex<Instance>>),
    UserSqliteDb { path: String },
}

pub enum TcpConfig {
    Port(u16),
    TcpListener(TcpListener),
}

pub struct ServerConfig<'a> {
    pub tcp: TcpConfig,
    pub pki: Option<ServerPki<'a>>,
}

pub struct ClientConfig<'a> {
    pub addr: String,
    pub nickname: String,
    pub pki: ClientPki<'a>,
}

pub async fn run(
    instance_config: InstanceConfig,
    server_config: ServerConfig<'_>,
    user_input: bool,
    stop: crossbeam::channel::Receiver<()>,
) -> Result<()> {
    let instance = match instance_config {
        InstanceConfig::UserInstance(instance) => instance,
        InstanceConfig::UserSqliteDb { path } => {
            info!("Loading {}", path);
            Arc::new(Mutex::new(Instance::from_path(path.as_str()).await?))
        }
    };

    let listener = match server_config.tcp {
        TcpConfig::Port(port) => TcpListener::bind(format!("localhost:{}", port))
            .await
            .map_err(|err| Error::TcpCouldNotConnect(err))?,
        TcpConfig::TcpListener(listener) => listener,
    };

    let tls_acceptor = if let Some(pki) = server_config.pki {
        Some(network::tls::get_acceptor(pki)?)
    } else {
        None
    };

    let mut ref_instant = tokio::time::Instant::now();
    let mut maybe_input_stream = if user_input {
        Some(EventStream::new())
    } else {
        None
    };
    let mut prompt: String = String::new();
    let mut update_tick_delay = tokio::time::interval(std::time::Duration::from_millis(250));
    let mut save_tick_delay = tokio::time::interval(std::time::Duration::from_secs(10));

    info!(
        "Server loop starts, listenning on {}",
        listener.local_addr().unwrap().port()
    );

    loop {
        tokio::select! {
            // ----------------------------------------------------
            // ON UPDATE TICK DELAY---------------------------------------
            _ = update_tick_delay.tick() => {
                debug!("Update Tick");

                if stop.try_recv().is_ok() {
                    instance.lock().await.sync_to_db().await?;
                    info!("Server loop stops now (on stop channel)!");
                    return Ok(())
                }
                let now = tokio::time::Instant::now();
                let delta = now - ref_instant;
                ref_instant = now;
                instance.lock().await.galaxy.update(delta.as_secs_f64()).await;
            },
            // ----------------------------------------------------
            // ON SAVE TICK DELAY---------------------------------------
            _ = save_tick_delay.tick() => {
                debug!("Save Tick");
                instance.lock().await.sync_to_db().await?;
            },
            // ----------------------------------------------------
            // ON TERM EVENT---------------------------------------
            Some(Ok(_event)) = crossterm_wrapper_next(&mut maybe_input_stream) => {
                if on_term_event(_event, &mut prompt) {
                    instance.lock().await.sync_to_db().await?;
                    info!("Server loop stops now (on user input)!");
                    return Ok(())
                }
            },
            // ----------------------------------------------------
            // ON TCP ACCEPT---------------------------------------
            Ok((stream, addr)) = listener.accept() => {
                on_tcp_accept(stream, addr, tls_acceptor.clone(), Arc::clone(&instance));
            },
        }
    }

    fn on_tcp_accept(
        stream: TcpStream,
        addr: SocketAddr,
        tls_acceptor: Option<tokio_rustls::TlsAcceptor>,
        instance: Arc<Mutex<Instance>>,
    ) {
        info!("TCP accept from: {}", addr);

        if let Some(tls_acceptor) = tls_acceptor {
            let acceptor = tls_acceptor.clone();
            tokio::spawn(async move {
                let maybe_tls_stream = acceptor.accept(stream).await;
                match maybe_tls_stream {
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            panic!("Would block");
                        }
                        _ => {
                            info!("TLS error: {}", err);
                        }
                    },
                    Ok(tls_stream) => {
                        run_http(tls_stream, Arc::clone(&instance));
                    }
                }
            });
        } else {
            run_http(stream, Arc::clone(&instance));
        }
    }

    fn run_http<T>(stream: T, instance: Arc<Mutex<Instance>>)
    where
        T: tokio::io::AsyncRead
            + tokio::io::AsyncWrite
            + std::marker::Unpin
            + std::marker::Send
            + 'static,
    {
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            let instance = Arc::clone(&instance);

            if let Err(_err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: Request<hyper::body::Incoming>| {
                        let instance = Arc::clone(&instance);
                        service::serve_http(req, instance)
                    }),
                )
                .with_upgrades()
                .await
            {}
            ()
        });
    }
}
