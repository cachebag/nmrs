use async_trait::async_trait;
use zbus::Connection;

#[async_trait]
pub trait ActiveTransport {
    type Output;

    async fn current(conn: &Connection) -> Option<Self::Output>;
}
