use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use chrono::{prelude::*, Duration};
use regex::Regex;
use serenity::{
    client::Context,
    http::Http,
    model::{
        channel::{Message, Reaction},
        event::MessageUpdateEvent,
        guild::Member,
        id::{ChannelId, RoleId},
    },
};
use tokio::sync::Mutex;

use super::undelete::is_deleted;

lazy_static! {
    static ref EH_ISTE: RoleId = RoleId {
        0: 763769069605224458
    };
    static ref ACE: RoleId = RoleId {
        0: 664070917801902093
    };
    static ref NULL: RoleId = RoleId {
        0: 717039238423642242
    };
    static ref GUILD_ID: u64 = 589415209580625930;
    static ref LEVEL_FINDER: Regex = Regex::new(r"\^(\d+\.\d+)$").unwrap();
    static ref LEVELS: Mutex<HashMap<u64, Arc<Mutex<WithLevel>>>> = Mutex::new(HashMap::new());
    static ref ROLES: HashMap<u64, (f64, bool)> = {
        let mut roles = HashMap::new();
        roles.insert(EH_ISTE.0, (5.0, true));
        roles.insert(ACE.0, (20.0, false));
        roles.insert(NULL.0, (100.0, false));
        roles
    };
}

pub async fn ready(ctx: &Context) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        loop {
            let mut levels = LEVELS.lock().await;
            for member in levels.values() {
                let mut member = member.lock().await;
                member.xp_push(ctx.http.clone()).await;

                let mut mbase = member.base.clone();
                let mroles = mbase.roles.clone();

                for _ in 0..2 {
                    let add = ROLES
                        .iter()
                        .filter_map(|role| {
                            let xp_req = role.1 .0;
                            let alone = role.1 .1;
                            let role = RoleId { 0: *role.0 };
                            if !mroles.contains(&role)
                                && member.current_xp() >= xp_req
                                && (!alone || !mroles.contains(&*ACE))
                            {
                                Some(RoleId { 0: role.0 })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<RoleId>>();
                    mbase.add_roles(&ctx, &add).await.expect("add_roles");

                    let del = ROLES
                        .iter()
                        .filter_map(|role| {
                            let xp_req = role.1 .0;
                            let alone = role.1 .1;
                            let role = RoleId { 0: *role.0 };
                            if mroles.contains(&role)
                                && (member.current_xp() < xp_req
                                    || (alone && mroles.contains(&*ACE)))
                            {
                                Some(RoleId { 0: role.0 })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<RoleId>>();
                    mbase.remove_roles(&ctx, &del).await.expect("rm_roles");
                }
            }

            levels.clear();
            drop(levels);
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = match reaction.message(ctx).await {
        Ok(m) => m,
        Err(_) => return,
    };

    if reaction.emoji != *super::clap::CLAP || reaction.user_id.unwrap() == message.author.id {
        // clap.rs already deletes the clap
        return;
    }

    if let Ok(member) = ctx
        .http
        .get_member(
            *reaction.guild_id.unwrap().as_u64(),
            *message.author.id.as_u64(),
        )
        .await
    {
        find_member(member).await.lock().await.xp_give(2.0);
    }
}

pub async fn reaction_remove(ctx: &Context, reaction: &Reaction) {
    let message = match reaction.message(ctx).await {
        Ok(m) => m,
        Err(_) => return,
    };

    if reaction.emoji != *super::clap::CLAP || reaction.user_id.unwrap() == message.author.id {
        return;
    }

    if let Ok(member) = ctx
        .http
        .get_member(
            *reaction.guild_id.unwrap().as_u64(),
            *message.author.id.as_u64(),
        )
        .await
    {
        find_member(member).await.lock().await.xp_take(2.0);
    }
}

pub async fn message(ctx: &Context, msg: &Message) {
    if let Ok(member) = msg.member(ctx).await {
        let with_level = find_member(member).await;
        let mut with_level = with_level.lock().await;
        if with_level.enough_passed(ctx.http.clone()).await {
            with_level.xp_give(0.05);
        } else {
            with_level.xp_take(0.015);
        }
    }
}

pub async fn message_delete(ctx: &Context, _channel_id: ChannelId, message: Message) {
    if let Ok(member) = message.member(ctx).await {
        find_member(member).await.lock().await.xp_take(1.5);
    }
}

pub async fn message_update(
    ctx: &Context,
    old: Option<Message>,
    new: Option<Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(old) = old {
        if let Some(new) = new {
            if is_deleted(&old, &new) {
                if let Ok(member) = new.member(ctx).await {
                    find_member(member).await.lock().await.xp_take(1.0);
                }
            }
        }
    }
}

#[async_recursion]
async fn find_member(base: Member) -> Arc<Mutex<WithLevel>> {
    let mut levels = LEVELS.lock().await;
    if let Some(member) = levels.clone().get(base.user.id.as_u64()) {
        drop(levels);
        member.clone()
    } else {
        levels.insert(
            *base.user.id.as_u64(),
            Arc::new(Mutex::new(WithLevel::new(base.clone()))),
        );
        drop(levels);
        find_member(base).await
    }
}

struct WithLevel {
    base: Member,
    xp_to_give: f64,
}

impl WithLevel {
    const fn new(base: Member) -> Self {
        Self {
            base,
            xp_to_give: 0.0,
        }
    }

    async fn time_after_last_message(&self, http: Arc<Http>) -> i64 {
        let mut last_millis = 0;

        for val in self.base.guild_id.channels(&http).await.unwrap().values() {
            if let Ok(mes) = val.messages(&http, |buil| buil.limit(1)).await {
                if let Some(mes) = mes.first() {
                    let millis = mes.timestamp.timestamp_millis();
                    if millis.max(last_millis) == millis {
                        last_millis = mes.timestamp.timestamp_millis();
                    }
                }
            };
        }

        Utc::now().timestamp_millis() - last_millis
    }

    async fn enough_passed(&self, http: Arc<Http>) -> bool {
        self.time_after_last_message(http).await > Duration::seconds(5).num_milliseconds()
    }

    async fn update(&mut self, http: Arc<Http>) {
        self.base = http
            .get_member(*self.base.guild_id.as_u64(), *self.base.user.id.as_u64())
            .await
            .unwrap();
    }

    fn current_xp(&self) -> f64 {
        if LEVEL_FINDER.is_match(self.base.display_name().as_str()) {
            LEVEL_FINDER
                .captures(self.base.display_name().as_str())
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap()
        } else {
            0.0
        }
    }

    async fn xp_push(&mut self, http: Arc<Http>) {
        #[cfg(debug_assertions)]
        println!(
            "Giving {} XP to {}.",
            self.xp_to_give,
            &self.base.distinct(),
        );

        self.update(http.clone()).await;

        let mut name = String::new();
        name.push_str(self.base.display_name().as_str());

        if !LEVEL_FINDER.is_match(name.as_str()) {
            name.push_str(" ^0.0");
        }

        let mut xp_new = (self.current_xp() + self.xp_to_give).max(0.0);

        let time_diff = self.time_after_last_message(http.clone()).await;
        let four_hours = 14400000;

        if time_diff > four_hours / 8 {
            xp_new = (xp_new - time_diff as f64 / four_hours as f64).max(0.0);
        }

        let name = LEVEL_FINDER.replace(
            &name,
            if xp_new > 0.0 {
                format!("^{:.2}", xp_new)
            } else {
                String::new()
            }
            .as_str(),
        );

        self.base.edit(http, |e| e.nickname(name)).await.ok();
        self.xp_to_give = 0.0;
    }

    fn xp_give(&mut self, amount: f64) {
        self.xp_to_give += amount;
    }

    fn xp_take(&mut self, amount: f64) {
        self.xp_give(-amount);
    }
}
