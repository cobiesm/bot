use serenity::framework::standard::{
    help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Context;
use std::collections::HashSet;

#[help]
#[strikethrough_commands_tip_in_guild = ""]
#[individual_command_tip = "hello_world'de kullanabileceğin komutların listesi:"]
#[no_help_available_text = "Böyle bir komut yok."]
#[command_not_found_text = "Böyle bir komut yok."]
#[available_text = "Sadece"]
#[guild_only_text = "Sunucu İçinde"]
#[grouped_label = "Grup"]
#[lacking_role = "hide"]
#[lacking_ownership = "hide"]
#[lacking_permissions = "hide"]
#[suggestion_text = "wtf"]
#[usage_label = "Kullanım"]
#[usage_sample_label = "Örnek"]
#[aliases_label = "Namıdiğer"]
#[description_label = "wtf4"]
#[checks_label = "wtf5"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
