use std::error::Error;
use serenity::client::Context;
use serenity::model::{
    channel::{ Reaction, PermissionOverwrite, PermissionOverwriteType, ReactionType },
    guild::Member,
    permissions::Permissions,
    id::UserId,
    user::{
        OnlineStatus, User
    }
};

trait Muteable {
    fn mute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>>;
    fn unmute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>>;
}

impl Muteable for Member {
    fn mute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        let guild = self.guild_id.to_partial_guild(ctx)?;
        let roleid = if let Some(role) = guild.role_by_name("muted") {
            role.id
        } else {
            let role = guild.create_role(ctx, |builder| {
                builder.name("muted")
                    .mentionable(true)
                    .colour(818_386)
            })?;

            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);
            let overwrite = PermissionOverwrite {
                allow,
                deny,
                kind: PermissionOverwriteType::Role(role.id)
            };

            guild.channels(ctx)?.values().for_each(|channel| {
                channel.create_permission(ctx, &overwrite).ok();
            });
            role.id
        };

        self.add_role(ctx, roleid)?;
        Ok(())
    }

    fn unmute(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        if let Some(role) = self.guild_id.to_partial_guild(ctx)?.role_by_name("muted") {
            self.remove_role(ctx, role.id)?;
        }
        Ok(())
    }
}

#[allow(clippy::unreadable_literal)]
pub fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let guild = reaction.guild_id;

    if reaction.emoji != ReactionType::Unicode("👿".into())
        || reaction.user(ctx).unwrap().bot
        || guild.is_none()
    {
        return;
    }

    let guild = guild.unwrap().to_guild_cached(ctx).unwrap().read().clone();
    let ace = guild.role_by_name("ACE").unwrap().id;
    let is_ace = reaction.user(ctx).unwrap().has_role(ctx, guild.id, ace).unwrap();

    if guild.role_by_name("ACE").is_none() || !is_ace {
        return;
    }

    let online_count = guild.presences.iter().filter(|p| {
        p.1.status != OnlineStatus::Offline &&
            !p.0.to_user(ctx).unwrap().bot &&
            p.0 != &reaction.message(ctx).unwrap().author.id &&
            guild.member(ctx, p.0).unwrap().roles.iter().any(|role| role == &ace)
    }).count();

    if online_count <= 2 || (!is_ace && online_count < 2) {
        return;
    }

    reaction.channel_id.broadcast_typing(ctx).ok();

    let reacters: Vec<User> = reaction.users(ctx, "👿", Some(100), None::<UserId>)
        .unwrap().iter().filter(|user| {
            user.has_role(ctx, guild.id, ace).unwrap()
        }).cloned().collect();

    let is_curse = reacters.iter().any(|user| {
        user.id == ctx.http.get_current_user().unwrap().id
    });

    let unwanted_curse = is_curse
        && reacters.len() as f32 >= (online_count as f32 / 2.4).round();

    let unwanted_noncurse = reacters.len() == online_count;

    if unwanted_curse || (online_count >= 3 && unwanted_noncurse) {
        reaction.message(ctx).unwrap().delete(ctx).unwrap();
    }
}
