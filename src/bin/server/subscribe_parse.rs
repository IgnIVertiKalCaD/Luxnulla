use base64::{Engine as _, engine::general_purpose};
use std::error::Error;

pub async fn fetch_and_parse_configs(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let body = response.text().await?;
    let body = body.trim();

    let content = match general_purpose::STANDARD.decode(body) {
        Ok(decoded_bytes) => {
            println!("INFO: Content detected as Base64. Decoding...");
            String::from_utf8(decoded_bytes)?
        }
        Err(_) => {
            println!("INFO: Content detected as plain text.");
            body.to_string()
        }
    };

    // .lines()
    // .map(|s| s.trim())
    // .filter(|s| !s.is_empty() && s.starts_with("vless://"))
    // .map(String::from)
    // .collect();

    Ok(content)
}

// #[tokio::main]
// async fn main() {
//     println!("\n---------------------------------------------\n");

//     let mock_base664_url =
//         "https://raw.githubusercontent.com/barry-far/V2ray-Configs/main/All_Configs_base64.txt";

//     println!("--- Fetching from Base64 URL: {} ---", mock_base664_url);
//     match fetch_and_parse_configs(mock_base664_url).await {
//         Ok(configs) => {
//             println!("Successfully parsed {} configs.", configs.len());
//             for config in configs.iter().take(2) {
//                 println!("- {}", config);
//             }
//         }
//         Err(e) => {
//             eprintln!("Error: {}", e);
//         }
//     }
// }
