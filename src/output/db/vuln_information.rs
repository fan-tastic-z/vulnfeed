use error_stack::Result;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::models::{
        page_utils::PageFilter,
        vuln_information::{CreateVulnInformation, VulnInformation},
    },
    errors::Error,
    output::db::base::{
        Dao, DaoQueryBuilder, dao_create, dao_fetch_by_column, dao_fetch_by_id, dao_update,
    },
};

const REASON_NEW_CREATED: &str = "漏洞创建";
const REASON_TAG_UPDATED: &str = "标签更新";
const REASON_SEVERITY_UPDATE: &str = "等级更新";

pub struct VulnInformationDao;

impl Dao for VulnInformationDao {
    const TABLE: &'static str = "vuln_information";
}

impl VulnInformationDao {
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateVulnInformation,
    ) -> Result<i64, Error> {
        let id = dao_create::<Self, _>(tx, req).await?;
        Ok(id)
    }

    /// 检查漏洞严重性是否发生变化，如果变化则记录日志并添加变更原因
    fn check_severity_update(vuln: &mut VulnInformation, req: &CreateVulnInformation) -> bool {
        let severity = vuln.severity.to_string();
        if severity != req.severity {
            log::info!(
                "{} from {} change severity from {} to {}",
                vuln.title.as_str(),
                vuln.source.as_str(),
                vuln.severity.as_str(),
                req.severity.as_str()
            );
            let reason = format!(
                "{}: {} => {}",
                REASON_SEVERITY_UPDATE,
                vuln.severity.as_str(),
                req.severity
            );
            vuln.reasons.push(reason);
            true
        } else {
            false
        }
    }

    /// 检查漏洞标签是否发生变化，如果变化则记录日志并添加变更原因
    fn check_tag_update(vuln: &mut VulnInformation, req: &CreateVulnInformation) -> bool {
        let new_tags = req
            .tags
            .iter()
            .filter(|&x| !vuln.tags.contains(x))
            .collect::<Vec<_>>();
        if !new_tags.is_empty() {
            log::info!(
                "{} from {} add new tag {:?}",
                vuln.title,
                vuln.source,
                new_tags
            );
            let reason = format!("{}: {:?} => {:?}", REASON_TAG_UPDATED, vuln.tags, req.tags);
            vuln.reasons.push(reason);
            true
        } else {
            false
        }
    }

    pub async fn create_or_update(
        tx: &mut Transaction<'_, Postgres>,
        mut req: CreateVulnInformation,
    ) -> Result<(), Error> {
        if let Some(mut vuln) =
            dao_fetch_by_column::<Self, VulnInformation>(tx, "key", &req.key).await?
        {
            let mut as_new_vuln = false;
            as_new_vuln |= VulnInformationDao::check_severity_update(&mut vuln, &req);
            as_new_vuln |= VulnInformationDao::check_tag_update(&mut vuln, &req);
            if as_new_vuln {
                req.pushed = false;
                dao_update::<Self, _>(tx, vuln.id, req).await?;
                Ok(())
            } else {
                log::warn!("Vuln information already exists: {}", req.key);
                Ok(())
            }
        } else {
            log::info!("New vulnerability created: {}", req.key);
            req.reasons.push(REASON_NEW_CREATED.to_string());
            dao_create::<Self, _>(tx, req).await?;
            Ok(())
        }
    }

    pub async fn filter_vulnfusion_information(
        tx: &mut Transaction<'_, Postgres>,
        page_filter: &PageFilter,
        search: Option<&str>,
    ) -> Result<Vec<VulnInformation>, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();
        let page_no = *page_filter.page_no().as_ref();
        let page_size = *page_filter.page_size().as_ref();
        let offset = (page_no - 1) * page_size;

        // 添加搜索条件
        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            query_builder = query_builder
                .and_where_like("title", search_term)
                .and_where_like("description", search_term);
        }

        query_builder
            .order_by_desc("id")
            .limit_offset(page_size as i64, offset as i64)
            .fetch_all(tx)
            .await
    }

    pub async fn filter_vulnfusion_information_count(
        tx: &mut Transaction<'_, Postgres>,
        search: Option<&str>,
    ) -> Result<i64, Error> {
        let mut query_builder = DaoQueryBuilder::<Self>::new();

        // 添加搜索条件
        if let Some(search_term) = search
            && !search_term.is_empty()
        {
            query_builder = query_builder
                .and_where_like("title", search_term)
                .and_where_like("description", search_term);
        }

        query_builder.count(tx).await
    }

    pub async fn fetch_by_id(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<Option<VulnInformation>, Error> {
        dao_fetch_by_id::<Self, VulnInformation>(tx, id).await
    }
}
