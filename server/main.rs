use std::{env, time::Duration};

use clap::Parser;

use spacebuild::{
    network::tls::ServerPki,
    server::{self, InstanceConfig, ServerConfig},
};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use anyhow::{bail, Result};

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    #[arg(value_name = "PORT", default_value_t = 2567)]
    port: u16,

    #[arg(short, long,
        num_args = 2,
        value_names = ["CERT_PATH", "KEY_PATH"],
    )]
    tls: Option<Vec<String>>,

    #[arg(short, long, default_value = "galaxy.sbdb")]
    instance: String,

    #[arg(short, long, default_value_t = false)]
    no_input: bool,

    #[arg(short, long)]
    stop_after: Option<u64>,

    #[arg(long, default_value = "spacebuild::(.*)", value_name = "REGEX")]
    trace_filter: String,

    #[arg(
        long,
        default_value = "INFO",
        value_name = "TRACE|DEBUG|INFO|WARN|ERROR"
    )]
    trace_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env::set_var("RUST_LOG", args.trace_level);
    let pki = if let Some(tls) = args.tls {
        Some(ServerPki::Paths {
            cert: tls.first().unwrap().clone(),
            key: tls.last().unwrap().clone(),
        })
    } else {
        None
    };

    common::trace::init(Some(args.trace_filter));

    let (stop_send, stop_recv) = crossbeam::channel::bounded(1);
    let server_hdl: JoinHandle<Result<()>> = tokio::spawn(async move {
        if let spacebuild::Result::Err(err) = server::run(
            InstanceConfig::UserSqliteDb {
                path: args.instance,
            },
            ServerConfig {
                tcp: server::TcpConfig::Port(args.port),
                pki,
            },
            !args.no_input,
            stop_recv,
        )
        .await
        {
            bail!(format!("Server error: {}", err))
        } else {
            Ok(())
        }
    });

    let waiter_hdl = tokio::spawn(async move {
        if let Some(stop_after) = args.stop_after {
            sleep(Duration::from_secs(stop_after)).await;
            let _ = stop_send.send(());
        }
        anyhow::Ok(())
    });

    tokio::select! {
        result = server_hdl => {
            result??;
        },
        result = waiter_hdl, if args.stop_after.is_some() => {
            result??;
        }
    }

    Ok(())
}
