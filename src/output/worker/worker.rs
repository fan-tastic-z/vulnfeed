use std::sync::Arc;

use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedReceiver;

use crate::{
    domain::models::vuln_information::CreateVulnInformation,
    errors::Error,
    output::db::{pg::Pg, vuln_information::VulnInformationDao},
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
            if let Err(e) = self.store(req).await {
                log::error!("Failed to store vuln information: {:?}", e);
                continue;
            }
        }
        Ok(())
    }

    pub async fn store(&self, req: CreateVulnInformation) -> Result<(), Error> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;

        let _ = VulnInformationDao::create_or_update(&mut tx, req).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(())
    }
}
