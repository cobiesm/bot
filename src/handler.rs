use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::{ Message, Reaction };
use serenity::model::id::{ ChannelId, MessageId };
use serenity::model::gateway::Ready;
use serenity::prelude::EventHandler;
use crate::module::{ badword, blacklink, presence, selfmod, slowmode, startup_message,
                     faq, undelete };

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
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        presence::ready(&ctx).await;
        startup_message::ready(&ctx).await;
    }

    async fn message_delete(&self, ctx: Context,
                      channel_id: ChannelId, message_id: MessageId)
    {
        undelete::message_delete(&ctx, channel_id, message_id).await;
        undelete::message_delete(&ctx, channel_id, message_id).await;
    }
}
