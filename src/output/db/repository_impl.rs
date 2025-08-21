use error_stack::{Result, ResultExt};

use crate::{
    domain::{models::sync_data_task::CreateSyncDataTaskRequest, ports::VulnRepository},
    errors::Error,
    output::db::{pg::Pg, sync_data_task::SyncDataTaskDao},
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
}
