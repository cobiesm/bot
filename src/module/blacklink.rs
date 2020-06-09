use regex::Regex;
use serenity::client::Context;
use serenity::model::channel::Message;

lazy_static! {
    static ref MATCHER: Regex = Regex::new(
        r"(?ix)(
        facebook | twitter | spotify | webtekno
        ){1}\.[a-z]{2,4}\b(/[a-z0-9@:%\s+.~\#?&/=-]*)?"
    ).unwrap();
}

pub fn message(ctx: &Context, msg: &Message) {
    if msg.guild_id.is_some() && !msg.author.bot && MATCHER.is_match(&msg.content) {
        msg.reply(ctx, "Bu linki paylaşamazsın.").ok();
        msg.author.dm(ctx, |pm| {
            pm.content(&msg.content)
        }).ok();
        msg.delete(ctx).unwrap();
    }
}
