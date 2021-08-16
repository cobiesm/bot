use crate::module::{
    badword, blacklink, clap, faq, irc, level, presence, selfmod, slowmode, undelete,
};
use serenity::model::channel::{Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, MessageId};
use serenity::prelude::EventHandler;
use serenity::{async_trait, model::event::MessageUpdateEvent};
use serenity::{client::Context, model::id::GuildId};
use tokio::join;

pub struct Handler;

async fn shouldnt_handle(ctx: &Context, message: &Message) -> bool {
    #[cfg(debug_assertions)]
    println!(
        "{}: {},{},{:?}",
        message.content,
        message.author.bot,
        message.is_private(),
        message.webhook_id
    );

    message.author.bot
        || (message.is_private()
            && ctx
                .http
                .get_channel(message.channel_id.0)
                .await
                .expect("channel")
                .guild()
                .is_none())
        || message.webhook_id.is_some()
}

async fn is_admin(ctx: &Context, message: &Message) -> bool {
    let mut i = 1;
    let member = loop {
        match ctx
            .http
            .get_member(
                ctx.http
                    .get_channel(message.channel_id.0)
                    .await
                    .expect("channel")
                    .guild()
                    .expect("guild")
                    .guild_id
                    .0,
                message.author.id.0,
            )
            .await
        {
            Ok(member) => break member,
            Err(e) => {
                if i == 10 {
                    eprintln!("Couldn't get the member because {}.", e);
                    return false;
                }

                println!("Trying to get member {}", i);
                i += 1;
            }
        };
    };

    let perms = member.permissions(&ctx).await;

    if let Ok(perms) = perms {
        return perms.administrator();
    } else if let Err(err) = perms {
        eprintln!("Can't fetch permissions because {}.", err);
    }

    false
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if shouldnt_handle(&ctx, &new_message).await {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Message received \"{}\".", new_message.content);

        join!(
            faq::message(&ctx, &new_message),
            irc::message(&ctx, &new_message),
        );

        if !is_admin(&ctx, &new_message).await {
            join!(
                blacklink::message(&ctx, &new_message),
                badword::message(&ctx, &new_message),
                slowmode::message(&ctx, &new_message),
                level::message(&ctx, &new_message)
            );
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let message = ctx
            .http
            .get_message(reaction.channel_id.0, reaction.message_id.0)
            .await
            .expect("message");
        if shouldnt_handle(&ctx, &message).await {
            return;
        }

        #[cfg(debug_assertions)]
        println!(
            "Reaction received \"{}\" from \"{}\". At channel \"{}\" to message \"{}\".",
            reaction.emoji,
            reaction.user_id.unwrap().to_user(&ctx).await.unwrap().name,
            reaction.channel_id.0,
            reaction.message_id.0,
        );

        join!(
            selfmod::reaction_add(&ctx, &reaction),
            clap::reaction_add(&ctx, &reaction),
        );

        if !is_admin(&ctx, &message).await {
            level::reaction_add(&ctx, &reaction).await;
        }
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let message = reaction.message(&ctx).await.expect("message");
        if shouldnt_handle(&ctx, &message).await {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Reaction removed \"{}\".", reaction.emoji);

        if !is_admin(&ctx, &message).await {
            level::reaction_remove(&ctx, &reaction).await;
        }
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        #[cfg(debug_assertions)]
        println!("Ready");

        join!(presence::ready(&ctx), level::ready(&ctx), irc::ready(&ctx));
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
            #[cfg(debug_assertions)]
            eprintln!("Could not find the message in cache.");
            return;
        };

        if shouldnt_handle(&ctx, &message).await {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Message deleted \"{}\".", message.content);

        if !is_admin(&ctx, &message).await {
            join!(
                undelete::message_delete(&ctx, channel_id, message.clone()),
                level::message_delete(&ctx, channel_id, message.clone())
            );
        }
    }

    async fn message_update(
        &self,
        ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(message) = &new {
            if shouldnt_handle(&ctx, message).await {
                return;
            }
        } else {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Message updated \"{}\".", new.clone().unwrap().content);

        join!(
            undelete::message_update(&ctx, old.clone(), new.clone(), event.clone()),
            blacklink::message_update(&ctx, old.clone(), new.clone(), event.clone()),
            irc::message_update(&ctx, old.clone(), new.clone(), event.clone()),
        );

        if !is_admin(&ctx, new.as_ref().unwrap()).await {
            level::message_update(&ctx, old.clone(), new.clone(), event.clone()).await;
        }
    }
}
