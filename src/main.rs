use std::{env, sync::Arc, time::Duration};

use actix_web::{web, App, HttpServer};
use async_ctrlc::CtrlC;
use config::{Config, SniMap};
use handler::{reverse_proxy, ClientPair};
use tlscert::{cert_generate, rustls_client_config, rustls_server_config, DisableSni};
use utils::edit_hosts;

mod config;
mod dirs;
mod handler;
mod resolver;
mod tlscert;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let sni_map = SniMap::from(Config::from_file().await?);

    let hostnames = sni_map.hostnames();

    edit_hosts(&hostnames).await?;

    let cert = cert_generate(&hostnames).await?;

    let sni_map_data = web::Data::new(sni_map);

    let (client_config_enable_sni, client_config_disable_sni) = (
        Arc::new(rustls_client_config()),
        Arc::new(rustls_client_config().disable_sni()),
    );

    let server = HttpServer::new(move || {
        App::new()
            .app_data(sni_map_data.clone())
            .app_data(web::Data::new(ClientPair::new(
                client_config_enable_sni.clone(),
                client_config_disable_sni.clone(),
            )))
            .default_service(web::to(reverse_proxy))
    })
    .bind_rustls("127.0.0.1:443", rustls_server_config(cert)?)?
    .disable_signals()
    .client_request_timeout(Duration::from_secs(30))
    .client_disconnect_timeout(Duration::from_secs(30))
    .run();

    let server_handle = server.handle();

    futures::try_join!(
        async {
            CtrlC::new()
                .expect("Failed to install Ctrl-C handler")
                .await;
            server_handle.stop(true).await;
            edit_hosts(&Vec::new()).await?;
            log::info!(target: "proxy", "restore hosts");
            Ok::<(), Box<dyn std::error::Error>>(())
        },
        async {
            log::info!(target: "proxy", "start server on :443");
            server.await?;
            Ok::<(), Box<dyn std::error::Error>>(())
        }
    )?;

    Ok(())
}

fn init_logger() {
    let log_name = "RUST_LOG";
    if env::var(log_name).is_err() {
        env::set_var(log_name, "error,proxy,resolver,forward");
    }
    pretty_env_logger::init_custom_env(log_name);
}
