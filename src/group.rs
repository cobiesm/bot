use serenity::framework::standard::macros::group;
use crate::module::purge::*;

#[group]
#[commands(purge)]
#[allowed_roles("🔑")]
#[only_in(guilds)]
pub(crate) struct Admin;
