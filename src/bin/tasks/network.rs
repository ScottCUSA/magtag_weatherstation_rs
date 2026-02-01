use embassy_net::Runner;
use embassy_time::{Duration, Instant, Timer, with_deadline};
use esp_radio::wifi::{
    ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};
use log::{error, info};

use magtag_weatherstation::config::{WIFI_PASSWORD, WIFI_SSID};

use crate::{NETWORK_ERROR, NETWORK_READY};
use core::fmt::Write;
use heapless::String;

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    info!("Starting connection task");
    info!("Device capabilities {:?}", controller.capabilities());
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
                error!("Failed to set WiFi config: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            info!("Starting Wifi");
            if let Err(e) = controller.start_async().await {
                error!("Failed to start WiFi: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            info!("Wifi Started");

            info!("About to connect");
            match controller.connect_async().await {
                Ok(_) => info!("Wifi connected!"),
                Err(e) => {
                    error!("Failed to connect to wifi: {e:>}");
                    Timer::after(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn network_validator_task(stack: embassy_net::Stack<'static>) {
    // Wait for Link (timeout ~10s)
    if with_deadline(Instant::now() + Duration::from_secs(10), async {
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    })
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

    // Wait for IP (timeout ~20s)
    if with_deadline(Instant::now() + Duration::from_secs(20), async {
        loop {
            if let Some(config) = stack.config_v4() {
                log::info!("Network ready with IP: {}", config.address);
                NETWORK_READY.signal(());
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    })
    .await
    .is_err()
    {
        log::error!("Timed out waiting for IP address");
        let mut msg: String<128> = String::new();
        let _ = write!(msg, "Timed out waiting for IP address");
        NETWORK_ERROR.send(msg).await;
    }
}
