use std::thread::sleep;
use std::time::Duration;
use serenity::http::Http;
use serenity::client::Context;
use serenity::model::channel::{ Reaction, ReactionType, Message };

const Q_CHANNELID: u64 = 670984869941346304;

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let message = reaction.message(ctx).await.unwrap();

    if reaction.user_id.unwrap() == message.author.id {
        reaction.delete(ctx).await.ok();
        return;
    }

    let reactions = calc_reactions(ctx, &message).await;

    if reactions <= 4 {
        return;
    }

    message.channel_id.broadcast_typing(ctx).await.unwrap();

    let http = ctx.http.clone();

    tokio::spawn(async move {
        sleep(Duration::from_secs(1200));
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
    });
}

async fn calc_reactions(http: impl AsRef<Http>, message: &Message) -> usize {
    let reactions = message.reaction_users(
        &http, ReactionType::Unicode("👏".into()), None, None
    ).await.unwrap();

    reactions.len()
}
