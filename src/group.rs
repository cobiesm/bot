use serenity::framework::standard::macros::group;
use crate::module::purge::PURGE_COMMAND;
use crate::module::yarra::YARRA_COMMAND;

#[group]
#[commands(purge)]
#[allowed_roles("🔑")]
#[only_in(guilds)]
pub struct Admin;

#[group]
#[commands(yarra)]
#[only_in(guilds)]
pub struct Fun;
