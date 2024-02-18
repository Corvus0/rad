use regex::Regex;
use reqwest::header::REFERER;
use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum DownloadStatus {
    #[default]
    Initial,
    Downloading,
    Completed,
    Failed,
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
struct DownloadInfo {
    audio: String,
    title: String,
    extension: String,
    headers: HashMap<String, String>,
}

impl DownloadInfo {
    fn new(
        audio: String,
        title: String,
        extension: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            audio,
            title,
            extension,
            headers,
        }
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DownloadInput {
    url: String,
    op: String,
    sub: String,
}

impl DownloadInput {
    pub fn url(&self) -> &str {
        &self.url
    }

    async fn info_from_page(
        &self,
        audio_regex: &str,
        title_selector: &str,
    ) -> Result<(String, String, String), String> {
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
        let audio: String = serde_json::from_str(&format!(
            "\"{}\"",
            Regex::new(audio_regex)
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
        let extension = audio
            .split(".")
            .last()
            .ok_or(format!("Audio URL contains no valid file extension"))?
            .to_owned();
        let document = Html::parse_document(&html);
        let raw_title: String = Selector::parse(title_selector)
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
            .replace_all(&raw_title, "")
            .trim()
            .to_owned();
        Ok((audio, title, extension))
    }

    // Hostname regex pattern from URI spec: https://www.rfc-editor.org/rfc/rfc3986#appendix-B
    async fn parse_info(&self) -> Result<DownloadInfo, String> {
        let url_captures =
            Regex::new(r"^(([^:\/?#]+):)?(\/\/([^\/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?")
                .map_err(|e| e.to_string())
                .and_then(|re| {
                    re.captures(&self.url)
                        .ok_or(format!("Failed to match hostname: {}", &self.url))
                })?;
        let hostname = url_captures
            .get(4)
            .ok_or(format!("URL contains no valid hostname: {}", &self.url))?
            .as_str()
            .to_owned();
        let mut headers = HashMap::new();
        let (audio, title, extension) = match () {
            _ if hostname.contains("soundgasm.net") => {
                self.info_from_page(
                    r#"(https:\/\/media\.soundgasm\.net\/sounds\/[^\r\n\t\f\v"]+)"#,
                    "div.jp-title",
                )
                .await?
            }
            _ if hostname.contains("whyp.it") => {
                headers.insert(REFERER.to_string(), "https://whyp.it/".to_owned());
                self.info_from_page(
                    r#"(https:\\u002F\\u002Fcdn\.whyp\.it\\u002F[^\r\n\t\f\v"]+)"#,
                    "h1",
                )
                .await?
            }
            _ if hostname.contains("vocaroo.com") => {
                headers.insert(REFERER.to_string(), "https://vocaroo.com/".to_owned());
                let id = url_captures
                    .get(5)
                    .ok_or(format!("URL contains no valid id: {}", &self.url))?
                    .as_str()
                    .to_owned();
                (
                    format!("https://media1.vocaroo.com/mp3{id}"),
                    format!("Vocaroo {id}"),
                    "mp3".to_owned(),
                )
            }
            _ => {
                return Err(format!(
                    "URL contains invalid or unsupported host: {}",
                    &self.url
                ))
            }
        };
        Ok(DownloadInfo::new(audio, title, extension, headers))
    }

    pub async fn parse_input(self, id: usize) -> Result<DownloadItem, String> {
        let info = self.parse_info().await?;
        Ok(DownloadItem::new(self, info, id))
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
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

    pub fn extension(&self) -> &str {
        &self.info.extension
    }

    pub fn filename(&self) -> Result<String, String> {
        let filename = format!(
            "[{}] [{}] {}.{}",
            &self.input.sub, &self.input.op, &self.info.title, &self.info.extension,
        );
        Regex::new(r#"[<>:"/\\\?\*|]+"#)
            .map_err(|e| format!("Invalid regex pattern: {}", e.to_string()))
            .map(|re| re.replace_all(&filename, "").trim().to_owned())
    }
}
