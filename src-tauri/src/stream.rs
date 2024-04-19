use std::env::temp_dir;
use std::fs::{create_dir_all, read, remove_file, write, File};
use std::io::Write;
use std::process::{Command, Stdio};

use reqwest::header::RANGE;
use reqwest::Client;

pub struct Stream {
    pub url: String,
    pub mime_type: String,
    pub bitrate: u32,
    pub content_length: u64,
    pub itag: u16,
    pub id: u8,
    pub title: String,
    pub other_url: String,
    pub other_content_lengh: u64,
}

impl Stream {
    pub fn new(
        bitrate: u32,
        content_length: u64,
        id: Option<u8>,
        itag: u16,
        mime_type: String,
        other_content_lenght: Option<u64>,
        other_url: Option<String>,
        title: String,
        url: String,
    ) -> Self {
        Self {
            bitrate,
            content_length,
            id: id.unwrap_or(0),
            itag,
            mime_type,
            other_content_lengh: other_content_lenght.unwrap_or(0),
            other_url: other_url.unwrap_or(String::from("")),
            title,
            url,
        }
    }

    pub async fn download(
        &self,
        name: Option<String>,
        only_audio: Option<bool>,
        output_dir: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let prefer_type = if self.mime_type.contains("webm") {
            "webm"
        } else {
            "mp4"
        };
        let output_dir = output_dir.unwrap_or("").trim_matches('"');
        let only_audio: bool = only_audio.unwrap_or(false);
        let name = name.unwrap_or(String::from("40994f30-fdb5-4096-9992-e8d27a23d2cb"));
        let client: Client = Client::new();
        let mut init_byte: u64 = 0;
        let mut last_byte: u64 = 10485760;
        let audio_file = temp_dir().join(format!("audio.{prefer_type}"));
        let audio_file = audio_file.to_str().unwrap();
        let video_file = temp_dir().join(format!("video.{prefer_type}"));
        let video_file = video_file.to_str().unwrap();

        if !only_audio {
            let packets: f64 = self.content_length as f64 / 10485760.0;
            let packets: u64 = packets.ceil() as u64;

            for _ in 0..=packets - 1 {
                let content = client
                    .get(&self.url)
                    .header(RANGE, format!("bytes={init_byte}-{last_byte}"))
                    .send()
                    .await?
                    .bytes()
                    .await?;

                let name = format!("video-{}.{}", last_byte / 10485760, prefer_type);

                let name_out = temp_dir().join(name);

                match write(&name_out.to_str().unwrap(), content) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:#?} download failed! {}", name_out, e);
                        init_byte += if init_byte == 0 { 10485761 } else { 10485760 };
                        last_byte += 10485760;
                        continue;
                    }
                }

                init_byte += if init_byte == 0 { 10485761 } else { 10485760 };
                last_byte += 10485760;
            }

            let mut counter: u64 = 1;

            let mut file = File::create(video_file)?;

            loop {
                match read(temp_dir().join(format!("video-{counter}.{prefer_type}"))) {
                    Err(_) => break,
                    Ok(f) => {
                        _ = file.write_all(&f);
                        _ = remove_file(temp_dir().join(format!("video-{counter}.{prefer_type}")));
                        counter += 1;
                    }
                }
            }
        }
        init_byte = 0;
        last_byte = 10485760;

        let packets: f64 = self.other_content_lengh as f64 / 10485760.0;
        let packets: u64 = packets.ceil() as u64;

        for _ in 0..=packets - 1 {
            let content = client
                .get(&self.other_url)
                .header(RANGE, format!("bytes={init_byte}-{last_byte}"))
                .send()
                .await?
                .bytes()
                .await?;

            let name = format!("audio-{}.{}", last_byte / 10485760, prefer_type);

            let name_out = temp_dir().join(name);

            match write(&name_out, content) {
                Ok(_) => {}
                Err(e) => {
                    println!("{:#?} download failed! {}", name_out, e);
                    init_byte += if init_byte == 0 { 10485761 } else { 10485760 };
                    last_byte += 10485760;
                    continue;
                }
            }

            init_byte += if init_byte == 0 { 10485761 } else { 10485760 };
            last_byte += 10485760;
        }

        let mut counter: u64 = 1;

        let mut file = File::create(audio_file)?;

        loop {
            match read(temp_dir().join(format!("audio-{counter}.{prefer_type}"))) {
                Err(_) => break,
                Ok(f) => {
                    _ = file.write_all(&f);
                    _ = remove_file(temp_dir().join(format!("audio-{counter}.{prefer_type}")));
                    counter += 1;
                }
            }
        }

        let mut cmd: Command = Command::new("ffmpeg");

        let title = if name == "40994f30-fdb5-4096-9992-e8d27a23d2cb" {
            self.clean_filename(&self.title)
        } else {
            self.clean_filename(&name)
        };

        let out_file: String;

        if output_dir == "" {
            out_file = format!("{title}.{prefer_type}");
        } else {
            create_dir_all(output_dir)?;
            out_file = format!("{output_dir}/{title}.{prefer_type}");
        }

        cmd.args([
            "-i", video_file, "-i", audio_file, "-c", "copy", "-map", "0:v:0", "-map", "1:a:0",
            &out_file,
        ]);

        let output = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Video processing failed on ffmpeg command");

        assert!(
            output.success(),
            "Video processing failed on ffmpeg command"
        );

        remove_file(video_file)?;
        remove_file(audio_file)?;

        Ok(())
    }

    fn clean_filename(&self, filename: &str) -> String {
        let illegal_chars = vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if illegal_chars.iter().any(|c| filename.contains(*c)) {
            let clean_name = filename
                .chars()
                .map(|c| if illegal_chars.contains(&c) { '_' } else { c })
                .collect();
            return clean_name;
        }
        return filename.to_string();
    }
}
