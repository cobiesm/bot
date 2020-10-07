use super::selfmod::Muteable;
use serenity::{
    client::Context, framework::standard::macros::command, framework::standard::Args,
    framework::standard::CommandError, framework::standard::CommandResult, model::channel::Message,
    model::guild::Member,
};

#[command]
#[num_args(1)]
pub async fn mute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    find_member(ctx, msg, args)
        .await?
        .mute(ctx)
        .await
        .map_err(CommandError::from)
}

#[command]
#[num_args(1)]
pub async fn unmute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    find_member(ctx, msg, args)
        .await?
        .unmute(ctx)
        .await
        .map_err(CommandError::from)
}

pub async fn find_member(ctx: &Context, msg: &Message, mut args: Args) -> serenity::Result<Member> {
    Ok(msg
        .guild(ctx)
        .await
        .ok_or(serenity::Error::Other("guild!!?"))?
        .members_containing(&args.single::<String>().unwrap(), false, true)
        .await
        .first()
        .ok_or(serenity::Error::Other("kim bu amk tanımıyorum."))?
        .0
        .clone())
    //.map_or(Err(serenity::Error::Other("Kim bu amk tanımıyorum.")))
}
