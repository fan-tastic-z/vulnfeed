use std::sync::Arc;

use error_stack::ResultExt;
use mea::mpsc::UnboundedReceiver;

use crate::{
    AppResult,
    domain::models::security_notice::CreateSecurityNotice,
    errors::Error,
    output::{
        db::{ding_bot_config::DingBotConfigDao, pg::Pg, security_notice::SecurityNoticeDao},
        push::{MessageBot, ding_bot::DingBot, render_sec_notice},
    },
};

pub struct SecNoticeWorker {
    pub receiver: UnboundedReceiver<CreateSecurityNotice>,
    pub pg: Arc<Pg>,
}

impl SecNoticeWorker {
    pub fn new(receiver: UnboundedReceiver<CreateSecurityNotice>, pg: Pg) -> Self {
        SecNoticeWorker {
            receiver,
            pg: Arc::new(pg),
        }
    }

    pub async fn run(&mut self) -> AppResult<()> {
        while let Some(req) = self.receiver.recv().await {
            match self.store(req).await {
                Err(e) => {
                    log::error!("Failed to store vuln information: {:?}", e);
                    continue;
                }
                Ok((id, created)) => {
                    if created {
                        self.ding_bot_push(id).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn ding_bot_push(&self, id: i64) -> AppResult<()> {
        log::info!("ding bot push start! id: {}", id);
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let ding_bot_config = DingBotConfigDao::first(&mut tx).await?;
        if let Some(config) = ding_bot_config
            && config.status
        {
            let sec_notice = SecurityNoticeDao::fetch_by_id(&mut tx, id).await?;
            if let Some(s) = sec_notice
                && !s.pushed
            {
                let ding = DingBot::try_new(config.access_token, config.secret_token)?;
                let title = s.title.clone();
                let msg = match render_sec_notice(s) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Failed to render sec notice: {:?}", e);
                        return Err(Error::Message(format!(
                            "Failed to render sec notice: {:?}",
                            e
                        ))
                        .into());
                    }
                };
                ding.push_markdown(title, msg).await?;
                log::info!("ding bot push success! id: {}", id);
                SecurityNoticeDao::update_pushed(&mut tx, id, true).await?;
            }
        } else {
            log::info!("ding bot config not found or status is false");
        }
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(())
    }

    pub async fn store(&self, req: CreateSecurityNotice) -> AppResult<(i64, bool)> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let res = SecurityNoticeDao::fetch_by_key(&mut tx, &req.key).await?;
        let (id, created) = match res {
            None => {
                let id = SecurityNoticeDao::create(&mut tx, req).await?;
                (id, true)
            }
            Some(sec_notice) => (sec_notice.id, false),
        };

        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok((id, created))
    }
}
