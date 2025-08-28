use crate::{
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
        ports::{VulnRepository, VulnService},
    },
    errors::Error,
};
use error_stack::Result;

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
    async fn login(&self, req: &LoginRequest) -> Result<AdminUser, Error> {
        let res = self.repo.login(req).await?;
        Ok(res)
    }
    async fn create_sync_data_task(&self, req: CreateSyncDataTaskRequest) -> Result<i64, Error> {
        let ret = self.repo.create_sync_data_task(req).await?;
        Ok(ret)
    }

    async fn get_sync_data_task(&self) -> Result<Option<SyncDataTask>, Error> {
        let ret = self.repo.get_sync_data_task().await?;
        Ok(ret)
    }

    async fn list_vulnfusion_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> Result<ListVulnInformationResponseData, Error> {
        let ret = self.repo.list_vuln_information(req).await?; // Implement the logic here
        Ok(ret)
    }

    async fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> Result<Option<VulnInformation>, Error> {
        let ret = self.repo.get_vuln_information(req).await?;
        Ok(ret)
    }

    async fn get_ding_bot_config(&self) -> Result<Option<DingBotConfig>, Error> {
        let ret = self.repo.get_ding_bot_config().await?;
        Ok(ret)
    }

    async fn create_ding_bot_config(&self, req: CreateDingBotRequest) -> Result<i64, Error> {
        let ret = self.repo.create_ding_bot_config(req).await?;
        Ok(ret)
    }
}
