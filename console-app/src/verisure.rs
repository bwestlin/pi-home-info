use std::collections::{BTreeMap, HashMap};

use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;

use crate::BoxError;

// Note that they seem to be switching between https://m-api01.verisure.com/ and https://m-api02.verisure.com/
// where only one of them works, som might need to cycle between them if some specific error occurs

#[derive(Debug)]
pub struct VerisureData {
    pub climate: BTreeMap<String, f64>,
}

pub async fn fetch_verisure_data() -> Result<Option<VerisureData>, BoxError> {
    let username =
        std::env::var("VERISURE_USER").map_err(|_| format!("No VERISURE_USER env var set"))?;
    let password = std::env::var("VERISURE_PASSWORD")
        .map_err(|_| format!("No VERISURE_PASSWORD env var set"))?;

    let client = reqwest::Client::builder().cookie_store(true).build()?;

    login(&client, &username, &password).await?;

    let giid = get_giid(&client, &username).await?;

    let climate = get_climate(&client, &giid).await?.into_iter().collect();

    Ok(Some(VerisureData { climate }))
}

async fn login(client: &Client, username: &str, password: &str) -> Result<(), BoxError> {
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

    let body = resp.bytes().await?;

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

async fn get_giid(client: &Client, username: &str) -> Result<String, BoxError> {
    let body = format!("
      [
        {{
            \"operationName\": \"AccountInstallations\",
            \"variables\": {{
              \"email\": \"{username}\"
            }},
            \"query\": \"query AccountInstallations($email: String!) {{\n  account(email: $email) {{\n    owainstallations {{\n      giid\n      alias\n      type\n      subsidiary\n      dealerId\n      installationOwner\n      subtype\n      __typename\n    }}\n    __typename\n  }}\n}}\n\"
          }}
      ]
    ");

    let req = client
        .post("https://m-api01.verisure.com/graphql")
        .header(http::header::USER_AGENT, "curl/7.81.0")
        .header(http::header::ACCEPT, mime::APPLICATION_JSON.to_string())
        .header(http::header::CONTENT_LENGTH, format!("{}", body.len()))
        .body(body);

    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        dbg!(text);
        return Err(format!("Got status: {}", status).into());
    }

    let body = resp.bytes().await?;

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

    Ok(resp["data"]["account"].owainstallations[0].giid.clone())
}

async fn get_climate(client: &Client, giid: &str) -> Result<Vec<(String, f64)>, BoxError> {
    let body = format!("
      [
        {{
            \"operationName\": \"Climate\",
            \"variables\": {{
              \"giid\": \"{giid}\"
            }},
            \"query\": \"query Climate($giid: String!) {{\\n  installation(giid: $giid) {{\\n    climates {{\\n      device {{\\n        deviceLabel\\n        area\\n        gui {{\\n          label\\n          support\\n          __typename\\n        }}\\n        __typename\\n      }}\\n      humidityEnabled\\n      humidityTimestamp\\n      humidityValue\\n      temperatureTimestamp\\n      temperatureValue\\n      supportsThresholdSettings\\n      thresholds {{\\n        aboveMaxAlert\\n        belowMinAlert\\n        sensorType\\n        __typename\\n      }}\\n      __typename\\n    }}\\n    __typename\\n  }}\\n}}\\n\"
          }}
      ]
    ");

    let req = client
        .post("https://m-api01.verisure.com/graphql")
        .header(http::header::USER_AGENT, "curl/7.81.0")
        .header(http::header::ACCEPT, mime::APPLICATION_JSON.to_string())
        .header(http::header::CONTENT_LENGTH, format!("{}", body.len()))
        .body(body);

    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        dbg!(text);
        return Err(format!("Got status: {}", status).into());
    }

    let body = resp.bytes().await?;

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

    let mut ret = vec![];

    for climate in &resp["data"]["installation"].climates {
        ret.push((climate.device.area.clone(), climate.temperature_value));
    }

    Ok(ret)
}
