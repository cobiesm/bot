use super::selfmod::Muteable;
use serenity::{framework::standard::macros::command, client::Context, model::channel::Message, framework::standard::Args, framework::standard::CommandResult, framework::standard::CommandError, model::guild::Member};

#[command]
#[num_args(1)]
pub async fn mute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    find_member(ctx, msg, args).await.unwrap().mute(ctx).await
        .map_err(CommandError::from)
}

#[command]
#[num_args(1)]
pub async fn unmute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    find_member(ctx, msg, args).await.unwrap().unmute(ctx).await
        .map_err(CommandError::from)
}

pub async fn find_member(ctx: &Context, msg: &Message, mut args: Args)
    -> serenity::Result<Member>
{
    if let Some(user) = msg.guild(ctx).await.ok_or(serenity::Error::Other("Guild!!?"))?
        .member_named(&args.single::<String>().unwrap())
    {
        ctx.http.get_member(user.guild_id.into(), user.user.id.into()).await
    } else {
        Err(serenity::Error::Other("Kim bu amk tanımıyorum."))
    }
}

