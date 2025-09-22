use poem::{handler, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    input::http::response::{ApiError, ApiSuccess},
    output::plugins::vuln::get_registry,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PluginData {
    pub name: String,
    pub display_name: String,
    pub link: String,
}

#[handler]
pub async fn list_plugins() -> Result<ApiSuccess<Vec<PluginData>>, ApiError> {
    let res = get_registry()
        .iter()
        .map(|plugin| PluginData {
            name: plugin.get_name(),
            display_name: plugin.get_display_name(),
            link: plugin.get_link(),
        })
        .collect::<Vec<_>>();
    Ok(ApiSuccess::new(StatusCode::OK, res))
}
