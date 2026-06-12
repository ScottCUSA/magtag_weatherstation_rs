use core::time::Duration;

use esp_hal::{
    peripherals::LPWR,
    rtc_cntl::{Rtc, sleep::TimerWakeupSource},
};

use crate::SLEEP_REQUEST;

#[derive(Debug)]
pub(crate) enum SleepReason {
    Success,
    HardwareInitError,
    DisplayError,
    NetworkError,
}

#[embassy_executor::task]
pub(crate) async fn deep_sleep_task(lpwr: LPWR<'static>) {
    let mut rtc = Rtc::new(lpwr);
    let (sleep_seconds, reason) = SLEEP_REQUEST.wait().await;
    log::info!("Received sleep request for {sleep_seconds} seconds. reason: {reason:?}");
    let timer = TimerWakeupSource::new(Duration::from_secs(sleep_seconds));
    rtc.sleep_deep(&[&timer]);
}
