use serenity::client::Context;
use serenity::http::Http;
use serenity::model::channel::{Message, Reaction, ReactionType};
use tokio::sync::Mutex;

static Q_CHANNELID: u64 = 670984869941346304;

lazy_static! {
    static ref TASKS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
    pub static ref CLAP: ReactionType = ReactionType::Unicode("👏".into());
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = match reaction.message(ctx).await {
        Ok(m) => m,
        Err(_) => return,
    };

    if reaction.emoji != *CLAP {
        return;
    } else if reaction.user_id == Some(message.author.id) {
        reaction.delete(ctx).await.ok();
        return;
    }

    if calc_reactions(ctx, &message).await == 3 {
        for task in TASKS.lock().await.iter() {
            if task == message.id.as_u64() {
                return;
            }
        }
    } else {
        return;
    }

    message.channel_id.broadcast_typing(ctx).await.ok();

    let http = ctx.http.clone();

    tokio::spawn(async move {
        let mut tasks = TASKS.lock().await;
        tasks.push(*message.id.as_u64());
        drop(tasks);
        tokio::time::delay_for(std::time::Duration::from_secs(5 * 60)).await;
        let reactions = calc_reactions(&http, &message).await;

        let channel = if let Ok(channel) = http.get_channel(Q_CHANNELID).await {
            channel.guild().unwrap() // SAFETY: Handler assures that the message is from a guild
        } else {
            eprintln!("Couldn't get the channel with its id.");
            return;
        };

        let author = message.author.clone();

        channel
            .send_message(http, |m| {
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
            })
            .await
            .ok();
        let mut tasks = TASKS.lock().await;
        match tasks
            .iter()
            .position(|message_id| message_id == message.id.as_u64())
        {
            Some(task) => {
                tasks.remove(task);
            }
            None => eprintln!("Couldn't find the task, somethings worng."),
        };
    });
}

async fn calc_reactions(http: impl AsRef<Http> + Send + Sync, message: &Message) -> usize {
    message
        .reaction_users(http, CLAP.clone(), None, None)
        .await
        .unwrap_or_else(|_| Vec::new())
        .len()
}
