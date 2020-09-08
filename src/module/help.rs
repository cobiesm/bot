use std::collections::HashSet;
use serenity::model::id::UserId;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    Args,
    CommandGroup,
    CommandResult,
    HelpOptions,
    help_commands,
    macros::help
};

#[help]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
