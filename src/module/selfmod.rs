use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use futures::future;
use futures::stream::{self, StreamExt};
use serenity::model::{
    channel::{PermissionOverwrite, PermissionOverwriteType, Reaction, ReactionType},
    gateway::Presence,
    guild::Member,
    id::ChannelId,
    id::UserId,
    permissions::Permissions,
    user::{OnlineStatus, User},
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
    let user = match reaction.user(ctx).await {
        Ok(u) => u,
        Err(_) => {
            return;
        }
    };

    if reaction.emoji != ReactionType::Unicode("👿".into()) || user.bot {
        return;
    }

    let guild = match reaction.guild_id {
        Some(g) => g.to_guild_cached(ctx).await.unwrap().clone(),
        None => {
            return;
        }
    };

    let ace = match guild.role_by_name("ACE") {
        Some(r) => r.id,
        None => {
            return;
        }
    };

    let is_ace = match user.has_role(ctx, guild.id, ace).await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return;
        } // TODO: Error printing could be nicer
    };

    let online_count = stream::iter(&guild.presences)
        .filter(|p| {
            future::ready(
                p.1.status != OnlineStatus::Offline
                    && block_on(guild.member(ctx, p.0))
                        .unwrap()
                        .roles
                        .iter()
                        .any(|role| role == &ace),
            )
        })
        .collect::<Vec<(&UserId, &Presence)>>()
        .await
        .len();

    if (online_count <= 2 && is_ace) || online_count < 2 {
        return;
    }

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

    let unwanted_curse = is_curse && reacters.len() as f32 >= (online_count as f32 / 2.4).round();

    let unwanted_noncurse = reacters.len() as f32 >= (online_count as f32 / 1.15).round();

    if unwanted_curse || (online_count >= 3 && unwanted_noncurse) {
        reaction
            .message(ctx)
            .await
            .unwrap()
            .delete(ctx)
            .await
            .unwrap();
    }

    if unwanted_curse {
        mute(
            guild.member(ctx, user.id).await.unwrap(),
            ctx.http.clone(),
            reaction.channel_id,
        )
        .await;
    }
}

async fn mute(mut member: Member, http: Arc<Http>, channel: ChannelId) {
    member.mute(http.clone()).await.ok();

    let mut mem2 = member.clone();
    let http2 = http.clone();
    let tstatus = tokio::spawn(async move {
        channel
            .send_message(http2.as_ref(), |m| {
                m.content(format!(
                    "{}, uygunsuz kelime kullanımından ötürü oy birliği ile susturuldu.",
                    mem2
                ))
            })
            .await
            .ok();
        std::thread::sleep(std::time::Duration::from_secs(1200));
        mem2.unmute(http2.clone()).await.ok();
        channel
            .send_message(http2.as_ref(), |m| {
                m.content(format!("Artık konuşabilirsin {}.", &mem2))
            })
            .await
            .ok();
    })
    .await;

    if tstatus.is_err() {
        member.unmute(http.clone()).await.ok();
        channel
            .send_message(http.as_ref(), |m| {
                m.content("Susturma başarısız olduğu için işlem geri alındı.")
            })
            .await
            .ok();
    }
}
