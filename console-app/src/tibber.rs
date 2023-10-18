use chrono::{DateTime, Timelike, Utc};
use cynic::QueryBuilder;

use crate::BoxError;

// Pull in the tibber schema we registered in build.rs
#[cynic::schema("tibber")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct MyQuery {
    pub viewer: Viewer,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Viewer {
    pub login: Option<String>,
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub websocket_subscription_url: Option<String>,
    pub homes: Vec<Option<Home>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Home {
    pub current_subscription: Option<Subscription>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Subscription {
    pub price_info: Option<PriceInfo>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PriceInfo {
    pub today: Vec<Option<Price>>,
    pub tomorrow: Vec<Option<Price>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Price {
    pub currency: String,
    pub energy: Option<f64>,
    pub level: Option<PriceLevel>,
    pub starts_at: Option<String>,
    pub tax: Option<f64>,
    pub total: Option<f64>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum PriceLevel {
    Normal,
    Cheap,
    VeryCheap,
    Expensive,
    VeryExpensive,
}

#[derive(Debug)]
pub struct TibberData {
    pub hourly_prices: Vec<HourlyPrice>,
}

#[derive(Debug)]
pub struct HourlyPrice {
    pub total: f64,
    pub starts_at: DateTime<Utc>,
}

pub async fn fetch_tibber_data() -> Result<Option<TibberData>, BoxError> {
    use cynic::http::SurfExt;
    let token = std::env::var("TIBBER").map_err(|_| format!("No TIBBER env var set"))?;

    let operation = MyQuery::build(());

    let result = surf::post("https://api.tibber.com/v1-beta/gql")
        .header("Authorization", format!("Bearer {token}"))
        .run_graphql(operation)
        .await?;

    let tibber_data = result.data.map(|data| {
        let mut hourly_prices = vec![];
        let start = Utc::now()
            .with_minute(0)
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap();
        println!("start={start:?}");

        for home in data.viewer.homes.iter().flatten() {
            if let Some(sub) = &home.current_subscription {
                if let Some(price_info) = &sub.price_info {
                    for price in price_info
                        .today
                        .iter()
                        .chain(price_info.tomorrow.iter())
                        .flatten()
                    {
                        let starts_at = price.starts_at.as_ref().map(|starts_at| {
                            DateTime::parse_from_rfc3339(&starts_at.as_str())
                                .map(DateTime::<Utc>::from)
                        });

                        if let (&Some(total), Some(Ok(starts_at))) = (&price.total, starts_at) {
                            if starts_at >= start {
                                hourly_prices.push(HourlyPrice { total, starts_at })
                            }
                        }
                    }
                }
            }
        }

        hourly_prices.sort_by(|a, b| a.starts_at.cmp(&b.starts_at));

        TibberData { hourly_prices }
    });

    Ok(tibber_data)
}
