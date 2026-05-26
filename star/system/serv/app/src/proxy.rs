use async_trait::async_trait;
use pingora::upstreams::peer::HttpPeer;
use pingora_http::ResponseHeader;
use pingora_proxy::{ProxyHttp, Session};
use pingora_web::StatusCode;
use std::sync::Arc;
use std::time::Duration;

use simple_filter_box::core::FilterBox;

use crate::filter::StarFilter;

pub struct StarProxy<T: FilterBox + Send + Sync> {
    pub filter: Arc<StarFilter<T>>,
    pub upstream_addr: String,
}

async fn exit_filter(
    status: impl TryInto<StatusCode>,
    session: &mut Session,
) -> pingora::Result<bool> {
    let header = ResponseHeader::build(status, None)?;
    session
        .write_response_header(Box::new(header), true)
        .await?;
    Ok(true)
}

#[async_trait]
impl<T: FilterBox + Send + Sync> ProxyHttp for StarProxy<T> {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        if session.req_header().method == "OPTIONS" {
            return exit_filter(204, session).await;
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let sni = match self.upstream_addr.split(':').next() {
            Some(host) => host,
            None => self.upstream_addr.as_str(),
        }
        .to_string();
        let mut peer = HttpPeer::new(self.upstream_addr.as_str(), false, sni);
        peer.options.connection_timeout = Some(Duration::from_secs(3));
        peer.options.total_connection_timeout = Some(Duration::from_secs(5));
        Ok(Box::new(peer))
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        Ok(())
    }
}
