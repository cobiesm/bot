use std::error::Error;
use futures::future;
use futures::executor::block_on;
use futures::stream::{self, StreamExt};
use serenity::client::Context;
use serenity::model::{
    channel::{ Reaction, PermissionOverwrite, PermissionOverwriteType, ReactionType },
    guild::Member,
    permissions::Permissions,
    id::UserId,
    user::{
        OnlineStatus, User
    },
    gateway::Presence
};

trait Muteable {
    fn mute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>>;
    fn unmute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>>;
}

impl Muteable for Member {
    fn mute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        let guild = block_on(self.guild_id.to_partial_guild(ctx))?;
        let roleid = if let Some(role) = guild.role_by_name("muted") {
            role.id
        } else {
            let role = block_on(guild.create_role(ctx, |builder| {
                builder.name("muted")
                    .mentionable(true)
                    .colour(818_386)
            }))?;

            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);
            let overwrite = PermissionOverwrite {
                allow,
                deny,
                kind: PermissionOverwriteType::Role(role.id)
            };

            block_on(guild.channels(ctx))?.values().for_each(|channel| {
                block_on(channel.create_permission(ctx, &overwrite)).ok();
            });
            role.id
        };

        block_on(self.add_role(ctx, roleid))?;
        Ok(())
    }

    fn unmute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        if let Some(role) = block_on(self.guild_id.to_partial_guild(ctx))?.role_by_name("muted") {
            block_on(self.remove_role(ctx, role.id))?;
        }
        Ok(())
    }
}

#[allow(clippy::unreadable_literal)]
pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let user = match reaction.user(ctx).await {
        Ok(u) => u,
        Err(_) => { return; }
    };

    if reaction.emoji != ReactionType::Unicode("👿".into()) || user.bot {
        return;
    }

    let guild = match reaction.guild_id {
        Some(g) => g.to_guild_cached(ctx).await.unwrap().clone(),
        None => { return; }
    };

    let ace = match guild.role_by_name("ACE") {
        Some(r) => r.id,
        None => { return; }
    };

    let is_ace = match user.has_role(ctx, guild.id, ace).await {
        Ok(r) => r,
        Err(e) => { println!("{}", e); return; } // TODO: Error printing could be nicer
    };

    let online_count = stream::iter(&guild.presences).filter(|p| {
        future::ready(
            p.1.status != OnlineStatus::Offline && !block_on(p.0.to_user(ctx)).unwrap().bot
            && block_on(guild.member(ctx, p.0)).unwrap().roles.iter().any(|role| role == &ace)
        )
    }).collect::<Vec<(&UserId, &Presence)>>().await.len();

    if (online_count <= 2 && is_ace) || online_count < 2 {
        return;
    }

    reaction.channel_id.broadcast_typing(ctx).await.ok();

    let reacters = stream::iter(
        reaction.users(
            ctx,
            ReactionType::Unicode("👿".into()),
            Some(100), None::<UserId>
        ).await.unwrap()
    ).filter(|u| {
        future::ready(block_on(u.has_role(ctx, guild.id, ace)).unwrap())
    }).collect::<Vec<User>>().await;

    let is_curse = !stream::iter(&reacters).then(|u| async move {
        u.id == ctx.http.get_current_user().await.unwrap().id
    }).collect::<Vec<bool>>().await.is_empty();

    let unwanted_curse = is_curse
        && reacters.len() as f32 >= (online_count as f32 / 2.4).round();

    let unwanted_noncurse = reacters.len() == online_count;

    if unwanted_curse || (online_count >= 3 && unwanted_noncurse) {
        reaction.message(ctx).await.unwrap().delete(ctx).await.unwrap();
    }
}
