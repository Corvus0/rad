use regex::Regex;
use scraper::{Html, Selector};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DownloadInput {
    url: String,
    op: String,
    sub: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadStatus {
    Initial,
    Downloading,
    Completed,
    Failed,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DownloadItem {
    input: DownloadInput,
    audio: String,
    title: String,
    status: DownloadStatus,
    id: usize,
    failure: Option<String>,
}

impl DownloadInput {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn parse_input(self, id: usize) -> Result<DownloadItem, String> {
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
        let audio_url = Regex::new(r#"(https:\/\/media.soundgasm.net\/sounds\/[^\r\n\t\f\v"]+)"#)
            .map_err(|e| e.to_string())
            .and_then(|re| {
                re.captures(&html)
                    .ok_or(format!("Failed to find valid audio url: {}", &self.url))
            })
            .and_then(|caps| {
                caps.get(0)
                    .ok_or(format!("Page contains no valid audio url: {}", &self.url))
            })?
            .as_str()
            .to_owned();
        let document = Html::parse_document(&html);
        let title = Selector::parse("div.jp-title")
            .map_err(|e| e.to_string())
            .and_then(|selector| {
                document
                    .select(&selector)
                    .next()
                    .ok_or(format!("Page does not contain title: {}", &self.url))
            })?
            .text()
            .collect();
        Ok(DownloadItem::new(self, audio_url, title, id))
    }
}

impl DownloadItem {
    fn new(input: DownloadInput, audio: String, title: String, id: usize) -> Self {
        Self {
            input,
            audio,
            title,
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
        &self.audio
    }

    pub fn title(&self) -> &str {
        &self.title
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
}
