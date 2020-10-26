use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use futures::future;
use futures::stream::{self, StreamExt};
use serenity::model::{
    channel::Message,
    channel::{PermissionOverwrite, PermissionOverwriteType, Reaction, ReactionType},
    guild::Member,
    id::ChannelId,
    id::UserId,
    permissions::Permissions,
    user::User,
};
use serenity::{client::Context, http::Http};

#[async_trait]
pub trait Muteable {
    async fn mute(&mut self, http: Arc<Http>) -> serenity::Result<()>;
    async fn unmute(&mut self, http: Arc<Http>) -> serenity::Result<()>;
}

#[async_trait]
impl Muteable for Member {
    async fn mute(&mut self, http: Arc<Http>) -> serenity::Result<()> {
        let guild = self.guild_id.to_partial_guild(http.as_ref()).await?;
        let roleid = if let Some(role) = guild.role_by_name("Muted") {
            role.id
        } else {
            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);

            let role = guild
                .create_role(http.as_ref(), |builder| {
                    builder.name("Muted").mentionable(true).colour(818_386)
                })
                .await?;

            for channel in guild.channels(http.as_ref()).await?.values() {
                channel
                    .create_permission(
                        http.as_ref(),
                        &PermissionOverwrite {
                            allow,
                            deny,
                            kind: PermissionOverwriteType::Role(role.id),
                        },
                    )
                    .await
                    .ok();
            }
            role.id
        };

        self.add_role(http.as_ref(), roleid).await?;
        Ok(())
    }

    async fn unmute(&mut self, http: Arc<Http>) -> serenity::Result<()> {
        if let Some(role) = self
            .guild_id
            .to_partial_guild(http.as_ref())
            .await?
            .role_by_name("Muted")
        {
            self.remove_role(http.as_ref(), role.id).await
        } else {
            Err(serenity::Error::Other("Rol yok hocam bu nası iş?"))
        }
    }
}

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
    let member = reaction
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
        punish(message, member, ctx, true).await;
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
        punish(message, member, ctx, unwanted_curse).await;
    }
}

async fn punish(message: Message, member: Member, ctx: &Context, is_curse: bool) {
    message.delete(ctx).await.unwrap();

    if is_curse {
        mute(member, ctx.http.clone(), message.channel_id).await;
    }
}

async fn mute(mut member: Member, http: Arc<Http>, channel: ChannelId) {
    member.mute(http.clone()).await.ok();

    tokio::spawn(async move {
        channel
            .send_message(http.as_ref(), |m| {
                m.content(format!(
                    "{}, uygunsuz kelime kullanımından ötürü oy birliği ile 5dk susturuldu.",
                    member
                ))
            })
            .await
            .ok();
        tokio::time::delay_for(std::time::Duration::from_secs(5*60)).await;
        member.unmute(http.clone()).await.ok();
        channel
            .send_message(http.as_ref(), |m| {
                m.content(format!("Artık konuşabilirsin {}.", &member))
            })
            .await
            .ok();
    });
}
