use chrono::prelude::*;
use serenity::client::Context;
use serenity::utils::Colour;

pub async fn ready(ctx: &Context) {
    let dm = ctx
        .http
        .get_user(124226104931254276)
        .await
        .expect("ADMIN is missing!")
        .create_dm_channel(&ctx)
        .await
        .unwrap();
    #[deny(clippy::unreadable_literal)]
    dm.send_message(&ctx, |m| {
        m.embed(|e| {
            e.colour(Colour::from_rgb(128, 237, 153))
                .timestamp(&Utc::now())
                .footer(|f| f.text("Heroku'nun amına koyim bu downtime ne ya!?"))
        })
    })
    .await
    .ok();
}
