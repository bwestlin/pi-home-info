use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let username =
        std::env::var("VERISURE_USER").map_err(|_| format!("No VERISURE_USER env var set"))?;
    let password = std::env::var("VERISURE_PASSWORD")
        .map_err(|_| format!("No VERISURE_PASSWORD env var set"))?;

    let client = reqwest::Client::builder().cookie_store(true).build()?;

    login(&client, &username, &password).await?;

    let giid = get_giid(&client).await?;
    dbg!(&giid);

    let climate = get_climate(&client).await?;
    dbg!(climate);

    Ok(())
}

async fn login(client: &Client, username: &str, password: &str) -> Result<(), BoxError> {
    println!("Logging in.\n");
    let req = client
        .post("https://m-api01.verisure.com/auth/login")
        .body("")
        .header(http::header::USER_AGENT, "curl/7.81.0")
        .header(
            http::header::AUTHORIZATION,
            format!(
                "Basic {}",
                general_purpose::STANDARD_NO_PAD.encode(format!("{username}:{password}"))
            ),
        )
        .header(http::header::ACCEPT, mime::APPLICATION_JSON.to_string())
        .header(http::header::CONTENT_LENGTH, "0");

    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        dbg!(text);
        return Err(format!("Got status: {}", status).into());
    }

    println!("Got status: {status}\n");

    let body = resp.bytes().await?;
    println!("Got response:\n{}\n", String::from_utf8_lossy(&body));

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct LoginResponse {
        access_token: String,
        access_token_max_age_seconds: u32,
        refresh_token: String,
        refresh_token_max_age_seconds: u32,
    }

    let _resp = serde_json::from_slice::<LoginResponse>(&body)?;
    Ok(())
}

async fn get_giid(client: &Client) -> Result<String, BoxError> {
    println!("Getting giid.\n");

    let body = "
      [
        {
            \"operationName\": \"AccountInstallations\",
            \"variables\": {
              \"email\": \"bjorn@wedako.se\"
            },
            \"query\": \"query AccountInstallations($email: String!) {\n  account(email: $email) {\n    owainstallations {\n      giid\n      alias\n      type\n      subsidiary\n      dealerId\n      installationOwner\n      subtype\n      __typename\n    }\n    __typename\n  }\n}\n\"
          }
      ]
    ";

    let req = client
        .post("https://m-api01.verisure.com/graphql")
        .body(body)
        .header(http::header::USER_AGENT, "curl/7.81.0")
        .header(http::header::ACCEPT, mime::APPLICATION_JSON.to_string())
        .header(http::header::CONTENT_LENGTH, format!("{}", body.len()));

    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        dbg!(text);
        return Err(format!("Got status: {}", status).into());
    }

    println!("Got status: {status}\n");

    let body = resp.bytes().await?;
    println!("Got response:\n{}\n", String::from_utf8_lossy(&body));

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct Account {
        owainstallations: Vec<OWAInstallation>,
    }

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct OWAInstallation {
        giid: String,
        alias: String,
        r#type: String,
        subsidiary: Option<String>,
        dealer_id: Option<String>,
    }

    let resp = serde_json::from_slice::<HashMap<String, HashMap<String, Account>>>(&body)?;
    // let resp = resp.text().await?;

    // println!("{:#?}", resp);

    Ok(resp["data"]["account"].owainstallations[0].giid.clone())
}

async fn get_climate(client: &Client) -> Result<Vec<(String, f64)>, BoxError> {
    println!("Getting climate.\n");

    let body = "
      [
        {
            \"operationName\": \"Climate\",
            \"variables\": {
              \"giid\": \"112832675891\"
            },
            \"query\": \"query Climate($giid: String!) {\\n  installation(giid: $giid) {\\n    climates {\\n      device {\\n        deviceLabel\\n        area\\n        gui {\\n          label\\n          support\\n          __typename\\n        }\\n        __typename\\n      }\\n      humidityEnabled\\n      humidityTimestamp\\n      humidityValue\\n      temperatureTimestamp\\n      temperatureValue\\n      supportsThresholdSettings\\n      thresholds {\\n        aboveMaxAlert\\n        belowMinAlert\\n        sensorType\\n        __typename\\n      }\\n      __typename\\n    }\\n    __typename\\n  }\\n}\\n\"
          }
      ]
    ";

    let req = client
        .post("https://m-api01.verisure.com/graphql")
        .body(body)
        .header(http::header::USER_AGENT, "curl/7.81.0")
        .header(http::header::ACCEPT, mime::APPLICATION_JSON.to_string())
        .header(http::header::CONTENT_LENGTH, format!("{}", body.len()));

    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        dbg!(text);
        return Err(format!("Got status: {}", status).into());
    }

    println!("Got status: {status}\n");

    let body = resp.bytes().await?;
    println!("Got response:\n{}\n", String::from_utf8_lossy(&body));

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct Installation {
        climates: Vec<Climate>,
    }

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct Climate {
        device: Device,
        humidity_enabled: Option<bool>,
        humidity_timestamp: String,
        humidity_value: f64,
        temperature_timestamp: String,
        temperature_value: f64,
        supports_threshold_settings: bool,
        // TODO thresholds
    }

    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct Device {
        device_label: String,
        area: String,
        // TODO gui
    }

    let resp = serde_json::from_slice::<HashMap<String, HashMap<String, Installation>>>(&body)?;
    // let resp = resp.text().await?;

    // println!("{:#?}", resp);

    let mut ret = vec![];

    for climate in &resp["data"]["installation"].climates {
        ret.push((climate.device.area.clone(), climate.temperature_value));
    }

    Ok(ret)
}
