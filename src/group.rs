use crate::module::addemoji::ADDEMOJI_COMMAND;
use crate::module::mute::{MUTE_COMMAND, UNMUTE_COMMAND};
use crate::module::purge::PURGE_COMMAND;
use crate::module::uwu::UWU_COMMAND;
use crate::module::yarra::YARRA_COMMAND;
use serenity::framework::standard::macros::group;

#[group]
#[commands(purge, mute, unmute)]
#[allowed_roles("🔑")]
#[only_in(guilds)]
pub struct Admin;

#[group]
#[commands(yarra, uwu)]
#[only_in(guilds)]
pub struct Fun;

#[group]
#[commands(addemoji)]
#[allowed_roles("ACE")]
#[only_in(guilds)]
pub struct Ace;
