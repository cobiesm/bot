use serenity::client::Context;
use serenity::model::id::ChannelId;
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
        let w = w
            .content(content)
            .username(format!("{}@{}", message.author.name, channel_name));
        if let Some(avatar) = message.author.avatar_url() {
            w.avatar_url(avatar);
        }

        w
    })
    .await
    .ok();

    if !message.attachments.is_empty() {
        let r_client = reqwest::blocking::Client::builder()
            .use_rustls_tls()
            .build()
            .unwrap();

        hook.channel_id
            .send_files(
                &ctx,
                message
                    .attachments
                    .iter()
                    .map(|at| AttachmentType::Bytes {
                        data: r_client
                            .get(&at.proxy_url)
                            .send()
                            .unwrap()
                            .bytes()
                            .unwrap()
                            .into_iter()
                            .collect(),
                        filename: at.filename.clone(),
                    })
                    .collect::<Vec<_>>(),
                |at| at.content(format!("{}:", message.author)),
            )
            .await
            .ok();
    }
}

pub async fn message_delete(ctx: &Context, channel_id: ChannelId, message: Message) {
    if message.author.bot || BLACKLIST.contains(channel_id.as_u64()) {
        return;
    }

    undelete(ctx, message).await;
}

pub async fn message_update(
    ctx: &Context,
    old: Option<Message>,
    new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(old) = old {
        if let Some(new) = new {
            if (!new.attachments.is_empty() || new.content.len() > 3) && is_deleted(&old, &new) {
                undelete(ctx, old).await;
            }
        }
    }
}

pub fn is_deleted(old: &Message, new: &Message) -> bool {
    normalized_damerau_levenshtein(&old.content, &new.content) < 0.4
        && !new.content.contains(&old.content)
}
