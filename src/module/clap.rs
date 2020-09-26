use std::thread::sleep;
use std::collections::HashMap;
use tokio::sync::Mutex;
use serenity::http::Http;
use serenity::client::Context;
use serenity::model::channel::{ Reaction, ReactionType, Message };

static Q_CHANNELID: u64 = 670984869941346304;

lazy_static!(
    static ref TASKS: Mutex<HashMap<u64, bool>> = Mutex::new(HashMap::new());
    static ref CLAP: ReactionType = ReactionType::Unicode("👏".into());
);

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = reaction.message(ctx).await.unwrap();

    if reaction.emoji != *CLAP {
        return;
    } else if reaction.user_id.unwrap() == message.author.id {
        reaction.delete(ctx).await.ok();
        return;
    }

    let reactions = calc_reactions(ctx, &message).await;

    if reactions == 5 {
        for task in TASKS.lock().await.iter() {
            if task.0 == message.id.as_u64() && *task.1 {
                return;
            }
        }
    } else {
        return;
    }

    message.channel_id.broadcast_typing(ctx).await.unwrap();

    let http = ctx.http.clone();

    tokio::spawn(async move {
        let mut tasks = TASKS.lock().await;
        tasks.insert(*message.id.as_u64(), true);
        sleep(chrono::Duration::minutes(5).to_std().unwrap());
        let reactions = calc_reactions(&http, &message).await;

        let channel = http.get_channel(Q_CHANNELID).await.unwrap()
            .guild().unwrap();
        let author = message.author.clone();

        channel.send_message(http, |m| {
            m.embed(|em| {
                em.description(&message.content)
                    .author(|em_aut| {
                        let mut em_aut = em_aut.name(author.name.clone());
                        if let Some(image) = author.avatar_url() {
                            em_aut = em_aut.icon_url(image);
                        }
                        em_aut.url(message.link())
                    })
                    .timestamp(&message.timestamp)
                    .footer(|foot| foot.text(format!("{} 👏", reactions)))
            })
        }).await.ok();
        tasks.insert(*message.id.as_u64(), false);
    });
}

async fn calc_reactions(http: impl AsRef<Http> + Send + Sync, message: &Message) -> usize {
    message.reaction_users(
        http, CLAP.clone(), None, None
    ).await.unwrap().len()
}
