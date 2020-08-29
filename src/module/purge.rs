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
#[only_in(guilds)]
#[num_args(1)]
pub fn purge(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let amount = match args.single::<u64>() {
        Ok(num) => num,
        Err(_) => { return Err(CommandError("Bozuk sayı girdin arkadaşım.".into())); }
    };

    msg.channel_id.broadcast_typing(&ctx).ok();

    let messages = msg.channel_id.messages(&ctx, |builder| {
        builder.before(msg.id).limit(amount)
    })?;

    for message in messages {
        message.delete(&ctx)?;
    }

    Ok(())
}
