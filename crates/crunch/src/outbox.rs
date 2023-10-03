use crate::{Persistence, Transport};

pub struct OutboxHandler {
    persistence: Persistence,
    transport: Transport,
}

impl OutboxHandler {
    pub fn new(persistence: Persistence, transport: Transport) -> Self {
        Self {
            persistence,
            transport,
        }
    }

    pub fn spawn(&mut self) {
        let p = self.persistence.clone();
        let t = self.transport.clone();
        tokio::spawn(async move {
            loop {
                match handle_messages(&p, &t).await {
                    Err(e) => {
                        tracing::error!("failed to handle message: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    Ok(None) => {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    _ => (),
                }
            }
        });
    }
}

async fn handle_messages(p: &Persistence, t: &Transport) -> anyhow::Result<Option<()>> {
    match p.next().await? {
        Some((item, _)) => match p.get(&item).await? {
            Some((info, content)) => {
                t.publish(&info, content).await?;
                p.update_published(&item).await?;
                tracing::debug!("published item: {}", item);
            }
            None => {
                tracing::info!("did not find any events for item: {}", item);
            }
        },
        None => return Ok(None),
    }

    Ok(Some(()))
}
