use embassy_net::Runner;
use embassy_time::{Duration, Instant, Timer, with_deadline};
use esp_radio::wifi::{
    ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};

use magtag_weatherstation::config::{
    NETWORK_IP_TIMEOUT_SECS, NETWORK_LINK_TIMEOUT_SECS, WIFI_PASSWORD, WIFI_SSID,
};

use crate::{NETWORK_ERROR, NETWORK_READY};
use core::fmt::Write;
use heapless::String;

/// Manage the WiFi connection lifecycle.
///
/// Configures and starts the provided `WifiController`, attempts connection,
/// and retries on failures or after disconnects.
#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    log::info!("Starting connection task");
    log::info!("Device capabilities {:?}", controller.capabilities());
    loop {
        if esp_radio::wifi::sta_state() == WifiStaState::Connected {
            // wait untill disconnected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_secs(5)).await;
        }
        if !matches!(controller.is_started(), Ok(true)) {
            log::info!("Attempting to connect to WiFi network SSID: {}", WIFI_SSID);
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(WIFI_SSID.into())
                    .with_password(WIFI_PASSWORD.into()),
            );
            if let Err(e) = controller.set_config(&client_config) {
                log::error!("Failed to set WiFi config: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            log::info!("Starting Wifi");
            if let Err(e) = controller.start_async().await {
                log::error!("Failed to start WiFi: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            log::info!("Wifi Started");

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
}

/// Run the embassy-net network event runner.
///
/// Drives the network stack by running the provided `Runner` until completion.
#[embassy_executor::task]
pub async fn net_runner_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

/// Validate network readiness (link and IP assignment).
///
/// Waits for link up and IPv4 configuration with timeouts, signals
/// `NETWORK_READY` when an IP is acquired or sends `NETWORK_ERROR` on failure.
#[embassy_executor::task]
pub async fn net_validator_task(stack: embassy_net::Stack<'static>) {
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
        NETWORK_ERROR.send(msg).await;
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
        NETWORK_ERROR.send(msg).await;
    }
}
