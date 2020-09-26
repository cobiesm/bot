use async_trait::async_trait;
use futures::future;
use futures::executor::block_on;
use futures::stream::{self, StreamExt};
use serenity::{client::Context, framework::standard::Args, framework::standard::macros::command, framework::standard::CommandResult, model::channel::Message, framework::standard::CommandError};
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

#[async_trait]
trait Muteable {
    async fn mute(&mut self, ctx: &Context) -> serenity::Result<()>;
    async fn unmute(&mut self, ctx: &Context) -> serenity::Result<()>;
}

#[async_trait]
impl Muteable for Member {
    async fn mute(&mut self, ctx: &Context) -> serenity::Result<()> {
        let guild = self.guild_id.to_partial_guild(ctx).await?;
        let roleid = if let Some(role) = guild.role_by_name("Muted") {
            role.id
        } else {
            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);

            ctx.invisible().await;
            let role = guild.create_role(ctx, |builder| {
                builder.name("Muted")
                    .mentionable(true)
                    .colour(818_386)
            }).await?;

            for channel in guild.channels(ctx).await?.values() {
                channel.create_permission(ctx, &PermissionOverwrite {
                    allow,
                    deny,
                    kind: PermissionOverwriteType::Role(role.id)
                }).await.ok();
            }
            role.id
        };

        self.add_role(ctx, roleid).await?;
        Ok(())
    }

    async fn unmute(&mut self, ctx: &Context) -> serenity::Result<()> {
        if let Some(role) = self.guild_id.to_partial_guild(ctx).await?.role_by_name("Muted") {
            self.remove_role(ctx, role.id).await
        } else {
            Err(serenity::Error::Other("Rol yok hocam bu nası iş?"))
        }
    }
}

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
            p.1.status != OnlineStatus::Offline
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

    let unwanted_noncurse = reacters.len() as f32 >= (online_count as f32 / 1.15).round();

    if unwanted_curse || (online_count >= 3 && unwanted_noncurse) {
        reaction.message(ctx).await.unwrap().delete(ctx).await.unwrap();
    }
}

#[command]
#[num_args(1)]
pub async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Some(user) = msg.guild(ctx).await.unwrap().member_named(&args.single::<String>()?) {
        let mut user = ctx.http.get_member(user.guild_id.into(), user.user.id.into()).await?;
        user.mute(ctx).await.map_err(CommandError::from)
    } else {
        Err("Kim bu amk tanımıyorum.".into())
    }
}

#[command]
#[num_args(1)]
pub async fn unmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Some(user) = msg.guild(ctx).await.unwrap().member_named(&args.single::<String>()?) {
        let mut user = ctx.http.get_member(user.guild_id.into(), user.user.id.into()).await?;
        user.unmute(ctx).await.map_err(CommandError::from)
    } else {
        Err("Kim bu amk tanımıyorum.".into())
    }
}
