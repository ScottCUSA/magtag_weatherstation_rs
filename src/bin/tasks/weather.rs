use embassy_net::Stack;
use embassy_time::{Duration, Timer};

use crate::{NETWORK_ERROR, NETWORK_READY, SLEEP_ON_ERROR_SECS, WEATHER_CHANNEL};
use core::fmt::Write;
use heapless::String;
use magtag_weatherstation::weather::fetch_weather;

#[embassy_executor::task]
pub async fn weather_fetcher_task(stack: Stack<'static>) {
    // Wait until network validator signals that network is ready
    NETWORK_READY.wait().await;

    // Try a few times to fetch the weather, otherwise send a NETWORK_ERROR
    const MAX_ATTEMPTS: usize = 3;
    for attempt in 0..MAX_ATTEMPTS {
        match fetch_weather(stack).await {
            Ok(weather_data) => {
                WEATHER_CHANNEL.send(weather_data).await;
                return;
            }
            Err(e) => {
                log::error!("Failed to fetch weather (attempt {}): {:?}", attempt + 1, e);
                if attempt + 1 >= MAX_ATTEMPTS {
                    let mut err_msg: String<128> = String::new();
                    let _ = write!(err_msg, "Failed to fetch weather: {:?}", e);
                    // notify display about network/fetch error
                    NETWORK_ERROR.send(err_msg).await;
                    return;
                }
                Timer::after(Duration::from_secs(SLEEP_ON_ERROR_SECS)).await;
            }
        }
    }
}
