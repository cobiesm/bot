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
    static ref EH_ISTE: RoleId = RoleId{ 0: 763769069605224458 };
    static ref ACE: RoleId = RoleId{ 0: 664070917801902093 };
    static ref NULL: RoleId = RoleId{ 0: 717039238423642242 };
    static ref GUILD_ID: u64 = 589415209580625930;
    static ref LEVEL_FINDER: Regex = Regex::new(r"\^(\d+\.\d+)$").unwrap();
    static ref LEVELS: Mutex<HashMap<u64, Arc<Mutex<WithLevel>>>> = Mutex::new(HashMap::new());
    static ref ROLES: HashMap<u64, f64> = {
        let mut roles = HashMap::new();
        roles.insert(ACE.0, 6.0); // ACE
        roles.insert(NULL.0, 100.0); // null
        roles
    };
}

pub async fn ready(ctx: &Context) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        loop {
            let mut users_to_clean: Vec<u64> = vec![];
            for member in LEVELS.lock().await.values() {
                let mut member = member.lock().await;
                let mut roles_to_add: Vec<RoleId> = vec![];
                let mut roles_to_del: Vec<RoleId> = vec![];
                for (role, xp_req) in ROLES.iter() {
                    let role = RoleId { 0: *role };
                    if member.xp() >= *xp_req && !member.base.roles.contains(&role) {
                        roles_to_add.push(role);
                    } else if member.xp() < *xp_req && member.base.roles.contains(&role) {
                        roles_to_del.push(role);
                    }
                }

                if !roles_to_add.is_empty() {
                    member
                        .base
                        .add_roles(&ctx, &roles_to_add)
                        .await
                        .expect("Couldn't add roles");
                } else if member.base.roles.contains(&*EH_ISTE)
                    && (member.xp() < 3.0 || member.base.roles.contains(&*ACE))
                {
                    member
                        .base
                        .remove_role(&ctx, *EH_ISTE)
                        .await
                        .expect("Couldn't del role");
                } else if member.xp() >= 3.0 && !member.base.roles.contains(&*ACE) {
                    member
                        .base
                        .add_role(&ctx, *EH_ISTE)
                        .await
                        .expect("Couldn't add role");
                }

                if !roles_to_del.is_empty() {
                    member
                        .base
                        .remove_roles(&ctx, &roles_to_del)
                        .await
                        .expect("Couldn't del roles");
                }

                if member.enough_passed().await {
                    users_to_clean.push(*member.base.user.id.as_u64());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            for user in users_to_clean {
                LEVELS.lock().await.remove(&user);
            }
            std::thread::sleep(std::time::Duration::from_secs(15));
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
        find_member(member)
            .await
            .lock()
            .await
            .xp_give(ctx.http.clone(), 2.0)
            .await;
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
        find_member(member)
            .await
            .lock()
            .await
            .xp_take(ctx.http.clone(), 2.0)
            .await;
    }
}

pub async fn message(ctx: &Context, msg: &Message) {
    if let Ok(member) = msg.member(ctx).await {
        find_member(member)
            .await
            .lock()
            .await
            .xp_give(ctx.http.clone(), 0.1)
            .await;
    }
}

pub async fn message_delete(ctx: &Context, _channel_id: ChannelId, message: Message) {
    if let Ok(member) = message.member(ctx).await {
        find_member(member)
            .await
            .lock()
            .await
            .xp_take(ctx.http.clone(), 1.5)
            .await;
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
                    find_member(member)
                        .await
                        .lock()
                        .await
                        .xp_take(ctx.http.clone(), 1.0)
                        .await;
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
    last: Option<DateTime<Utc>>,
}

impl WithLevel {
    const fn new(base: Member) -> Self {
        Self { base, last: None }
    }

    async fn enough_passed(&self) -> bool {
        self.last.map_or(true, |last_give| {
            Utc::now() - last_give > Duration::seconds(5)
        })
    }

    async fn update(&mut self, http: Arc<Http>) {
        self.base = http
            .get_member(*self.base.guild_id.as_u64(), *self.base.user.id.as_u64())
            .await
            .unwrap();
    }

    fn xp(&self) -> f64 {
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

    #[async_recursion]
    async fn xp_give(&mut self, http: Arc<Http>, amount: f64) {
        if amount.is_sign_positive() && !self.enough_passed().await {
            self.xp_take(http, 0.02).await;
            return;
        }

        self.update(http.clone()).await;

        let mut name = String::new();
        name.push_str(self.base.display_name().as_str());

        if !LEVEL_FINDER.is_match(name.as_str()) {
            name.push_str(" ^0.0");
        }

        let mut xp_new = (self.xp() + amount).max(0.0);

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

        let time_diff = Local::now().timestamp_millis() - last_millis;
        let four_hours = 14400000;

        if time_diff > four_hours / 8 {
            xp_new = (xp_new - time_diff as f64 / four_hours as f64).max(0.0);
        }

        let name = LEVEL_FINDER.replace(
            name.as_str(),
            if xp_new > 0.0 {
                format!("^{:.2}", xp_new)
            } else {
                String::new()
            }
            .as_str(),
        );

        self.base.edit(http, |e| e.nickname(name)).await.ok();

        if amount.is_sign_positive() {
            self.last = Some(Utc::now());
        }
    }

    async fn xp_take(&mut self, http: Arc<Http>, amount: f64) {
        self.xp_give(http, -amount).await;
    }
}
