use crate::muteable::Muteable;

use chrono::Duration;
use futures::executor::block_on;
use futures::future;
use futures::stream::{self, StreamExt};

use serenity::client::Context;
use serenity::model::{
    channel::{Reaction, ReactionType},
    id::UserId,
    user::User,
};

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let user_reacted = match reaction.user(ctx).await {
        Ok(u) => u,
        Err(_) => {
            return;
        }
    };

    if reaction.emoji != ReactionType::Unicode("👿".into()) || user_reacted.bot {
        return;
    }

    let message = reaction.message(ctx).await.unwrap();
    let mut member = reaction
        .guild_id
        .unwrap()
        .member(ctx, message.author.id)
        .await
        .unwrap();
    let guild = match reaction.guild_id {
        Some(g) => g.to_guild_cached(ctx).await.unwrap().clone(),
        None => {
            return;
        }
    };

    if user_reacted
        .has_role(ctx, guild.id, guild.role_by_name("🔑").unwrap())
        .await
        .unwrap()
    {
        reaction.channel_id.broadcast_typing(ctx).await.ok();
        message.delete(&ctx.http).await.ok();
        return;
    }

    let ace = match guild.role_by_name("ACE") {
        Some(r) => r.id,
        None => {
            return;
        }
    };

    let ace_count = guild
        .presences
        .iter()
        .filter(|presence| {
            block_on(guild.member(ctx, presence.0))
                .unwrap()
                .roles
                .iter()
                .any(|role| role == &ace)
        })
        .count();

    reaction.channel_id.broadcast_typing(ctx).await.ok();

    let reacters = stream::iter(
        reaction
            .users(
                ctx,
                ReactionType::Unicode("👿".into()),
                Some(100),
                None::<UserId>,
            )
            .await
            .unwrap(),
    )
    .filter(|u| future::ready(block_on(u.has_role(ctx, guild.id, ace)).unwrap()))
    .collect::<Vec<User>>()
    .await;

    let is_curse = !stream::iter(&reacters)
        .then(|u| async move { u.id == ctx.http.get_current_user().await.unwrap().id })
        .collect::<Vec<bool>>()
        .await
        .is_empty();

    let unwanted_curse = is_curse && reacters.len() as f32 >= (ace_count as f32 / 2.4).round();
    let unwanted_noncurse = reacters.len() as f32 >= (ace_count as f32 / 1.4).round();

    if unwanted_curse || unwanted_noncurse {
        message.delete(&ctx.http).await.expect("message.delete");

        member
            .mute(
                ctx.http.clone(),
                Some(Duration::minutes(10)),
                Some("Diğer üyeler tarafından hoş karşılanmayan bir kelime kullandığın için"),
            )
            .await
            .ok();
    }
}
