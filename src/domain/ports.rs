use crate::{
    AppResult,
    domain::models::{
        admin_user::AdminUser,
        auth::LoginRequest,
        ding_bot::{CreateDingBotRequest, DingBotConfig},
        sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
        vuln_information::{
            GetVulnInformationRequest, ListVulnInformationRequest, ListVulnInformationResponseData,
            VulnInformation,
        },
    },
};
use std::future::Future;

pub trait VulnService: Clone + Send + Sync + 'static {
    fn login(&self, req: &LoginRequest) -> impl Future<Output = AppResult<AdminUser>> + Send;
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = AppResult<i64>> + Send;
    fn get_sync_data_task(&self) -> impl Future<Output = AppResult<Option<SyncDataTask>>> + Send;

    fn list_vulnfusion_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> impl Future<Output = AppResult<ListVulnInformationResponseData>> + Send;

    fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> impl Future<Output = AppResult<Option<VulnInformation>>> + Send;

    fn create_ding_bot_config(
        &self,
        req: CreateDingBotRequest,
    ) -> impl Future<Output = AppResult<i64>> + Send;

    fn get_ding_bot_config(&self) -> impl Future<Output = AppResult<Option<DingBotConfig>>> + Send;
}

pub trait VulnRepository: Clone + Send + Sync + 'static {
    fn login(&self, req: &LoginRequest) -> impl Future<Output = AppResult<AdminUser>> + Send;
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = AppResult<i64>> + Send;

    fn get_sync_data_task(&self) -> impl Future<Output = AppResult<Option<SyncDataTask>>> + Send;

    fn list_vuln_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> impl Future<Output = AppResult<ListVulnInformationResponseData>> + Send;

    fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> impl Future<Output = AppResult<Option<VulnInformation>>> + Send;

    fn create_ding_bot_config(
        &self,
        req: CreateDingBotRequest,
    ) -> impl Future<Output = AppResult<i64>> + Send;

    fn get_ding_bot_config(&self) -> impl Future<Output = AppResult<Option<DingBotConfig>>> + Send;
}
