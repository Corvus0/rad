use std::collections::HashMap;

use regex::Regex;
use reqwest::header::REFERER;
use scraper::{Html, Selector};

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadStatus {
    Initial,
    Downloading,
    Completed,
    Failed,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct DownloadInfo {
    audio: String,
    title: String,
    headers: HashMap<String, String>,
}

impl DownloadInfo {
    fn new(audio: String, title: String, headers: HashMap<String, String>) -> Self {
        Self {
            audio,
            title,
            headers,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DownloadInput {
    url: String,
    op: String,
    sub: String,
}

impl DownloadInput {
    pub fn url(&self) -> &str {
        &self.url
    }

    // Hostname regex pattern from URI spec: https://www.rfc-editor.org/rfc/rfc3986#appendix-B
    fn parse_hostname(
        &self,
        headers: &mut HashMap<String, String>,
    ) -> Result<(String, String), String> {
        let hostname = Regex::new(r"^(([^:\/?#]+):)?(\/\/([^\/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?")
            .map_err(|e| e.to_string())
            .and_then(|re| {
                re.captures(&self.url)
                    .ok_or(format!("Failed to match hostname: {}", &self.url))
            })
            .and_then(|caps| {
                caps.get(4)
                    .ok_or(format!("URL contains no valid hostname: {}", &self.url))
            })?
            .as_str();
        let (audio_regex, title_selector) = match () {
            _ if hostname.contains("soundgasm.net") => (
                r#"(https:\/\/media.soundgasm.net\/sounds\/[^\r\n\t\f\v"]+)"#,
                "div.jp-title",
            ),
            _ if hostname.contains("whyp.it") => {
                headers.insert(REFERER.to_string(), "https://whyp.it/".to_owned());
                (
                    r#"(https:\\u002F\\u002Fcdn.whyp.it\\u002F[^\r\n\t\f\v"]+)"#,
                    "h1",
                )
            }
            _ => {
                return Err(format!(
                    "URL contains invalid or unsupported host: {}",
                    &self.url
                ))
            }
        };
        Ok((audio_regex.to_owned(), title_selector.to_owned()))
    }

    async fn parse_info(&self) -> Result<DownloadInfo, String> {
        let mut headers = HashMap::new();
        let (audio_regex, title_selector) = self.parse_hostname(&mut headers)?;
        let response = reqwest::get(&self.url)
            .await
            .map_err(|e| format!("Failed to fetch page {}: {}", &self.url, e.to_string()))?;
        let html = response.text().await.map_err(|e| {
            format!(
                "Failed to parse html to text {}: {}",
                &self.url,
                e.to_string()
            )
        })?;
        let audio = serde_json::from_str(&format!(
            "\"{}\"",
            Regex::new(&audio_regex)
                .map_err(|e| e.to_string())
                .and_then(|re| {
                    re.captures(&html)
                        .ok_or(format!("Failed to find valid audio url: {}", &self.url))
                })
                .and_then(|caps| {
                    caps.get(0)
                        .ok_or(format!("Page contains no valid audio url: {}", &self.url))
                })?
                .as_str(),
        ))
        .map_err(|e| e.to_string())?;
        let document = Html::parse_document(&html);
        let raw_title: String = Selector::parse(&title_selector)
            .map_err(|e| e.to_string())
            .and_then(|selector| {
                document
                    .select(&selector)
                    .next()
                    .ok_or(format!("Page does not contain title: {}", &self.url))
            })?
            .text()
            .collect();
        let title = Regex::new(r"(\[.+?\])")
            .map_err(|e| e.to_string())?
            .replace_all(&raw_title, "");
        Ok(DownloadInfo::new(audio, title.trim().to_string(), headers))
    }

    pub async fn parse_input(self, id: usize) -> Result<DownloadItem, String> {
        let info = self.parse_info().await?;
        Ok(DownloadItem::new(self, info, id))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DownloadItem {
    input: DownloadInput,
    info: DownloadInfo,
    status: DownloadStatus,
    id: usize,
    failure: Option<String>,
}

impl DownloadItem {
    fn new(input: DownloadInput, info: DownloadInfo, id: usize) -> Self {
        Self {
            input,
            info,
            status: DownloadStatus::Initial,
            id,
            failure: None,
        }
    }

    pub async fn parse_input(self) -> Result<Self, String> {
        self.input.parse_input(self.id).await
    }

    pub fn url(&self) -> &str {
        &self.input.url
    }

    pub fn op(&self) -> &str {
        &self.input.op
    }

    pub fn sub(&self) -> &str {
        &self.input.sub
    }

    pub fn audio(&self) -> &str {
        &self.info.audio
    }

    pub fn title(&self) -> &str {
        &self.info.title
    }

    pub fn set_status(&mut self, status: DownloadStatus) {
        self.status = status;
    }

    pub fn set_failure(&mut self, failure: String) {
        self.failure = Some(failure);
    }

    pub fn is_completed(&self) -> bool {
        self.status == DownloadStatus::Completed
    }

    pub fn id(&self) -> &usize {
        &self.id
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.info.headers
    }
}
