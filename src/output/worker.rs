use std::sync::Arc;

use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedReceiver;

use crate::{
    domain::models::vuln_information::CreateVulnInformation,
    errors::Error,
    output::{
        db::{ding_bot_config::DingBotConfigDao, pg::Pg, vuln_information::VulnInformationDao},
        push::{MessageBot, ding_bot::DingBot, reader_vulninfo},
    },
};

pub struct Worker {
    pub receiver: UnboundedReceiver<CreateVulnInformation>,
    pub pg: Arc<Pg>,
}

impl Worker {
    pub fn new(receiver: UnboundedReceiver<CreateVulnInformation>, pg: Pg) -> Self {
        Worker {
            receiver,
            pg: Arc::new(pg),
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Some(req) = self.receiver.recv().await {
            match self.store(req).await {
                Err(e) => {
                    log::error!("Failed to store vuln information: {:?}", e);
                    continue;
                }
                Ok((id, as_new_vuln)) => {
                    if as_new_vuln {
                        self.ding_bot_push(id).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn ding_bot_push(&self, id: i64) -> Result<(), Error> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let ding_bot_config = DingBotConfigDao::first(&mut tx).await?;
        if let Some(config) = ding_bot_config
            && config.status
        {
            let vuln = VulnInformationDao::fetch_by_id(&mut tx, id).await?;
            if let Some(v) = vuln {
                let ding = DingBot::try_new(config.access_token, config.secret_token)?;
                let title = v.title.clone();
                let msg = match reader_vulninfo(v) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Failed to read vuln info: {:?}", e);
                        return Err(
                            Error::Message(format!("Failed to read vuln info: {:?}", e)).into()
                        );
                    }
                };
                ding.push_markdown(title, msg).await?;
                VulnInformationDao::update_status(&mut tx, id, true).await?;
            }
        }
        Ok(())
    }

    pub async fn store(&self, req: CreateVulnInformation) -> Result<(i64, bool), Error> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;

        let (id, as_new_vuln) = VulnInformationDao::create_or_update(&mut tx, req).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok((id, as_new_vuln))
    }
}
