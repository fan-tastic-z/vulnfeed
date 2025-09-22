use crate::{
    AppResult,
    domain::{
        models::{
            admin_user::AdminUser,
            auth::LoginRequest,
            ding_bot::{CreateDingBotRequest, DingBotConfig},
            security_notice::{ListSecNoticeRequest, ListSecNoticeResponseData},
            sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
            vuln_information::{
                GetVulnInformationRequest, ListVulnInformationRequest,
                ListVulnInformationResponseData, VulnInformation,
            },
        },
        ports::{VulnRepository, VulnService},
    },
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: VulnRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: VulnRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> VulnService for Service<R>
where
    R: VulnRepository,
{
    async fn login(&self, req: &LoginRequest) -> AppResult<AdminUser> {
        let res = self.repo.login(req).await?;
        Ok(res)
    }
    async fn create_sync_data_task(&self, req: CreateSyncDataTaskRequest) -> AppResult<i64> {
        let ret = self.repo.create_sync_data_task(req).await?;
        Ok(ret)
    }

    async fn get_sync_data_task(&self) -> AppResult<Option<SyncDataTask>> {
        let ret = self.repo.get_sync_data_task().await?;
        Ok(ret)
    }

    async fn list_vulnfusion_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> AppResult<ListVulnInformationResponseData> {
        let ret = self.repo.list_vuln_information(req).await?;
        Ok(ret)
    }

    async fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> AppResult<Option<VulnInformation>> {
        let ret = self.repo.get_vuln_information(req).await?;
        Ok(ret)
    }

    async fn get_ding_bot_config(&self) -> AppResult<Option<DingBotConfig>> {
        let ret = self.repo.get_ding_bot_config().await?;
        Ok(ret)
    }

    async fn create_ding_bot_config(&self, req: CreateDingBotRequest) -> AppResult<i64> {
        let ret = self.repo.create_ding_bot_config(req).await?;
        Ok(ret)
    }

    async fn list_sec_notice(
        &self,
        req: ListSecNoticeRequest,
    ) -> AppResult<ListSecNoticeResponseData> {
        let ret = self.repo.list_sec_notice(req).await?;
        Ok(ret)
    }
}
