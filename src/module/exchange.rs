use std::error::Error;

use regex::Regex;

pub async fn fetch_rate() -> Result<f64, Box<dyn Error + Send + Sync>> {
    let r_client = reqwest::Client::builder().use_rustls_tls().build().unwrap();
    let url = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml";
    let xml = r_client.get(url).send().await?.text().await?;
    Ok(format!("{:.2}", rate_of("TRY", &xml)? / rate_of("USD", &xml)?).parse::<f64>()?)
}

fn rate_of(currency: &str, xml: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
    Ok(Regex::new(r"<Cube currency='(.*)' rate='(.*)'/>")?
        .captures_iter(xml)
        .map(|cap| {
            let currency = cap[1].to_string();
            let rate = cap[2].to_string();
            (currency, rate)
        })
        .find(|(cr, _)| cr == currency).ok_or(format!("Can't find the rate of {}", currency))?
        .1
        .parse::<f64>()?)
}
