use crate::module::{badword, blacklink, clap, faq, level, presence, selfmod, slowmode, undelete};
use serenity::model::channel::{Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, MessageId};
use serenity::prelude::EventHandler;
use serenity::{async_trait, model::event::MessageUpdateEvent};
use serenity::{client::Context, model::id::GuildId};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.is_own(&ctx).await || new_message.is_private() {
            return;
        }

        faq::message(&ctx, &new_message).await;

        let member = match new_message.member(&ctx).await {
            Ok(member) => member,
            Err(e) => {
                eprintln!("Couldn't get the member because {}.", e);
                return;
            }
        };

        if !member
            .permissions(&ctx)
            .await
            .expect("permissions for new message's member in cache")
            .administrator()
        {
            blacklink::message(&ctx, &new_message).await;
            badword::message(&ctx, &new_message).await;
            slowmode::message(&ctx, &new_message).await;
            level::message(&ctx, &new_message).await;
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if reaction.guild_id.is_none() || reaction.user(&ctx).await.expect("user").bot {
            return;
        }

        selfmod::reaction_add(&ctx, &reaction).await;
        clap::reaction_add(&ctx, &reaction).await;
        level::reaction_add(&ctx, &reaction).await;
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        if reaction.guild_id.is_none() || reaction.user(&ctx).await.expect("user").bot {
            return;
        }

        level::reaction_remove(&ctx, &reaction).await;
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        presence::ready(&ctx).await;
        level::ready(&ctx).await;
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        let message = if let Some(message) = ctx.cache.message(channel_id, message_id).await {
            message
        } else {
            eprintln!("Could not find the message in cache.");
            return;
        };

        if message.is_private() || message.is_own(&ctx).await {
            return;
        }

        undelete::message_delete(&ctx, channel_id, message.clone()).await;
        level::message_delete(&ctx, channel_id, message.clone()).await;
    }

    async fn message_update(
        &self,
        ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(message) = &new {
            if message.is_private() || message.is_own(&ctx).await {
                return;
            }
        }

        undelete::message_update(&ctx, old.clone(), new.clone(), event.clone()).await;
        level::message_update(&ctx, old.clone(), new.clone(), event.clone()).await;
    }
}
