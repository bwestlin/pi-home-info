use std::error::Error;

use chrono::{DateTime, Local, Timelike, Utc};
use inline_colorization::*;

use crate::{
    tibber::{HourlyPrice, TibberData},
    verisure::VerisureData,
};

mod tibber;
mod verisure;

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60 * 5));

    loop {
        interval.tick().await;

        let now = Utc::now();
        println!();
        println!("now={}", now);
        println!();

        display().await?;

        println!();
    }
}

async fn display() -> Result<(), Box<dyn Error>> {
    let tibber_data = tibber::fetch_tibber_data().await?;
    let verisure_data = verisure::fetch_verisure_data().await?;

    if let Some(TibberData { hourly_prices }) = tibber_data {
        println!("Elpriser:");

        let prices = hourly_prices
            .iter()
            .map(|HourlyPrice { total, .. }| *total)
            .collect::<Vec<_>>();

        let min_price = prices
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .cloned()
            .unwrap_or_default();
        let max_price = prices
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .cloned()
            .unwrap_or_default();
        let max_price = if max_price > 1.5 { 1.5 } else { max_price };
        let delta = max_price - min_price;
        let d3 = delta / 3.0;

        for HourlyPrice { starts_at, .. } in hourly_prices {
            let dt = DateTime::<Local>::from(starts_at);

            print!(" {:02}:00", dt.hour());
        }
        println!();

        for price in prices {
            let color = if price > min_price + d3 + d3 {
                color_red
            } else if price > min_price + d3 {
                color_yellow
            } else {
                color_green
            };
            print!("{color}{:>6.1}{color_reset}", price * 100.0);
        }
        println!();
    }

    if let Some(VerisureData { climate }) = verisure_data {
        println!();
        println!("Temperaturer:");

        for (name, temp) in climate {
            print!("{name}: {temp} grader ");
        }
        println!();
    }

    Ok(())
}
