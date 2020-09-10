use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    CommandResult,
    CommandError,
    macros::command,
    Args,
    Delimiter,
};

#[command]
#[num_args(1)]
pub async fn purge(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let amount = match args.advance().single::<u64>() {
        Ok(num) => num,
        Err(_) => { return Err(CommandError::from("Girdiğiniz sayı geri verildi.")); }
    };

    msg.channel_id.broadcast_typing(&ctx).await.ok();

    let with_id = amount > 100;
    let mut messages = msg.channel_id.messages(&ctx, |builder| {
        let builder = builder.before(msg.id);
        if with_id {
            builder.after(amount)
        } else {
            builder.limit(amount)
        }
    }).await?;

    if with_id {
        messages.remove(0);
    }

    msg.channel_id.delete_messages(&ctx, messages).await
        .map_err(|e| { CommandError::from(e) })
}
