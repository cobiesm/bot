use std::time::Duration;

use regex::Regex;
use serenity::{client::Context, model::guild::Member, model::id::ChannelId};
use serenity::{model::channel::Message, Error};

lazy_static! {
    static ref FAQS: Vec<Faq> = vec![
        Faq {
            expected: Regex::new(r"(?i)bad *bot").unwrap(),
            outcome: String::from("ku...;kusur...;kusura bakma (TT_TT)"),
            mentions: true,
        },
        Faq {
            expected: Regex::new(r"(?i)good *bot").unwrap(),
            outcome: String::from("saol cnm ^ω^"),
            mentions: true,
        },
        Faq {
            expected: Regex::new(r"(?i)(ora*da|bura*da) *m(ı|i)s(ı|i)n").unwrap(),
            outcome: String::from("burdayım amk nerde olucam??!"),
            mentions: true,
        },
    ];
}

struct Faq {
    expected: Regex,
    outcome: String,
    mentions: bool,
}

pub async fn message(ctx: &Context, new_message: &Message) {
    for faq in FAQS.iter() {
        if faq.expected.is_match(&new_message.content)
            && (!faq.mentions
                || new_message.mentions_user(
                    &ctx.http
                        .get_current_user()
                        .await
                        .unwrap() // SAFETY: Current user should always be available.
                        .into(),
                ))
        {
            new_message.channel_id.broadcast_typing(ctx).await.ok();
            let outcome = &faq.outcome;
            let member = match new_message.member(ctx).await {
                Ok(member) => member,
                Err(e) => {
                    eprintln!("Couldn't get member of the message because {}.", e);
                    return;
                }
            };
            let channel = new_message.channel_id;
            if !outcome.contains(';') {
                reply(ctx, &member, channel, &faq.outcome).await.ok();
                break;
            }

            let ctx = ctx.clone();
            tokio::spawn(async move {
                let answers = outcome.split(';');
                let mut message = match reply(
                    &ctx,
                    &member,
                    channel,
                    answers.clone().rev().last().unwrap(), // SAFETY: First element always exist
                )
                .await
                {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("Couldn't reply because {}.", e);
                        return;
                    }
                };
                for answer in answers.skip(1) {
                    channel.broadcast_typing(&ctx).await.ok();
                    tokio::time::sleep(Duration::from_secs(
                        rand::random::<f64>().mul_add(1.1, 1.0) as u64,
                    ))
                    .await;
                    message
                        .edit(&ctx, |b| b.content(format!("{}, {}", member, answer)))
                        .await
                        .ok();
                }
            });
        }
    }
}

pub async fn reply(
    ctx: &Context,
    member: &Member,
    channel: ChannelId,
    answer: &str,
) -> Result<Message, Error> {
    channel
        .send_message(ctx, |b| b.content(format!("{}, {}", member, answer)))
        .await
}
