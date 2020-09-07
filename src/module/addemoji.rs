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
#[num_args(2)]
pub fn addemoji(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let name = match args.advance().single::<String>() {
        Ok(name) => name,
        Err(e) => { return Err(CommandError::from(e)) },
    };
    let emoji = match args.single::<String>() {
        Ok(emoji) => emoji,
        Err(e) => { return Err(CommandError::from(e)) },
    };

    msg.guild(&ctx).unwrap().read().create_emoji(&ctx, &name, &emoji)
        .map_err(|e| { CommandError::from(e) }).map(|_| ())
}
