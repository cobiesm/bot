use serenity::client::Context;
use serenity::model::channel::{ Message, Reaction };
use serenity::prelude::EventHandler;
use crate::module::blacklink;
use crate::module::badword;
use crate::module::selfmod;
use crate::module::slowmode;

pub struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, new_message: Message) {
        blacklink::message(&ctx, &new_message);
        badword::message(&ctx, &new_message);
        slowmode::message(&ctx, &new_message);
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        selfmod::reaction_add(&ctx, &reaction);
    }
}
