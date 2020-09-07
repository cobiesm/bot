use chrono::prelude::*;
use serenity::http::AttachmentType;
use serenity::client::Context;
use serenity::model::id::{ ChannelId, MessageId };

lazy_static!(
    static ref BLACKLIST: Vec<u64> = vec![
        589470920146550794,
        589445261546487808,
        589470445989003270,
        589472041770549249,
        655552145667653654,
    ];
);

pub fn message_delete(ctx: &Context, channel_id: &ChannelId, message_id: &MessageId) {
    let old_message = match ctx.cache.read().message(channel_id, message_id) {
        Some(m) => m,
        None => { return }
    };

    let tc = &Utc::now();
    let tm = message_id.created_at();
    if old_message.author.bot
        || tc.timestamp_millis() - tm.timestamp_millis() < 1500
        || old_message.is_private()
        || BLACKLIST.contains(channel_id.as_u64())
    {
        return
    }

    let hook = ctx.http.as_ref().get_webhook(752309480812969996).unwrap();

    hook.execute(&ctx, false, |w| {
        w.content(&old_message.content)
            .username(format!("{}@{}", &old_message.author.name,
                              old_message.channel_id.name(&ctx).unwrap()))
            .avatar_url(old_message.author.avatar_url().unwrap())
    }).ok();

    if !old_message.attachments.is_empty() {
        hook.channel_id.send_files(
            &ctx,
            old_message.attachments.iter().map(|at| {
                AttachmentType::Bytes {
                    data: at.download().unwrap().into(),
                    filename: at.filename.clone(),
                }
            }), |at| at.content(format!("^^{}", &old_message.author))
        ).ok();
    }
}
