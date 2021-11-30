use super::exchange::fetch_rate;
use chrono::Duration;
use serenity::client::Context;
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use tokio::time::sleep;

pub async fn ready(ctx: &Context) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        loop {
            let rate = fetch_rate().await;
            if let Ok(rate) = rate {
                ctx.set_presence(
                    Some(Activity::watching(format!("{} :(", rate))),
                    OnlineStatus::DoNotDisturb,
                )
                .await;
            }
            sleep(Duration::minutes(20).to_std().expect("duration > zero")).await;
        }
    });
}
