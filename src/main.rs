#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::cast_precision_loss)]

#[macro_use]
extern crate lazy_static;

mod group;
use group::{ ADMIN_GROUP, FUN_GROUP, ACE_GROUP };
mod handler;
use handler::Handler;
mod module;
use module::help::HELP;

use std::env;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::DispatchError;
use serenity::framework::standard::macros::hook;
use serenity::model::channel::{ Message, ReactionType };
use serenity::prelude::Context;

#[tokio::main]
async fn main() {
    let mut client = Client::new(
        &env::var("ROBOTOKEN").expect("token")
    ).event_handler(Handler).framework(
        StandardFramework::new().configure(|c| c.prefix(".").allow_dm(false))
        .bucket("addemoji", |buc| buc.delay(43200)).await
        .bucket("fun", |buc| buc.delay(10)).await
        .after(after_hook).help(&HELP).group(&ADMIN_GROUP).group(&FUN_GROUP)
        .group(&ACE_GROUP).on_dispatch_error(dispatch_error_hook)
    ).await.expect("Girilen token, token değil.");

    client.cache_and_http.cache.set_max_messages(1000).await;
    if let Err(err) = client.start().await {
        println!("Başlangıç sırasında bir hata ile karşılaşıldı: {:?}", err);
    }
}

#[hook]
async fn after_hook(ctx: &Context, msg: &Message, _: &str, error: Result<(), CommandError>) {
    if let Err(e) = error {
        msg.reply(ctx, e).await.ok();
    } else {
        react_ok(ctx, msg).await;
    }
}

#[hook]
async fn dispatch_error_hook(_: &Context, _: &Message, error: DispatchError) {
    println!("{:?}", error);
}

async fn react_ok(ctx: &Context, msg: &Message) {
    msg.react(ctx, ReactionType::Unicode("🤘".into())).await.ok();
}
