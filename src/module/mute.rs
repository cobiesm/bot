use crate::muteable::Muteable;

use chrono::Duration;
use futures::StreamExt;
use regex::Regex;
use serenity::{
    client::Context, framework::standard::macros::command, framework::standard::Args,
    framework::standard::CommandError, framework::standard::CommandResult, model::channel::Message,
    model::guild::Member,
};
use tokio::time::sleep;

static GUILD_ID: u64 = 589415209580625930;

lazy_static! {
    static ref DURATION: Regex = Regex::new(r"^(\d+)(\w?)$").unwrap();
}

#[command]
#[max_args(3)]
pub async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>()?;

    let time_str = if args.len() > 1 {
        args.single::<String>()?
    } else {
        String::new()
    };

    let reason = args.remains();

    find_member(ctx, msg, &name)
        .await?
        .mute(ctx.clone(), time_conv(&time_str), reason)
        .await
        .map_err(CommandError::from)
}

#[command]
#[max_args(2)]
pub async fn unmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>()?;
    let time_str = if args.len() > 1 {
        args.single::<String>()?
    } else {
        String::new()
    };

    find_member(ctx, msg, &name)
        .await?
        .unmute(ctx.clone(), time_conv(&time_str))
        .await
        .map_err(CommandError::from)
}

pub async fn ready(ctx: &Context) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::seconds(3).to_std().unwrap()).await;

            let members =
                if let Ok(members) = ctx.http.get_guild_members(GUILD_ID, Some(1000), None).await {
                    members
                } else {
                    continue;
                };

            for mut member in members {
                member.try_unmute(ctx.clone()).await;
            }
        }
    });
}

pub async fn find_member(ctx: &Context, msg: &Message, name: &str) -> serenity::Result<Member> {
    fn contains(substring: &str, name: &str) -> bool {
        name.to_lowercase().contains(&substring.to_lowercase())
    }

    let members = msg
        .guild_id
        .ok_or(serenity::Error::Other("guild!!?"))?
        .members_iter(ctx)
        .filter_map(|member| async move {
            if member.is_err() {
                return None;
            }

            let member = member.unwrap();

            let username = &member.user.name;
            if contains(name, username) {
                Some(member)
            } else {
                member.nick.clone().and_then(|nick| {
                    if contains(name, &nick) {
                        Some(member)
                    } else {
                        None
                    }
                })
            }
        })
        .collect::<Vec<Member>>()
        .await;

    Ok(members
        .first()
        .ok_or(serenity::Error::Other("kim bu amk tanımıyorum."))?
        .clone())
}

fn time_conv(time_str: &str) -> Option<Duration> {
    DURATION.captures(time_str).and_then(|caps| {
        caps.get(1)
            .map(|cap| cap.as_str().parse::<i64>())
            .and_then(Result::ok)
            .map(|time| {
                match caps
                    .get(2)
                    .map(|cap| cap.as_str().to_lowercase())
                    .as_deref()
                {
                    Some("d") => Duration::days(time),
                    Some("h") => Duration::hours(time),
                    Some("s") => Duration::seconds(time),
                    _ => Duration::minutes(time),
                }
            })
    })
}
