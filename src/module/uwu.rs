use regex::Regex;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;

lazy_static! {
    static ref UWUWIZER: Regex = Regex::new(r"(?i)r|l").unwrap();
}

static ERR_NOMATCH: &str = "uwuwanacak bişi buwamadım.";

#[command]
#[max_args(1)]
#[description = "Uwuwuwu."]
#[example = "uwu <id>"]
#[bucket = "fun"]
pub async fn uwu(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let target_id = args.single::<u64>();
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let mut text = match target_id {
        Ok(target_id) => msg.channel_id.message(ctx, target_id).await?,
        Err(_) => channel
            .messages(ctx, |b| b.before(msg.id).limit(1))
            .await
            .map_err(|_| ERR_NOMATCH)?
            .first()
            .ok_or(ERR_NOMATCH)?
            .clone(),
    }
    .content_safe(&ctx)
    .await;

    if !UWUWIZER.is_match(&text) {
        return Err(ERR_NOMATCH.into());
    }

    UWUWIZER.find_iter(&text.clone()).for_each(|m| {
        text.replace_range(
            m.start()..m.end(),
            if m.as_str().chars().last().unwrap().is_uppercase() {
                "W"
            } else {
                "w"
            },
        );
    });

    channel
        .send_message(ctx, |m| m.content(text))
        .await
        .map_err(CommandError::from)
        .map(|_| ())
}
