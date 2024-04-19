// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::youtube::YouTube;
use serde::Serialize;
use tauri::{generate_handler, AppHandle, Manager};

#[cfg(target_os = "linux")]
fn webkit_hidpi_workaround() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

fn main_prelude() {
    #[cfg(target_os = "linux")]
    webkit_hidpi_workaround();
}

fn main() {
    main_prelude();
    tauri::Builder::default()
        .invoke_handler(generate_handler![download_handler])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn download_handler(app: AppHandle, url: &str, output: &str) -> Result<String, u8> {
    let url = String::from(url);
    let output = String::from(output);
    tauri::async_runtime::spawn(async move { downloa_teste(app, &url[..], &output[..]).await });
    Ok(String::from("TESTANDO 123"))
}

#[derive(Clone, Serialize)]
struct DownloadStruct {
    message: String,
}

async fn downloa_teste(app: AppHandle, url: &str, output: &str) -> Result<String, u8> {
    match YouTube::new(url).await {
        Ok(video) => match video.get_highest_resolution(None) {
            Ok(stream) => {
                stream.download(None, None, Some(output)).await.unwrap();
                app.emit_to(
                    "main",
                    "download_completed",
                    DownloadStruct {
                        message: format!("{} downloaded sucefull", video.title),
                    },
                )
                .unwrap();
                return Ok(video.title);
            }
            Err(error_code) => return Err(error_code),
        },
        Err(error_code) => return Err(error_code),
    }
}
