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
pub fn yarra(ctx: &mut Context, msg: &Message) -> CommandResult {
    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|em| {
            if !msg.mentions.is_empty() {
                em.description(&format!(
                    "**{}, {} sana yarra diyor**",
                    msg.mentions.first().unwrap(),
                    msg.member(&ctx).unwrap(),
                ));
            }
            em.image("https://i.ibb.co/5sBvWHC/yarra.gif")
        })
    }) {
        Err(e) => { return Err(CommandError::from(e)) },
        Ok(_) => ()
    };
    msg.delete(ctx).map_err(|e| { CommandError::from(e) })
}
