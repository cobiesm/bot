use futures::executor::block_on;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    CommandResult,
    CommandError,
    macros::command,
};

#[command]
#[only_in(guilds)]
#[max_args(1)]
#[description = "yArra"]
#[example = "<@menfie>"]
#[bucket = "fun"]
pub async fn yarra(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(e) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|em| {
            if !msg.mentions.is_empty() {
                em.description(&format!(
                    "**{}, {} sana yarra diyor**",
                    msg.mentions.first().unwrap(),
                    block_on(msg.member(&ctx)).unwrap(),
                ));
            }
            em.image("https://i.ibb.co/5sBvWHC/yarra.gif")
        })
    }).await {
        return Err(CommandError::from(e));
    };

    msg.delete(ctx).await.map_err(|e| { CommandError::from(e) })
}
