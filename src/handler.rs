use crate::module::{badword, blacklink, clap, faq, level, presence, selfmod, slowmode, undelete};
use serenity::model::channel::{Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, MessageId};
use serenity::prelude::EventHandler;
use serenity::{async_trait, model::event::MessageUpdateEvent};
use serenity::{client::Context, model::id::GuildId};
use tokio::join;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.is_own(&ctx).await
            || new_message.is_private()
            || new_message.webhook_id.is_some()
        {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Message received \"{}\".", new_message.content);

        faq::message(&ctx, &new_message).await;

        let mut i = 1;
        let member = loop {
            match new_message.member(&ctx).await {
                Ok(member) => break member,
                Err(e) => {
                    if i == 10 {
                        panic!("Couldn't get the member because {}.", e);
                    } else {
                        println!("Trying to get member {}", i);
                        i += 1;
                    }
                }
            };
        };

        let perms = member.permissions(&ctx).await;

        if let Ok(perms) = perms {
            if perms.administrator() {
                return;
            }
        } else if let Err(err) = perms {
            eprintln!("Can't fetch permissions because {}.", err);
        }

        join!(
            blacklink::message(&ctx, &new_message),
            badword::message(&ctx, &new_message),
            slowmode::message(&ctx, &new_message),
            level::message(&ctx, &new_message)
        );
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if reaction.guild_id.is_none() || reaction.user(&ctx).await.expect("user").bot {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Reaction received \"{}\".", reaction.emoji);

        join!(
            selfmod::reaction_add(&ctx, &reaction),
            clap::reaction_add(&ctx, &reaction),
            level::reaction_add(&ctx, &reaction)
        );
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        if reaction.guild_id.is_none() || reaction.user(&ctx).await.expect("user").bot {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Reaction removed \"{}\".", reaction.emoji);

        level::reaction_remove(&ctx, &reaction).await;
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        #[cfg(debug_assertions)]
        println!("Ready");

        join!(presence::ready(&ctx), level::ready(&ctx));
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

        #[cfg(debug_assertions)]
        println!("Message deleted \"{}\".", message.content);

        join!(
            undelete::message_delete(&ctx, channel_id, message.clone()),
            level::message_delete(&ctx, channel_id, message.clone())
        );
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
        } else {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Message updated \"{}\".", new.clone().unwrap().content);

        join!(
            undelete::message_update(&ctx, old.clone(), new.clone(), event.clone()),
            level::message_update(&ctx, old.clone(), new.clone(), event.clone()),
            blacklink::message_update(&ctx, old.clone(), new.clone(), event.clone()),
        );
    }
}
