use std::collections::HashMap;

use reqwest::header::{ACCEPT_LANGUAGE, USER_AGENT};
use reqwest::Client;
use serde_json::Value;
use soup::prelude::*;
use url::Url;

use crate::youtube::YouTube;

pub struct Playlist {
    videos_url: Vec<String>,
    pub title: String,
}

impl Playlist {
    pub async fn new(link: &str) -> Self {
        let mut ps_url = String::new();
        let client = Client::new();
        let parsed_url = Url::parse(link).unwrap();
        let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
        let api_url = "https://www.youtube.com/youtubei/v1/browse?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
        let bs = "https://www.youtube.com/watch?v={}";
        match hash_query.get("list") {
            Some(ps_id) => {
                ps_url = format!("https://www.youtube.com/playlist?list={}", ps_id);
            }
            None => {
                println!("Error")
            }
        }

        let response = client
            .get(ps_url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let soup = Soup::new(&response);
        let mut text = String::new();

        for i in soup.tag("script") {
            if i.text().contains("ytInitialData") {
                text = i.text();
                break;
            }
        }

        text = text.replace("var ytInitialData = ", "");
        let js: Value = serde_json::from_str(&text[..text.len() - 1]).unwrap();

        let title: String = js["metadata"]["playlistMetadataRenderer"]["title"].to_string();

        let mut videos_url: Vec<String> = Vec::new();

        let mut videos_json = js["contents"]["twoColumnBrowseResultsRenderer"]["tabs"][0]
            ["tabRenderer"]["content"]["sectionListRenderer"]["contents"][0]["itemSectionRenderer"]
            ["contents"][0]["playlistVideoListRenderer"]["contents"]
            .as_array()
            .unwrap()
            .clone();

        let mut continuation = videos_json[videos_json.len() - 1]["continuationItemRenderer"]
            ["continuationEndpoint"]["continuationCommand"]["token"]
            .to_string();

        if continuation != "null" {
            videos_json.pop();
        }

        for i in videos_json {
            videos_url
                .push(bs.replace("{}", &i["playlistVideoRenderer"]["videoId"].to_string()[..]));
        }

        while continuation != "null" {
            let data = r#"{
                "continuation": {},
                "context": {
                    "client": {"clientName": "WEB", "clientVersion": "2.20200720.00.02"}
                }
            }"#;

            let data = data.replace("{}", &continuation[..]);

            let js_response = client
                .post(api_url)
                .header(USER_AGENT, "Mozilla/5.0")
                .header(ACCEPT_LANGUAGE, "en-US,en")
                .header("X-YouTube-Client-Name", "1")
                .header("X-YouTube-Client-Version", "2.20200720.00.02")
                .body(data)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            let js: Value = serde_json::from_str(&js_response[..]).unwrap();
            videos_json = js["onResponseReceivedActions"][0]["appendContinuationItemsAction"]
                ["continuationItems"]
                .as_array()
                .unwrap()
                .clone();

            continuation = videos_json[videos_json.len() - 1]["continuationItemRenderer"]
                ["continuationEndpoint"]["continuationCommand"]["token"]
                .to_string();

            if continuation != "null" {
                videos_json.pop();
            }

            for i in videos_json {
                videos_url
                    .push(bs.replace("{}", &i["playlistVideoRenderer"]["videoId"].to_string()[..]));
            }
        }

        Self { videos_url, title }
    }

    pub async fn videos(&self) -> Vec<YouTube> {
        let mut videos: Vec<YouTube> = Vec::new();
        for i in &self.videos_url {
            let id = i.replace("\"", "");
            videos.push(YouTube::new(&id[..]).await.unwrap());
        }
        videos
    }
}
