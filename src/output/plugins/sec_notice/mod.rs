pub mod fanruan;
pub mod firefox;
pub mod oracle;
pub mod seeyon;
pub mod smartbi;
pub mod vmware;
pub mod weaver;
pub mod yongyou;

use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use lazy_static::lazy_static;
use mea::mpsc::UnboundedSender;

use crate::{
    AppResult,
    domain::models::security_notice::CreateSecurityNotice,
    output::plugins::sec_notice::{
        fanruan::FanRuanNoticePlugin, firefox::FirefoxNoticePlugin, oracle::OracleNoticePlugin,
        seeyon::SeeyonNoticePlugin, smartbi::SmartbiNoticePlugin, vmware::VmwareNoticePlugin,
        weaver::WeaverNoticePlugin, yongyou::YongYouNoticePlugin,
    },
};

lazy_static! {
    static ref NOTICE: Arc<DashMap<String, Box<dyn SecNoticePlugin>>> = Arc::new(DashMap::new());
}

pub fn init(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<()> {
    YongYouNoticePlugin::try_new(sender.clone())?;
    WeaverNoticePlugin::try_new(sender.clone())?;
    SmartbiNoticePlugin::try_new(sender.clone())?;
    FanRuanNoticePlugin::try_new(sender.clone())?;
    SeeyonNoticePlugin::try_new(sender.clone())?;
    VmwareNoticePlugin::try_new(sender.clone())?;
    OracleNoticePlugin::try_new(sender.clone())?;
    FirefoxNoticePlugin::try_new(sender.clone())?;
    Ok(())
}

#[async_trait]
pub trait SecNoticePlugin: Send + Sync + 'static {
    fn get_name(&self) -> String;
    fn get_display_name(&self) -> String;
    fn get_link(&self) -> String;
    async fn update(&self, page_limit: i32) -> AppResult<()>;
}

pub fn register_sec_notice(name: String, sec_notice: Box<dyn SecNoticePlugin>) {
    NOTICE.insert(name, sec_notice);
}

pub fn get_notice_registry() -> Arc<DashMap<String, Box<dyn SecNoticePlugin>>> {
    NOTICE.clone()
}

pub fn list_sec_notice_names() -> Vec<String> {
    NOTICE.iter().map(|r| r.key().clone()).collect()
}
