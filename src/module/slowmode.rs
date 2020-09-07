use chrono::Duration;
use serenity::model::channel::Message;
use serenity::prelude::Context;

pub fn message(ctx: &Context, msg: &Message) {
    if msg.guild_id.is_none() || msg.author.bot
        || !msg.member.as_ref().expect("member").roles.is_empty()
    {
        return;
    }

    let messages = msg.channel_id.messages(ctx, |builder| {
        builder.limit(10)
    }).unwrap().iter().filter(|oldmsg| {
        oldmsg.author == msg.author
    }).cloned().collect::<Vec<Message>>();

    if messages.len() > 1 && msg.timestamp - messages.get(1).unwrap().timestamp < Duration::milliseconds(750) {
        msg.delete(ctx).ok();
    }
}
