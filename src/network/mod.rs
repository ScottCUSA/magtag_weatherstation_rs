pub mod http;

use embassy_net::IpAddress;
use embassy_net::dns::DnsQueryType;
use embassy_time::{Instant, with_deadline};

use crate::{
    config::RESOLVE_TIMEOUT,
    error::{AppError, Result},
};

async fn get_ip(
    host: &str,
    stack: &embassy_net::Stack<'static>,
) -> Result<heapless::Vec<IpAddress, 1>> {
    log::info!("resolving IP for {}...", host);
    match with_deadline(Instant::now() + RESOLVE_TIMEOUT, async {
        stack.dns_query(host, DnsQueryType::A).await
    })
    .await
    {
        Ok(Ok(addrs)) => {
            log::info!("resolved IP(s) for {:?}...", addrs);
            Ok(addrs)
        }
        Ok(Err(e)) => {
            log::error!("DNS query failed: {:?}", e);
            log::error!("Cannot resolve {}", host);
            Err(AppError::DnsQueryFailed)
        }
        Err(_) => {
            log::error!("DNS query timed out");
            Err(AppError::RequestTimeout)
        }
    }
}
