use error_stack::{Result, ResultExt};

use crate::{
    domain::{
        models::{
            sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
            vuln_information::{ListVulnInformationRequest, ListVulnInformationResponseData},
        },
        ports::VulnRepository,
    },
    errors::Error,
    output::db::{pg::Pg, sync_data_task::SyncDataTaskDao, vuln_information::VulnInformationDao},
};

impl VulnRepository for Pg {
    async fn create_sync_data_task(&self, req: CreateSyncDataTaskRequest) -> Result<i64, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let sync_data_task_id = SyncDataTaskDao::create(&mut tx, req).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(sync_data_task_id)
    }

    async fn get_sync_data_task(&self) -> Result<Option<SyncDataTask>, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let sync_data_task = SyncDataTaskDao::first(&mut tx).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(sync_data_task)
    }

    async fn list_vulnfusion_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> Result<ListVulnInformationResponseData, Error> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let vuln_informations =
            VulnInformationDao::filter_vulnfusion_information(&mut tx, &req.page_filter).await?;
        let count = VulnInformationDao::filter_vulnfusion_information_count(&mut tx).await?;

        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(ListVulnInformationResponseData {
            data: vuln_informations,
            total: count,
        })
    }
}
