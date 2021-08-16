use async_trait::async_trait;
use chrono::prelude::*;
use chrono::Duration;
use serenity::client::Context;
use serenity::model::{
    channel::PermissionOverwrite, channel::PermissionOverwriteType, guild::Member, Permissions,
};

static KEY_MUTE: char = 'M';

lazy_static! {
    static ref TIME_START: DateTime<Utc> = Utc.ymd(2021, 4, 4).and_hms(0, 0, 0);
}

#[async_trait]
pub trait Muteable {
    async fn mute<T>(
        &mut self,
        ctx: Context,
        duration: Option<Duration>,
        reason: Option<T>,
    ) -> serenity::Result<()>
    where
        T: Into<String> + Send;
    async fn unmute(&mut self, ctx: Context, duration: Option<Duration>) -> serenity::Result<()>;
    async fn try_unmute(&mut self, ctx: Context);
}

#[async_trait]
impl Muteable for Member {
    async fn mute<T>(
        &mut self,
        ctx: Context,
        duration: Option<Duration>,
        reason: Option<T>,
    ) -> serenity::Result<()>
    where
        T: Into<String> + Send,
    {
        let guild = self.guild_id.to_partial_guild(&ctx).await?;
        let roleid = if let Some(role) = guild.role_by_name("Muted") {
            role.id
        } else {
            let role = guild
                .create_role(&ctx, |builder| {
                    builder.name("Muted").mentionable(true).colour(818_386)
                })
                .await?;

            let allow = Permissions::default();
            let mut deny = Permissions::SEND_MESSAGES;
            deny.insert(Permissions::SPEAK);
            deny.insert(Permissions::ADD_REACTIONS);

            for channel in guild.channels(&ctx).await?.values() {
                channel
                    .create_permission(
                        &ctx,
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

        self.add_role(&ctx, roleid).await?;
        {
            let document = nicknamedb::get(&ctx)
                .await
                .unwrap()
                .get_document(self.clone())
                .await;
            let mut document = document.lock().await;
            if let Some(duration) = duration {
                document
                    .insert(
                        KEY_MUTE,
                        ((Utc::now() + duration) - *TIME_START)
                            .num_minutes()
                            .to_string(),
                    )
                    .await;
                self.edit(&ctx, |m| m.nickname(&document.name)).await.ok();
            }
        }

        let mut message = format!(
            "{} susturuldun.",
            reason.map_or_else(|| "Sebep belirtilmeksizin".into(), T::into)
        );

        if let Some(duration) = duration {
            message.push_str(&format!(
                " ({} saat, {} dakika)",
                duration.num_hours(),
                duration.num_minutes() % 60
            ));
        }

        self.user
            .direct_message(&ctx, |m| m.content(message))
            .await
            .ok();

        Ok(())
    }

    async fn unmute(&mut self, ctx: Context, duration: Option<Duration>) -> serenity::Result<()> {
        let role = match self
            .guild_id
            .to_partial_guild(&ctx)
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
                .remove_role(&ctx, role)
                .await
                .expect("member.remove_role");
            {
                let document = nicknamedb::get(&ctx)
                    .await
                    .unwrap()
                    .get_document(self_.clone())
                    .await;
                let mut document = document.lock().await;
                document.delete::<String>(KEY_MUTE, None).await;
                self_.edit(&ctx, |m| m.nickname(&document.name)).await.ok();
            }

            self_
                .user
                .direct_message(&ctx, |m| m.content("Artık konuşabilirsin."))
                .await
                .ok();
        });
        Ok(())
    }

    async fn try_unmute(&mut self, ctx: Context) {
        let document = nicknamedb::get(&ctx)
            .await
            .unwrap()
            .get_document(self.clone())
            .await;
        let document = document.lock().await;
        let duration = document.fetch(KEY_MUTE).await;
        if let Some(duration) = duration {
            if Utc::now() - *TIME_START >= Duration::minutes(duration.parse::<i64>().unwrap()) {
                self.unmute(ctx, None).await.ok();
            }
        }
    }
}
