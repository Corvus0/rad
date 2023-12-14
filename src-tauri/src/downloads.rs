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
        let raw_html = match reqwest::get(&self.url).await {
            Ok(raw) => raw,
            Err(_) => return Err(format!("Failed to fetch page: {}", &self.url)),
        };
        let html = match raw_html.text().await {
            Ok(html) => html,
            Err(_) => return Err(format!("Failed to parse html to text for: {}", &self.url)),
        };
        let audio_url_re =
            match Regex::new(r#"(https:\/\/media.soundgasm.net\/sounds\/[^\r\n\t\f\v"]+)"#) {
                Ok(re) => re,
                Err(e) => return Err(e.to_string()),
            };
        let caps = match audio_url_re.captures(&html) {
            Some(caps) => caps,
            None => return Err(format!("Failed to find valid audio url: {}", &self.url)),
        };
        let audio_url = match caps.get(0) {
            Some(url) => url.as_str().to_owned(),
            None => return Err(format!("Page contains no valid audio url: {}", &self.url)),
        };
        let document = Html::parse_document(&html);
        let title_selector = match Selector::parse("div.jp-title") {
            Ok(selector) => selector,
            Err(e) => return Err(e.to_string()),
        };
        let title: String = match document.select(&title_selector).next() {
            Some(title) => title.text().collect(),
            None => return Err(format!("Page does not contain title: {}", &self.url)),
        };
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
