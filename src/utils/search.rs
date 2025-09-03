use regex::Regex;

use crate::{AppResult, errors::Error, utils::util::get_last_year_data};

pub async fn search_github_poc(cve_id: &str) -> Vec<String> {
    let mut res = Vec::new();
    let (nuclei_res, repo_res) = tokio::join!(search_nuclei_pr(cve_id), search_github_repo(cve_id));
    match nuclei_res {
        Ok(nuclei) => res.extend(nuclei),
        Err(e) => {
            log::warn!("search nucli pr error:{}", e);
        }
    }
    match repo_res {
        Ok(repo) => res.extend(repo),
        Err(e) => {
            log::warn!("search github repo error:{}", e);
        }
    }
    res
}

pub async fn search_nuclei_pr(cve_id: &str) -> AppResult<Vec<String>> {
    log::info!("search nuclei PR of {}", cve_id);
    let page = octocrab::instance()
        .pulls("projectdiscovery", "nuclei-templates")
        .list()
        .per_page(100)
        .page(1u32)
        .send()
        .await
        .map_err(|e| Error::Message(format!("Failed to search nuclei pr: {}", e)))?;
    let re = Regex::new(&format!(r"(?i)(?:\b|/|_){}(?:\b|/|_)", cve_id))
        .map_err(|e| Error::Message(format!("Failed to compile regex: {}", e)))?;
    let links = page
        .into_iter()
        .filter(|pull| pull.title.is_some() || pull.body.is_some())
        .filter(|pull| {
            re.is_match(pull.title.as_ref().unwrap_or(&String::new()))
                || re.is_match(pull.body.as_ref().unwrap_or(&String::new()))
        })
        .filter_map(|pull| pull.html_url)
        .map(|u| u.to_string())
        .collect::<Vec<_>>();
    Ok(links)
}

pub async fn search_github_repo(cve_id: &str) -> AppResult<Vec<String>> {
    log::info!("search github repo of {}", cve_id);
    let last_year = get_last_year_data();
    let query = format!(
        "language:Python language:JavaScript language:C language:C++ language:Java language:PHP language:Ruby language:Rust language:C# created:>{} {}",
        last_year, cve_id
    );
    let page = octocrab::instance()
        .search()
        .repositories(&query)
        .per_page(100)
        .page(1u32)
        .send()
        .await
        .map_err(|e| Error::Message(format!("Failed to search github repo: {}", e)))?;
    let re = Regex::new(&format!(r"(?i)(?:\b|/|_){}(?:\b|/|_)", cve_id))
        .map_err(|e| Error::Message(format!("Failed to compile regex: {}", e)))?;
    let links = page
        .into_iter()
        .filter_map(|r| r.html_url)
        .filter(|url| re.captures(url.as_str()).is_some())
        .map_while(|u| Some(u.to_string()))
        .collect::<Vec<_>>();
    Ok(links)
}
