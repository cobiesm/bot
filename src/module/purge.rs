use serenity::framework::standard::{
    macros::command, Args, CommandError, CommandResult, Delimiter,
};
use serenity::model::channel::Message;
use serenity::prelude::Context;

#[command]
#[num_args(1)]
pub async fn purge(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let amount = match args.advance().single::<u64>() {
        Ok(num) => num,
        Err(_) => {
            return Err(CommandError::from("girdiğiniz sayı geri verildi."));
        }
    };

    msg.channel_id.broadcast_typing(&ctx).await.ok();

    let with_id = msg.channel_id.message(ctx, amount).await.is_ok();
    let messages = msg
        .channel_id
        .messages(&ctx, |builder| {
            if with_id {
                builder.after(amount)
            } else if amount < 100 {
                builder.limit(amount + 1)
            } else {
                builder.limit(1)
            }
        })
        .await?;

    if messages.len() < 2 {
        Err("yok böyle bi mesaj.".into())
    } else {
        msg.channel_id
            .delete_messages(&ctx, messages)
            .await
            .map_err(CommandError::from)
    }
}
