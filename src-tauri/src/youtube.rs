use std::collections::HashMap;

use crate::stream::Stream;
use maplit::hashmap;
use reqwest::Client;
use serde_json::Value;
use url::Url;

pub struct YouTube {
    pub title: String,
    streams: Vec<Stream>,
}

impl YouTube {
    pub async fn new(id: &str) -> Result<Self, u8> {
        let parsed_url = Url::parse(id).unwrap();
        let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
        let v_id: String;
        match hash_query.get("v") {
            Some(video_id) => {
                v_id = video_id.clone();
            }
            None => return Err(1),
        }
        let client: Client = Client::new();
        let j_payload: &str = r#"{
            "context": {
                "client": {
                    "clientName": "ANDROID_EMBEDDED_PLAYER",
                    "clientVersion": "17.31.35",
                    "clientScreen": "EMBED",
                    "androidSdkVersion": 30,
                }
            }
        }"#;

        let json_obj: Value = serde_json::from_str(&client
            .post(format!("https://www.youtube.com/youtubei/v1/player?videoId={v_id}&key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8&contentCheckOk=True&racyCheckOk=True"))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(j_payload)
            .send()
            .await
            .unwrap()
            .text()
            .await.unwrap()[..]).unwrap();

        let title: String = String::from(
            json_obj["videoDetails"]["title"]
                .as_str()
                .expect("Vídeo Inválido"),
        );
        let stream_list: Vec<Value>;
        match json_obj["streamingData"]["adaptiveFormats"].as_array() {
            Some(streams) => {
                stream_list = streams.clone();
            }
            None => {
                let json_payload: &str = r#"{
                        "context": {
                            "client": {
                                "clientName": "ANDROID_MUSIC",
                                "clientVersion": "5.16.51",
                                "androidSdkVersion": 30,
                            }
                        }
                    }"#;

                let teste: Value = serde_json::from_str(&client
                    .post(format!("https://www.youtube.com/youtubei/v1/player?videoId={v_id}&key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8&contentCheckOk=True&racyCheckOk=True"))
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(json_payload)
                    .send()
                    .await.unwrap()
                    .text()
                    .await.unwrap()).unwrap();

                let teste_clone = teste.clone();

                match teste_clone["streamingData"]["adaptiveFormats"].as_array() {
                    Some(streams) => stream_list = streams.clone(),
                    None => return Err(2),
                }
            }
        };
        let mut streams: Vec<Stream> = Vec::new();

        for i in stream_list {
            let bitrate: u32 = i["bitrate"].as_u64().unwrap() as u32;
            let content_length: u64 = i["contentLength"].as_str().unwrap_or("0").parse().unwrap();
            let itag: u16 = i["itag"].as_u64().unwrap() as u16;
            let mime_type: String = String::from(i["mimeType"].as_str().unwrap());
            let url: String = String::from(i["url"].as_str().unwrap());
            streams.push(Stream::new(
                bitrate,
                content_length,
                None,
                itag,
                mime_type,
                None,
                None,
                title.clone(),
                url,
            ));
        }
        Ok(YouTube { title, streams })
    }

    pub fn get_highest_resolution(&self, prefer_type: Option<&str>) -> Result<Stream, u8> {
        let dict: HashMap<&str, u8> = hashmap! {
            "571"=> 0,
            "402"=> 1,
            "272"=> 2,
            "138"=> 3,
            "337"=> 4,
            "401"=> 5,
            "315"=> 6,
            "305"=> 7,
            "313"=> 8,
            "266"=> 9,
            "336"=> 10,
            "400"=> 11,
            "308"=> 12,
            "304"=> 13,
            "271"=> 14,
            "264"=> 15,
            "335"=> 16,
            "399"=> 17,
            "303"=> 18,
            "301"=> 19,
            "299"=> 20,
            "248"=> 21,
            "137"=> 22,
            "96"=> 23,
            "334"=> 24,
            "398"=> 25,
            "302"=> 26,
            "300"=> 27,
            "298"=> 29,
            "247"=> 30,
            "136"=> 31,
            "95"=> 32,
            "22"=> 33,
            "333"=> 34,
            "397"=> 35,
            "244"=> 36,
            "135"=> 37,
            "94"=> 38,
            "332"=> 39,
            "396"=> 40,
            "243"=> 41,
            "134"=> 42,
            "93"=> 43,
            "18"=> 44,
            "331"=> 45,
            "395"=> 46,
            "242"=> 47,
            "133"=> 48,
            "92"=> 49,
            "330"=> 50,
            "394"=> 51,
            "278"=> 52,
            "160"=> 53,
            "91"=> 54,
            "338"=> 60,
            "327"=> 61,
            "251"=> 62,
            "140"=> 63,
            "250"=> 64,
            "249"=> 65,
            "139"=> 67
        };

        let prefer_type = prefer_type.unwrap_or("webm");
        let mut id_list: Vec<u8> = Vec::new();
        let mut streams: Vec<Stream> = Vec::new();
        for i in &self.streams {
            if i.mime_type.contains(prefer_type) {
                let id: u8 = *dict.get(format!("{}", i.itag).as_str()).unwrap_or(&255);
                id_list.push(id);
                streams.push(Stream::new(
                    i.bitrate,
                    i.content_length,
                    Some(id),
                    i.itag,
                    i.mime_type.clone(),
                    None,
                    None,
                    i.title.clone(),
                    i.url.clone(),
                ));
            }
        }

        let mut index: u8 = 0;
        let min_id: u8 = *id_list.iter().min().unwrap();
        for i in id_list {
            if i == min_id {
                break;
            }
            index += 1;
        }

        let st = &streams[index as usize];
        let mut video_stream = Stream::new(
            st.bitrate,
            st.content_length,
            Some(st.id),
            st.itag,
            st.mime_type.clone(),
            Some(st.other_content_lengh),
            Some(st.other_url.clone()),
            st.title.clone(),
            st.url.clone(),
        );

        let mut id_list: Vec<u8> = Vec::new();

        let mut audio_streams: Vec<Stream> = Vec::new();
        for i in streams {
            if i.id >= 60 {
                id_list.push(i.id);
                audio_streams.push(i);
            }
        }

        let mut index: u8 = 0;
        let min_id: u8 = *id_list.iter().min().unwrap();
        for i in id_list {
            if i == min_id {
                break;
            }
            index += 1;
        }

        let audio_stream: &Stream = &audio_streams[index as usize];

        video_stream.other_content_lengh = audio_stream.content_length;
        video_stream.other_url = audio_stream.url.clone();

        if video_stream.url == video_stream.other_url {
            return Err(1);
        }

        return Ok(video_stream);
    }
}
