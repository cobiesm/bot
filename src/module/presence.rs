use chrono::prelude::*;
use serenity::client::Context;
use serenity::model::user::OnlineStatus;
use serenity::utils::Colour;
use serenity::model::gateway::Activity;

pub fn ready(ctx: &Context) {
    ctx.set_presence(
        Some(Activity::listening(".help | loli hunter")),
        OnlineStatus::DoNotDisturb
    );
}
