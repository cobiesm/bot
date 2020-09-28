use crate::module::{
    badword, blacklink, clap, faq, presence, selfmod, slowmode, startup_message, undelete,
};
use serenity::client::Context;
use serenity::model::channel::{Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, MessageId};
use serenity::prelude::EventHandler;
use serenity::{async_trait, model::event::MessageUpdateEvent};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        blacklink::message(&ctx, &new_message).await;
        badword::message(&ctx, &new_message).await;
        slowmode::message(&ctx, &new_message).await;
        faq::message(&ctx, &new_message).await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        selfmod::reaction_add(&ctx, &reaction).await;
        clap::reaction_add(&ctx, &reaction).await;
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        presence::ready(&ctx).await;
        startup_message::ready(&ctx).await;
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        undelete::message_delete(&ctx, channel_id, message_id).await;
    }

    async fn message_update(
        &self,
        ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        undelete::message_update(&ctx, old, new, event).await;
    }
}
