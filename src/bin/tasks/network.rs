use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Instant, Timer, with_deadline};
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config, Interface, WifiController, sta::StationConfig};

use magtag_weatherstation::{
    config::{NETWORK_IP_TIMEOUT_SECS, NETWORK_LINK_TIMEOUT_SECS, WIFI_PASSWORD, WIFI_SSID},
    error::{AppError, Result},
    mk_static,
};

use core::fmt::Write;
use heapless::String;

use crate::{NETWORK_ERROR, NETWORK_READY};

/// Manage the WiFi connection lifecycle.
///
/// Configures and starts the provided `WifiController`, attempts connection,
/// and retries on failures or after disconnects.
#[embassy_executor::task]
pub(crate) async fn wifi_task(mut controller: WifiController<'static>) {
    log::info!("Initializing wifi");
    loop {
        if controller.is_connected() {
            let _ = controller.wait_for_disconnect_async().await;
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }
        log::info!("Attempting to connect to WiFi network SSID: {}", WIFI_SSID);
        let client_config = Config::Station(
            StationConfig::default()
                .with_ssid(WIFI_SSID)
                .with_password(WIFI_PASSWORD.into()),
        );
        if let Err(e) = controller.set_config(&client_config) {
            log::error!("Failed to set WiFi config: {:?}", e);
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }
        log::info!("About to connect");
        match controller.connect_async().await {
            Ok(_) => log::info!("Wifi connected!"),
            Err(e) => {
                log::error!("Failed to connect to wifi: {e:>}");
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Run the embassy-net network event runner.
///
/// Drives the network stack by running the provided `Runner` until completion.
#[embassy_executor::task]
pub(crate) async fn net_runner_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}

/// Validate network readiness (link and IP assignment).
///
/// Waits for link up and IPv4 configuration with timeouts, signals
/// `NETWORK_READY` when an IP is acquired or sends `NETWORK_ERROR` on failure.
#[embassy_executor::task]
pub(crate) async fn net_validator_task(stack: embassy_net::Stack<'static>) {
    // Wait for Link (timeout configured)
    if with_deadline(
        Instant::now() + Duration::from_secs(NETWORK_LINK_TIMEOUT_SECS),
        async {
            loop {
                if stack.is_link_up() {
                    break;
                }
                Timer::after(Duration::from_millis(500)).await;
            }
        },
    )
    .await
    .is_err()
    {
        log::error!("Link failed");
        // Notify display about the network link failure
        let mut msg: String<128> = String::new();
        let _ = write!(msg, "Network link failed");
        NETWORK_ERROR.signal(msg);
        return;
    }

    // Wait for IP (timeout configured)
    if with_deadline(
        Instant::now() + Duration::from_secs(NETWORK_IP_TIMEOUT_SECS),
        async {
            loop {
                if let Some(config) = stack.config_v4() {
                    log::info!("Network ready with IP: {}", config.address);
                    NETWORK_READY.signal(());
                    break;
                }
                Timer::after(Duration::from_millis(500)).await;
            }
        },
    )
    .await
    .is_err()
    {
        log::error!("Timed out waiting for IP address");
        let mut msg: String<128> = String::new();
        let _ = write!(msg, "Timed out waiting for IP address");
        NETWORK_ERROR.signal(msg);
    }
}

pub(crate) struct RadioResources {
    pub(crate) wifi: esp_hal::peripherals::WIFI<'static>,
}

pub(crate) fn init_radio(
    resources: RadioResources,
) -> Result<(WifiController<'static>, Interface<'static>)> {
    let (controller, interfaces) = match esp_radio::wifi::new(resources.wifi, Default::default()) {
        Ok(wifi) => wifi,
        Err(e) => {
            let mut msg: String<128> = String::new();
            let _ = write!(msg, "Failed to initialize WiFi: {:?}", e);
            log::error!("{msg}");
            NETWORK_ERROR.signal(msg);
            return Err(AppError::ConnectionFailed);
        }
    };

    Ok((controller, interfaces.station))
}

pub(crate) fn init_network_stack(
    wifi_interface: Interface<'static>,
) -> (Stack<'static>, Runner<'static, Interface<'static>>) {
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);
    embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    )
}
