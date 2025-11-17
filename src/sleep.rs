use esp_hal::rtc_cntl::Rtc;
use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
use log::info;

/// Enter deep sleep mode with timer wakeup
///
/// # Arguments
/// * `rtc` - RTC controller
/// * `sleep_duration_secs` - Sleep duration in seconds
///
/// # Note
/// This function does not return - the device will reset when it wakes up.
/// If you first boot at 6 AM and sleep for 24 hours, the device will wake
/// at approximately 6 AM the next day.
pub fn enter_deep_sleep_secs(mut rtc: Rtc, sleep_duration_secs: u64) -> ! {
    info!("Entering deep sleep for {sleep_duration_secs} secs");

    // Configure timer wakeup source
    let timer = TimerWakeupSource::new(core::time::Duration::from_secs(sleep_duration_secs));

    // Enter deep sleep - this will not return, device will reset on wake
    rtc.sleep_deep(&[&timer]);
}
