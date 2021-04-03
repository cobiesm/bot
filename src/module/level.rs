use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::Utc;
use regex::Regex;
use serenity::{
    client::{Cache, Context},
    http::Http,
    model::{
        channel::{Message, Reaction},
        event::MessageUpdateEvent,
        guild::Member,
        id::{ChannelId, RoleId, UserId},
    },
};
use tokio::sync::Mutex;

use super::undelete::is_deleted;

static COOLDOWN_SPAM: i64 = 5000;
static _COOLDOWN_AFK: i64 = 14400000;

static GIVE_MESSAGE: f64 = 0.05;
static TAKE_MESSAGE: f64 = 0.01;
static GIVE_REACTION: f64 = 2.0;
static TAKE_DELETE: f64 = 1.5;
static TAKE_EDIT: f64 = 1.0;

lazy_static! {
    static ref EH_ISTE: RoleId = RoleId(763769069605224458);
    static ref ACE: RoleId = RoleId(664070917801902093);
    static ref NULL: RoleId = RoleId(717039238423642242);
    static ref GUILD_ID: u64 = 589415209580625930;
    static ref LEVEL_FINDER: Regex = Regex::new(r"\s\^(\d+\.\d+)").unwrap();
    static ref ROLES: HashMap<u64, (f64, bool)> = {
        let mut roles = HashMap::new();
        roles.insert(EH_ISTE.0, (5.0, true));
        roles.insert(ACE.0, (20.0, false));
        roles.insert(NULL.0, (100.0, false));
        roles
    };
    static ref TIMES: Mutex<HashMap<u64, i64>> = Mutex::new(HashMap::new());
    static ref LOCKS: Mutex<HashMap<u64, Arc<Mutex<MemberWithLevel>>>> = Mutex::new(HashMap::new());
}

pub async fn ready(ctx: &Context) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        loop {
            let members = if let Ok(members) = ctx
                .http
                .get_guild_members(*GUILD_ID, Some(1000), None)
                .await
            {
                members
            } else {
                continue;
            }; // I was wrong. This shouldn't fail or the bot will go on without this loop.

            for mut member in members {
                let lmember = MemberWithLevel {
                    member: member.clone(),
                    xp: None,
                };

                // TODO: This is being repeated until member becomes unafk but until then we take
                // all their XPs.
                // ------------------------------------------------------------------------------
                //let time_diff = lmember.ms_after_last_real_message(&ctx).await;
                //if time_diff > COOLDOWN_AFK / 8 {
                //let lock = find_member(&ctx, member.user.id).await;
                //let mut lmember = lock.lock().await;
                //lmember
                //.xp_take(&ctx, time_diff as f64 / COOLDOWN_AFK as f64)
                //.await;
                //}

                let mroles = member.roles.clone();

                let add = ROLES
                    .iter()
                    .filter_map(|role| {
                        let xp_req = role.1 .0;
                        let alone = role.1 .1;
                        let role = RoleId { 0: *role.0 };
                        if !mroles.contains(&role)
                            && lmember.xp_current() >= xp_req
                            && (!alone || !mroles.contains(&*ACE))
                        {
                            Some(role)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<RoleId>>();
                if !add.is_empty() {
                    member.add_roles(&ctx, &add).await.expect("add_roles");
                }

                let del = ROLES
                    .iter()
                    .filter_map(|role| {
                        let xp_req = role.1 .0;
                        let alone = role.1 .1;
                        let role = RoleId { 0: *role.0 };
                        if mroles.contains(&role)
                            && (lmember.xp_current() < xp_req || (alone && mroles.contains(&*ACE)))
                        {
                            Some(role)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<RoleId>>();
                if !del.is_empty() {
                    member.remove_roles(&ctx, &del).await.expect("rm_roles");
                }

                let uid = member.user.id.as_u64();
                if LOCKS.lock().await.contains_key(uid) {
                    let lock = find_member(&ctx, member.user.id).await;
                    let lmember = lock.lock().await;
                    if lmember.enough_passed().await {
                        LOCKS.lock().await.remove(uid);
                        TIMES.lock().await.remove(uid);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}

pub async fn message(ctx: &Context, msg: &Message) {
    let lock = find_member(ctx, msg.author.id).await;
    let mut lmember = lock.lock().await;
    if lmember.enough_passed().await {
        lmember.xp_give(ctx, GIVE_MESSAGE).await;
    } else {
        lmember.xp_take(ctx, TAKE_MESSAGE).await;
    }
    TIMES
        .lock()
        .await
        .insert(*msg.author.id.as_u64(), Utc::now().timestamp_millis());
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = match reaction.message(ctx).await {
        Ok(m) => m,
        Err(_) => return,
    };

    if reaction.emoji != *super::clap::CLAP
        || reaction.user_id.expect("Reaction User") == message.author.id
    {
        // clap.rs already deletes the clap
        return;
    }

    let lock = find_member(ctx, message.author.id).await;
    let mut lmember = lock.lock().await;
    lmember.xp_give(ctx, GIVE_REACTION).await;
}

pub async fn reaction_remove(ctx: &Context, reaction: &Reaction) {
    let message = match reaction.message(ctx).await {
        Ok(m) => m,
        Err(_) => return,
    };

    if reaction.emoji != *super::clap::CLAP
        || reaction.user_id.expect("Reaction User") == message.author.id
    {
        return;
    }

    let lock = find_member(ctx, message.author.id).await;
    let mut lmember = lock.lock().await;
    lmember.xp_take(ctx, GIVE_REACTION).await;
}

pub async fn message_delete(ctx: &Context, _channel_id: ChannelId, message: Message) {
    let lock = find_member(ctx, message.author.id).await;
    let mut lmember = lock.lock().await;
    lmember.xp_take(ctx, TAKE_DELETE).await;
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
                let lock = find_member(ctx, new.author.id).await;
                let mut lmember = lock.lock().await;
                lmember.xp_take(ctx, TAKE_EDIT).await;
            }
        }
    }
}

async fn find_member<T: AsRef<Http> + Sync + Send>(
    http: T,
    user_id: UserId,
) -> Arc<Mutex<MemberWithLevel>> {
    let mut locks = LOCKS.lock().await;

    #[allow(clippy::option_if_let_else)]
    if let Some(lock) = locks.get(user_id.as_u64()) {
        lock.clone()
    } else {
        locks.insert(
            *user_id.as_u64(),
            Arc::new(Mutex::new(MemberWithLevel {
                member: http
                    .as_ref()
                    .get_member(*GUILD_ID, *user_id.as_u64())
                    .await
                    .expect("member"),
                xp: None,
            })),
        );
        locks.get(user_id.as_u64()).unwrap().clone()
    }
}

struct MemberWithLevel {
    member: Member,
    xp: Option<f64>,
}

impl MemberWithLevel {
    async fn ms_after_last_message(&self) -> i64 {
        let times = TIMES.lock().await;
        let last_millis = times.get(self.member.user.id.as_u64()).unwrap_or(&0);

        Utc::now().timestamp_millis() - last_millis
    }

    async fn enough_passed(&self) -> bool {
        self.ms_after_last_message().await > COOLDOWN_SPAM
    }

    fn xp_current(&self) -> f64 {
        self.xp.map_or_else(
            || {
                LEVEL_FINDER
                    .captures(self.member.display_name().as_str())
                    .map_or(0.0, |captures| {
                        captures.get(1).unwrap().as_str().parse::<f64>().unwrap()
                    })
            },
            |xp| xp,
        )
    }

    async fn xp_give<T: AsRef<Http> + Sync + Send>(&mut self, http: T, amount: f64) {
        if (0.0..0.005).contains(&amount) {
            return;
        }

        #[cfg(debug_assertions)]
        println!("Giving {} XP to {}.", amount, &self.member.distinct());

        let mut name = self.member.display_name().as_ref().clone();

        if !LEVEL_FINDER.is_match(&name) {
            name.push_str(" ^0.0");
        }

        let mut xp_new = self.xp_current() + amount;

        xp_new = xp_new.max(0.0);

        let name = LEVEL_FINDER.replace(
            &name,
            if xp_new > 0.0 {
                format!(" ^{:.2}", xp_new)
            } else {
                String::new()
            }
            .as_str(),
        );

        self.member.edit(http, |e| e.nickname(name)).await.ok();
        self.xp = Some(xp_new);
    }

    async fn xp_take<T: AsRef<Http> + AsRef<Cache> + Sync + Send>(
        &mut self,
        cache_http: T,
        amount: f64,
    ) {
        self.xp_give(cache_http, -amount).await;
    }

    async fn _ms_after_last_real_message<T: AsRef<Http> + AsRef<Cache> + Sync + Send>(
        &self,
        cache_http: T,
    ) -> i64 {
        let mut last_millis = 0;

        for val in self
            .member
            .guild_id
            .channels(&cache_http)
            .await
            .unwrap()
            .values()
        {
            if let Ok(mes) = val.messages(&cache_http, |buil| buil.limit(1)).await {
                if let Some(mes) = mes.first() {
                    last_millis = last_millis.max(mes.timestamp.timestamp_millis());
                }
            }
        }

        Utc::now().timestamp_millis() - last_millis
    }
}
