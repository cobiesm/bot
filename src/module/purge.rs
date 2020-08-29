use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    CommandResult,
    CommandError,
    macros::command
};

#[command]
    let signed = *numbers(msg).first().ok_or("Sayı girmen lazım alooo???")?;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let amount: u64 = if signed.is_sign_positive() && signed < u64::max_value() as f64 {
        signed.floor() as u64
    } else {
        return Err(CommandError("Bozuk sayı girdin arkadaşım.".into()));
pub fn purge(ctx: &mut Context, msg: &Message) -> CommandResult {
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

fn numbers(msg: &Message) -> Vec<f64> {
    let it = msg.content.split_whitespace();
    let mut numbers: Vec<f64> = Vec::new();
    for slice in it {
        if let Ok(number) = slice.parse::<f64>() {
            numbers.push(number);
        }
    };
    numbers
}
