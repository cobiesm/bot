use regex::Regex;
use serenity::client::Context;
use serenity::model::channel::Message;
use strsim::normalized_damerau_levenshtein;

lazy_static! {
    static ref MATCHER: Regex =
        Regex::new(r"(?ix)(\w+){1}\.[a-z]{2,4}\b(/[a-z0-9@:%\s+.~\#?&/=-]*)?").unwrap();
}

pub async fn message(ctx: &Context, msg: &Message) {
    let captures = MATCHER.captures(&msg.content);
    if msg.guild_id.is_some() && !msg.author.bot && captures.is_some() {
        let mut domain = captures.unwrap().get(1).unwrap().as_str().to_owned();
        domain.replace_range(
            ..1,
            &domain
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>(),
        );

        if let Some(black) = is_blacked(&domain) {
            msg.reply_mention(ctx, format!("neden {} linki paylaşıyorsun ki?", black))
                .await
                .ok();
            msg.author.dm(ctx, |pm| pm.content(&msg.content)).await.ok();
            msg.delete(ctx).await.unwrap();
        }
    }
}

fn is_blacked(domain: &str) -> Option<&str> {
    for black in &["facebook", "twitter", "spotify", "webtekno", "onedio"] {
        if normalized_damerau_levenshtein(domain, black) >= 0.5 {
            return Some(black);
        }
    }

    None
}
