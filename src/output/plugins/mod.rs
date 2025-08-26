pub mod avd;
pub mod kev;

use async_trait::async_trait;
use dashmap::DashMap;
use error_stack::Result;
use lazy_static::lazy_static;
use mea::mpsc::UnboundedSender;
use std::sync::Arc;

use crate::{
    domain::models::vuln_information::CreateVulnInformation,
    errors::Error,
    output::plugins::{avd::AVDPlugin, kev::KevPlugin},
};

lazy_static! {
    static ref PLUGINS: Arc<DashMap<String, Box<dyn VulnPlugin>>> = Arc::new(DashMap::new());
}

pub fn init(sender: UnboundedSender<CreateVulnInformation>) -> Result<(), Error> {
    KevPlugin::try_new(sender.clone())?;
    AVDPlugin::try_new(sender)?;
    Ok(())
}

#[async_trait]
pub trait VulnPlugin: Send + Sync + 'static {
    fn get_name(&self) -> String;
    fn get_display_name(&self) -> String;
    fn get_link(&self) -> String;
    async fn update(&self, page_limit: i32) -> Result<(), Error>;
}

pub fn register_plugin(name: String, plugin: Box<dyn VulnPlugin>) {
    PLUGINS.insert(name, plugin);
}

pub fn get_registry() -> Arc<DashMap<String, Box<dyn VulnPlugin>>> {
    PLUGINS.clone()
}

pub fn list_plugin_names() -> Vec<String> {
    PLUGINS.iter().map(|r| r.key().clone()).collect()
}
