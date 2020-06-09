use std::error::Error;
use serenity::client::Context;
use serenity::model::{
    channel::{ Reaction, Message, PermissionOverwrite, PermissionOverwriteType, ReactionType },
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
    if guild.is_none() {
        return;
    }

    let guild = guild.unwrap();
    if guild.to_partial_guild(ctx).unwrap().role_by_name("ACE").is_none() {
        return;
    }

    let ace = guild.to_partial_guild(ctx).unwrap().role_by_name("ACE").unwrap().id;

    if reaction.user(ctx).unwrap().bot
            || reaction.emoji != ReactionType::Unicode("👿".into())
            || !reaction.user(ctx).unwrap().has_role(ctx, guild, ace).unwrap() {

        return;
    }

    reaction.channel_id.broadcast_typing(ctx).ok();

    let online = guild.to_guild_cached(ctx).unwrap().read()
            .members(ctx, Some(1000), None).unwrap().iter().filter(|member| {

        
        if let Some(presence) = guild.to_guild_cached(ctx).unwrap().read().
                presences.get(&member.user_id()) {

            presence.status != OnlineStatus::Offline &&
                member.roles.iter().any(|role| role == &ace)
        } else {
            false
        }
    }).count();

    if online < 2 {
        return;
    }

    let reacters: Vec<User> = reaction.users(ctx, "👿", Some(100), None::<UserId>)
        .unwrap().iter().filter(|user| {
            user.has_role(ctx, guild, ace).unwrap()
        }).cloned().collect();

    let is_curse = reacters.iter().any(|user| {
        user.id == ctx.http.get_current_user().unwrap().id
    });

    let is_msg_ace = reaction.user_id.to_user_cached(ctx).unwrap().read()
        .has_role(ctx, guild, ace).unwrap();

    let unwanted_curse = is_curse && reacters.len() as f32 >= online as f32 / 2.4;

    let noncurse_rate = (online - 1) as f32 / 1.2;
    let unwanted_normal = reacters.len() as f32 >= noncurse_rate;
    let unwanted_ace = is_msg_ace && reacters.len() as f32 >= noncurse_rate;

    if unwanted_curse || (online >= 5 && (unwanted_normal || unwanted_ace)) {
        punish(ctx, reaction.message(ctx).unwrap());
    }
}

pub fn punish<T: Into<Message>>(ctx: &Context, msg: T) {
    let msg = msg.into();
    msg.delete(ctx).ok();
}
