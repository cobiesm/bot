use regex::Regex;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    CommandError,
    CommandResult,
    macros::command,
    Args,
    Delimiter,
};

lazy_static! {
    static ref UWUWIZER: Regex = Regex::new(r"(?i)r|l").unwrap();
}

#[command]
#[max_args(1)]
#[description = "Uwuwuwu."]
#[example = "uwu <id>"]
#[bucket = "fun"]
pub async fn uwu(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let target_id = args.advance().single::<u64>();
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let mut text = match target_id {
        Ok(target_id) => {
            let target = ctx.http.get_message(msg.channel_id.into(), target_id).await?;
            target.content_safe(ctx).await
        },
        Err(_) => {
            channel.messages(ctx, |b| {
                b.before(msg.id)
                    .limit(1)
            }).await.unwrap().first().unwrap().content_safe(ctx).await
        }
    };

    if !UWUWIZER.is_match(&text) {
        return Err("Uwuwanacak bişi buwamadım.".into());
    }

    UWUWIZER.find_iter(&text.clone()).for_each(|m| {
        text.replace_range(
            m.start()..m.end(),
            if m.as_str().chars().last().unwrap().is_uppercase() { "W" } else { "w" }
        );
        
    });

    channel.send_message(ctx, |m| {
        m.content(text)
    }).await.map_err(CommandError::from).map(|_| ())
}
