#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::cast_precision_loss)]

#[macro_use]
extern crate lazy_static;

mod group;
use group::*;
mod handler;
use handler::Handler;
mod module;
use module::help::*;

use serenity::client::Client;
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::env;

fn main() {
    let mut client = Client::new(
        &env::var("ROBOTOKEN").expect("token"), Handler
    ).expect("Girilen token, token değil.");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("."))
        .after(|ctx, msg, _, res| {
            match res {
                Ok(()) => { react_ok(ctx, msg); }, // react to ok
                Err(err) => { msg.reply(ctx, err.0).ok(); } // CommandErrors as reply
            }
        })
        .on_dispatch_error(|ctx, msg, err| {
            msg.reply(ctx, format!("{:?}", err)).ok();
        })
        .help(&HELP)
        .group(&ADMIN_GROUP));

    if let Err(err) = client.start() {
        println!("Başlangıç sırasında bir hata ile karşılaşıldı: {:?}", err);
    }
}

fn react_ok(ctx: &Context, msg: &Message) {
    match msg.react(ctx, "🤘") {
        Ok(_) => (),
        Err(err) => {
            msg.reply(ctx, err.to_string()).ok();
        }
    };
}
