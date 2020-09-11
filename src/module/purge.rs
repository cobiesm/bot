use futures::executor::block_on;
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
        Err(_) => { return Err(CommandError::from("girdiğiniz sayı geri verildi.")); }
    };

    msg.channel_id.broadcast_typing(&ctx).await.ok();

    let with_id = amount + 1 > 100;
    let messages = msg.channel_id.messages(&ctx, |builder| {
        if with_id {
            if block_on(msg.channel_id.message(ctx, amount)).is_ok() {
                builder.after(amount)
            } else {
                builder
            }
        } else {
            builder.limit(amount + 1)
        }
    }).await?;

    if messages.is_empty() {
        Err("yok böyle bi mesaj.".into())
    } else {
        msg.channel_id.delete_messages(&ctx, messages).await
            .map_err(CommandError::from)
    }
}
