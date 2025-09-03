use error_stack::ResultExt;

use crate::{
    AppResult,
    domain::{
        models::{
            admin_user::AdminUser,
            auth::LoginRequest,
            ding_bot::{CreateDingBotRequest, DingBotConfig},
            sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
            vuln_information::{
                GetVulnInformationRequest, ListVulnInformationRequest,
                ListVulnInformationResponseData, VulnInformation,
            },
        },
        ports::VulnRepository,
    },
    errors::Error,
    output::db::{
        admin_user::AdminUserDao, ding_bot_config::DingBotConfigDao, pg::Pg,
        sync_data_task::SyncDataTaskDao, vuln_information::VulnInformationDao,
    },
    utils::password_hash::verify_password_hash,
};

impl VulnRepository for Pg {
    async fn login(&self, req: &LoginRequest) -> AppResult<AdminUser> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let admin = AdminUserDao::fetch_by_name(&mut tx, &req.username).await?;
        if let Some(admin_user) = admin {
            if verify_password_hash(&req.password, &admin_user.password) {
                return Ok(admin_user);
            } else {
                log::error!("invalid account or password: {}", req.username);
            }
        }

        Err(Error::BadRequest("invalid account or password".to_string()).into())
    }

    async fn create_sync_data_task(&self, req: CreateSyncDataTaskRequest) -> AppResult<i64> {
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

    async fn get_sync_data_task(&self) -> AppResult<Option<SyncDataTask>> {
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

    async fn list_vuln_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> AppResult<ListVulnInformationResponseData> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let data = VulnInformationDao::filter_vulnfusion_information(
            &mut tx,
            &req.page_filter,
            &req.search_params,
        )
        .await?;
        let count =
            VulnInformationDao::filter_vulnfusion_information_count(&mut tx, &req.search_params)
                .await?;

        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(ListVulnInformationResponseData { data, total: count })
    }

    async fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> AppResult<Option<VulnInformation>> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let vuln_information = VulnInformationDao::fetch_by_id(&mut tx, req.id).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(vuln_information)
    }

    async fn create_ding_bot_config(&self, req: CreateDingBotRequest) -> AppResult<i64> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let id = DingBotConfigDao::create(&mut tx, req).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(id)
    }

    async fn get_ding_bot_config(&self) -> AppResult<Option<DingBotConfig>> {
        let mut tx =
            self.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let ding_bot_config = DingBotConfigDao::first(&mut tx).await?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("failed to commit transaction".to_string()))?;
        Ok(ding_bot_config)
    }
}
