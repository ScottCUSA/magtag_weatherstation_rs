use core::time::Duration;

use esp_hal::{
    peripherals::LPWR,
    rtc_cntl::{Rtc, sleep::TimerWakeupSource},
};

use crate::SLEEP_CHANNEL;

#[derive(Debug)]
pub(crate) enum SleepReason {
    Success,
    HardwareInitError,
    DisplayError,
    NetworkError,
}

#[embassy_executor::task]
pub(crate) async fn deep_sleep_task(lpwr: LPWR<'static>) {
    // Initialize RTC for deep sleep
    let mut rtc = Rtc::new(lpwr);

    // Wait until sleep request is received
    let (sleep_seconds, reason) = SLEEP_CHANNEL.receive().await;
    log::info!("Received sleep request for {sleep_seconds} seconds. reason: {reason:?}");
    // Configure timer wakeup source
    let timer = TimerWakeupSource::new(Duration::from_secs(sleep_seconds));
    // Enter deep sleep - this will not return, device will reset on wake
    rtc.sleep_deep(&[&timer]);
}
