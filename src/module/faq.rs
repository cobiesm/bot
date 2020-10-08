use std::time::Duration;

use regex::Regex;
use serenity::model::channel::Message;
use serenity::{client::Context, model::guild::Member, model::id::ChannelId};

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
    if new_message.author.bot {
        return;
    }

    for faq in FAQS.iter() {
        if faq.expected.is_match(&new_message.content)
            && (!faq.mentions
                || new_message.mentions_user(&ctx.http.get_current_user().await.unwrap().into()))
        {
            let outcome = &faq.outcome;
            let member = new_message.member(ctx).await.unwrap().clone();
            let channel = new_message.channel_id;
            if !outcome.contains(';') {
                reply(ctx, &member, channel, &faq.outcome).await;
                break;
            }

            let answers = outcome.split(';');
            let ctx = ctx.clone();
            tokio::spawn(async move {
                let mut message = reply(
                    &ctx,
                    &member,
                    channel,
                    answers.clone().rev().last().unwrap(),
                )
                .await;
                for answer in answers.skip(1) {
                    channel.broadcast_typing(&ctx).await.ok();
                    std::thread::sleep(Duration::from_secs(
                        rand::random::<f64>().mul_add(1.1, 1.0) as u64,
                    ));
                    message
                        .edit(&ctx, |b| b.content(format!("{}, {}", member, answer)))
                        .await
                        .ok();
                }
            });
        }
    }
}

pub async fn reply(ctx: &Context, member: &Member, channel: ChannelId, answer: &str) -> Message {
    channel
        .send_message(ctx, |b| b.content(format!("{}, {}", member, answer)))
        .await
        .unwrap()
}
