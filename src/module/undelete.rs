use chrono::prelude::*;
use futures::stream::{self, StreamExt};
use serenity::client::Context;
use serenity::model::id::{ChannelId, MessageId};
use serenity::{http::AttachmentType, model::channel::Message, model::event::MessageUpdateEvent};
use strsim::normalized_damerau_levenshtein;

lazy_static! {
    static ref BLACKLIST: Vec<u64> = vec![
        589470920146550794,
        589445261546487808,
        589470445989003270,
        589472041770549249,
        655552145667653654,
    ];
}

async fn undelete(ctx: &Context, message: Message) {
    let hook = ctx
        .http
        .as_ref()
        .get_webhook(752309480812969996)
        .await
        .unwrap();
    let content = message.content_safe(ctx).await;

    let channel_name = message
        .channel_id
        .name(ctx)
        .await
        .map_or(String::from("bilinmeyen"), |name| name);

    hook.execute(&ctx, false, |w| {
        w.content(content)
            .username(format!("{}@{}", message.author.name, channel_name))
            .avatar_url(message.author.avatar_url().unwrap())
    })
    .await
    .ok();

    if !message.attachments.is_empty() {
        hook.channel_id
            .send_files(
                &ctx,
                stream::iter(&message.attachments)
                    .then(|at| async move {
                        AttachmentType::Bytes {
                            data: at.download().await.unwrap().into(),
                            filename: at.filename.clone(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .await,
                |at| at.content(format!("^^{}", message.author)),
            )
            .await
            .ok();
    }
}

pub async fn message_delete(ctx: &Context, channel_id: ChannelId, message_id: MessageId) {
    let old_message = match ctx.cache.message(channel_id, message_id).await {
        Some(m) => m,
        None => return,
    };

    let tc = &Utc::now();
    let tm = message_id.created_at();
    if old_message.author.bot
        || tc.timestamp_millis() - tm.timestamp_millis() < 1500
        || old_message.is_private()
        || BLACKLIST.contains(channel_id.as_u64())
    {
        return;
    }

    undelete(ctx, old_message).await;
}

pub async fn message_update(
    ctx: &Context,
    old: Option<Message>,
    new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(old) = old {
        if let Some(new) = new {
            if is_deleted(&old, &new) {
                undelete(ctx, old).await;
            }
        }
    }
}

pub fn is_deleted(old: &Message, new: &Message) -> bool {
    normalized_damerau_levenshtein(&old.content, &new.content) < 0.5
}
