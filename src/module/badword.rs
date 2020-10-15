use regex::Regex;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};

lazy_static! {
    static ref MATCHER: Regex = Regex::new(
        r"(?ix)
        (\s|^)(yar+ak | kansız | amı\s | ibi?ne) |
        ^sik(ik|ti|er|di) | or+os+pu | piç | ana*skm | yobaz | çomar | kancık | amcık |
        yavşak | göt\s?veren
        "
    )
    .unwrap();
}

pub async fn message(ctx: &Context, msg: &Message) {
    if msg.guild_id.is_some() && !msg.author.bot && MATCHER.is_match(&msg.content) {
        msg.react(ctx, ReactionType::Unicode("👿".into()))
            .await
            .ok(); // imp
    }
}
