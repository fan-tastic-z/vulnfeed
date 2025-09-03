use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use lazy_static::lazy_static;
use serde_json::Value;

use crate::{
    AppResult, domain::models::vuln_information::VulnInformation, errors::Error,
    utils::util::render_string,
};

pub mod ding_bot;

lazy_static! {
    static ref PLUGINS: Arc<DashMap<String, Box<dyn MessageBot>>> = Arc::new(DashMap::new());
}

#[async_trait]
pub trait MessageBot: Send + Sync {
    async fn push_markdown(&self, title: String, msg: String) -> AppResult<()>;
}

const VULN_INFO_MSG_TEMPLATE: &str = r####"
# {{ title }}

- CVE编号: {% if cve %} {{ cve }}{% else %}暂无 {% endif %}
- 危害定级: **{{ severity }}**
- 漏洞标签: {{ tags | join(sep=" ") }}
- 披露日期: **{{ disclosure }}**
- 推送原因: {{ reasons | join(sep=" ") }}
- 信息来源: [{{ source }}]

{% if description %}### **漏洞描述**
{{ description }}
{% endif %}
{% if solutions %}### **修复方案**
{{ solutions }}
{% endif %}
{% if references%}### **参考链接**
{% for reference in reference_links %}{{ loop.index }}. {{ reference }}
{% endfor %}{% endif %}

{% if cve %}### **开源检索**
{% if github_search | length > 0 %}{% for link in github_search %}{{ loop.index }}. {{ link }}
{% endfor %}{% else %}暂未找到{% endif %}{% endif %}"####;

const MAX_REFERENCE_LENGTH: usize = 8;

pub fn reader_vulninfo(mut vuln: VulnInformation) -> AppResult<String> {
    if vuln.reference_links.len() > MAX_REFERENCE_LENGTH {
        vuln.reference_links = vuln.reference_links[..MAX_REFERENCE_LENGTH].to_vec();
    }
    let json_value: Value = serde_json::to_value(vuln)
        .map_err(|e| Error::Message(format!("Failed to serialize vuln info: {e}")))?;
    let markdown = render_string(VULN_INFO_MSG_TEMPLATE, &json_value)?;
    Ok(markdown)
}

pub fn escape_markdown(input: String) -> String {
    input
        .replace('_', "\\_")
        .replace('.', "\\.")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('!', "\\!")
}
