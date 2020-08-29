use serenity::framework::standard::macros::group;
use crate::module::purge::PURGE_COMMAND;

#[group]
#[commands(purge)]
#[allowed_roles("🔑")]
#[only_in(guilds)]
pub struct Admin;
