#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

#[macro_use]
extern crate lazy_static;

mod group;
use group::{ACE_GROUP, ADMIN_GROUP, EVERYONE_GROUP, FUN_GROUP};
mod handler;
use handler::Handler;
mod module;
use module::help::HELP;
mod muteable;

use nicknamedb::SerenityInit;
use serenity::client::{bridge::gateway::GatewayIntents, Client};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::DispatchError;
use serenity::framework::StandardFramework;
use serenity::model::channel::{Message, ReactionType};
use serenity::prelude::Context;
use std::env;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    let mut client = Client::builder(&env::var("ROBOTOKEN").expect("token"))
        .event_handler(Handler)
        .intents(GatewayIntents::all())
        .register_nicknamedb('^')
        .framework(
            StandardFramework::new()
                .configure(|c| c.prefix(".").allow_dm(false))
                .bucket("addemoji", |buc| buc.delay(43200).check(bucket_check))
                .await
                .bucket("fun", |buc| buc.delay(10))
                .await
                .after(after_hook)
                .help(&HELP)
                .group(&ADMIN_GROUP)
                .group(&FUN_GROUP)
                .group(&ACE_GROUP)
                .group(&EVERYONE_GROUP)
                .on_dispatch_error(dispatch_error_hook),
        )
        .await
        .expect("Girilen token, token değil.");

    client.cache_and_http.cache.set_max_messages(1000).await;

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        let mut signal = signal(SignalKind::terminate()).unwrap();
        loop {
            signal.recv().await.unwrap();
            println!("\nHele bi soluklanayım.");
            shard_manager.lock().await.shutdown_all().await;
            std::process::exit(0);
        }
    });

    if let Err(err) = client.start().await {
        println!("Başlangıç sırasında bir hata ile karşılaşıldı: {:?}", err);
    }
}

#[hook]
async fn after_hook(ctx: &Context, msg: &Message, _: &str, error: Result<(), CommandError>) {
    if let Err(error) = error {
        let member = msg.member(ctx).await.unwrap();
        msg.channel_id
            .send_message(ctx, |b| b.content(format!("{}, *{}*", member, error)))
            .await
            .ok();
    } else {
        react_ok(ctx, msg).await;
    }
}

#[hook]
async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::Ratelimited(info) if info.is_first_try => {
            msg.reply(ctx, format!("{} saniye beklemen lazım", info.as_secs()))
                .await
                .ok();
        }
        _ => eprintln!("disp_err: {:?}", error),
    }
}

#[hook]
async fn bucket_check(_: &Context, msg: &Message) -> bool {
    !msg.member
        .as_ref()
        .unwrap()
        .roles
        .contains(&589415787668701185.into())
}

async fn react_ok(ctx: &Context, msg: &Message) {
    msg.react(ctx, ReactionType::Unicode("🤘".into()))
        .await
        .ok();
}
