// TODO: Manage multiple mute executions with different durations. We don't want the previous mute
// if we have a new one.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use serenity::{
    http::Http,
    model::{
        channel::PermissionOverwrite, channel::PermissionOverwriteType, guild::Member, Permissions,
    },
};

#[async_trait]
pub trait Muteable {
    async fn mute<T>(
        &mut self,
        http: Arc<Http>,
        duration: Option<Duration>,
        reason: Option<T>,
    ) -> serenity::Result<()>
    where
        T: Into<String> + Send;
    async fn unmute(&mut self, http: Arc<Http>, duration: Option<Duration>)
        -> serenity::Result<()>;
}

#[async_trait]
impl Muteable for Member {
    async fn mute<T>(
        &mut self,
        http: Arc<Http>,
        duration: Option<Duration>,
        reason: Option<T>,
    ) -> serenity::Result<()>
    where
        T: Into<String> + Send,
    {
        let guild = self.guild_id.to_partial_guild(http.as_ref()).await?;
        let roleid = if let Some(role) = guild.role_by_name("Muted") {
            role.id
        } else {
            let role = guild
                .create_role(http.as_ref(), |builder| {
                    builder.name("Muted").mentionable(true).colour(818_386)
                })
                .await?;

            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);

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

        let mut message = format!(
            "{} susturuldun.",
            reason.map_or_else(|| "Sebep belirtilmeksizin".into(), T::into)
        );

        if let Some(duration) = duration {
            message.push_str(&format!(
                " ({} saat, {} dakika, {} saniye)",
                duration.num_hours(),
                duration.num_minutes() % 60,
                duration.num_seconds() % 3600,
            ));
        }

        self.user
            .direct_message(http.clone(), |m| m.content(message))
            .await
            .ok();

        if let Some(duration) = duration {
            let mut self_ = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(duration.to_std().expect("duration")).await;
                self_.unmute(http.clone(), None).await.ok();
            });
        }
        Ok(())
    }

    async fn unmute(
        &mut self,
        http: Arc<Http>,
        duration: Option<Duration>,
    ) -> serenity::Result<()> {
        let role = match self
            .guild_id
            .to_partial_guild(http.as_ref())
            .await?
            .role_by_name("Muted")
        {
            Some(role) => role.id,
            None => {
                return Err(serenity::Error::Other("Rol yok hocam bu nası iş?"));
            }
        };

        let mut self_ = self.clone();
        tokio::spawn(async move {
            if let Some(duration) = duration {
                tokio::time::sleep(duration.to_std().expect("duration")).await;
            }

            self_
                .remove_role(http.as_ref(), role)
                .await
                .expect("member.remove_role");
            self_
                .user
                .direct_message(http, |m| m.content("Artık konuşabilirsin."))
                .await
                .ok();
        });
        Ok(())
    }
}
