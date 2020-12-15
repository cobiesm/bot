use std::sync::Arc;

use async_trait::async_trait;
use chrono::prelude::*;
use regex::Regex;
use serenity::{
    client::Context,
    http::Http,
    model::{
        channel::{Message, Reaction, ReactionType},
        event::MessageUpdateEvent,
        guild::Member,
        id::{ChannelId, MessageId},
    },
};

use super::undelete::is_deleted;

lazy_static! {
    static ref LEVEL_FINDER: Regex = Regex::new(r"\^(\d+\.\d+)$").unwrap();
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = reaction.message(ctx).await;
    if message.is_err() {
        return;
    }

    let message = message.unwrap();

    if reaction.emoji != *super::clap::CLAP {
        return;
    } else if reaction.user_id.unwrap() == message.author.id {
        reaction.delete(ctx).await.ok();
        return;
    }

    if let Some(mut member) = message.member(ctx).await {
        member.xp_give(ctx.http.clone(), 0.35).await;
    }
}

pub async fn message(ctx: &Context, msg: &Message) {
    if let Some(mut member) = msg.member(ctx).await {
        member.xp_give(ctx.http.clone(), 0.1).await;
    }
}

pub async fn message_delete(ctx: &Context, _channel_id: ChannelId, message: Message) {
    if let Some(mut member) = message.member(ctx).await {
        member.xp_take(ctx.http.clone(), 1.5).await;
    }
}

pub async fn message_update(
    ctx: &Context,
    old: Option<Message>,
    new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(old) = old {
        if let Some(new) = new {
            if is_deleted(&old, &new) {
                if let Some(mut member) = new.member(ctx).await {
                    member.xp_take(ctx.http.clone(), 1.0).await;
                }
            }
        }
    }
}

#[async_trait]
pub trait Level {
    async fn xp_give(&mut self, http: Arc<Http>, amount: f64);
    async fn xp_take(&mut self, http: Arc<Http>, amount: f64);
}

#[async_trait]
impl Level for Member {
    async fn xp_give(&mut self, http: Arc<Http>, amount: f64) {
        let mut name = String::new();
        name.push_str(self.display_name().as_str());

        if !LEVEL_FINDER.is_match(name.as_str()) {
            name.push_str(" ^0.0");
        }

        let xp_current = LEVEL_FINDER
            .captures(name.as_str())
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse::<f64>()
            .unwrap();
        let mut xp_new = (xp_current + amount).max(0.0);

        let mut last_millis = 0;

        for val in self.guild_id.channels(&http).await.unwrap().values() {
            if let Ok(mes) = val.messages(&http, |buil| buil.limit(1)).await {
                if let Some(mes) = mes.first() {
                    let millis = mes.timestamp.timestamp_millis();
                    if millis.max(last_millis) == millis {
                        last_millis = mes.timestamp.timestamp_millis();
                    }
                }
            };
        }

        let time_diff = Local::now().timestamp_millis() - last_millis;
        let half_day = 43200000;

        if time_diff > half_day {
            xp_new = (xp_new - time_diff as f64 / half_day as f64).max(0.0);
        }

        let name = LEVEL_FINDER.replace(name.as_str(), format!("^{:.2}", xp_new).as_str());

        self.edit(http, |e| e.nickname(name)).await.ok();
    }

    async fn xp_take(&mut self, http: Arc<Http>, amount: f64) {
        self.xp_give(http, -amount).await;
    }
}
