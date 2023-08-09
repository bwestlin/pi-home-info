use cynic::QueryBuilder;
use std::error::Error;

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
    pub subscriptions: Vec<Option<Subscription>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Subscription {
    pub price_info: Option<PriceInfo>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PriceInfo {
    pub today: Vec<Option<Price>>,
    pub tomorrow: Vec<Option<Price2>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Price")]
pub struct Price2 {
    pub currency: String,
    pub energy: Option<f64>,
    pub level: Option<PriceLevel>,
    pub tax: Option<f64>,
    pub starts_at: Option<String>,
    pub total: Option<f64>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use cynic::http::SurfExt;

    let token = std::env::var("TIBBER").map_err(|_| format!("No TIBBER env var set"))?;

    let operation = MyQuery::build(());

    let result = surf::post("https://api.tibber.com/v1-beta/gql")
        .header("Authorization", format!("Bearer {token}"))
        .run_graphql(operation)
        .await?;

    dbg!(result);

    Ok(())
}
