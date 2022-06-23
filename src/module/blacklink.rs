use regex::Regex;
use serenity::model::channel::Message;
use serenity::{client::Context, model::event::MessageUpdateEvent};
use strsim::normalized_damerau_levenshtein;

lazy_static! {
    static ref MATCHER: Regex =
        Regex::new(r"(?ix)(?:\w+\.)?(\w+){1}\.[a-z]{2,4}\b(?:/[a-z0-9@:%\s+.~\#?&/=-]*)?").unwrap();
}

pub async fn message(ctx: &Context, msg: &Message) {
    let captures = match MATCHER.captures(&msg.content) {
        Some(captures) => captures,
        None => return,
    };

    let domain = captures
        .get(1)
        .unwrap() // SAFETY: Capture group always exist.
        .as_str()
        .to_owned();

    if let Some(black) = is_blacked(&domain) {
        msg.reply_mention(ctx, format!("neden {} linki paylaşıyorsun ki?", black))
            .await
            .ok();
        msg.author.dm(ctx, |pm| pm.content(&msg.content)).await.ok();
        match msg.delete(ctx).await {
            Ok(()) => (),
            Err(e) => {
                msg.reply(&ctx, format!("Bu mesajı silemiyorum çünkü {}", e))
                    .await
                    .ok();
            }
        };
    }
}

pub async fn message_update(
    ctx: &Context,
    _old: Option<Message>,
    new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(new) = new {
        message(ctx, &new).await;
    }
}

fn is_blacked(domain: &str) -> Option<&str> {
    for black in &[
        "Facebook", "Spotify", "WebTekno", "Onedio", "tinyurl", "kisa", "cutt",
        "shorturl",
    ] {
        if normalized_damerau_levenshtein(domain, black) >= 0.65 {
            return Some(black);
        }
    }

    None
}
