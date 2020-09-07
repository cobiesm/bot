use regex::Regex;
use serenity::client::Context;
use serenity::model::channel::Message;

lazy_static!(
    static ref FAQS: Vec<Faq> = vec![
        Faq {
            expected: Regex::new(r"(?i)bilen *var *m(ı|i)").unwrap(),
            outcome: String::from("bilen var mı diye sormak yerine derdini anlatman \
                                  daha yararlı olur."),
            mentions: false
        },
        Faq {
            expected: Regex::new(r"(?i)good *bot").unwrap(),
            outcome: String::from("saol cnm <3"),
            mentions: true,
        },
    ];
);

struct Faq {
    expected: Regex,
    outcome: String,
    mentions: bool,
}

pub fn message(ctx: &Context, new_message: &Message) {
    if new_message.author.bot {
        return
    }

    for faq in FAQS.iter() {
        if faq.expected.is_match(&new_message.content) && (!faq.mentions
            || new_message.mentions_user(&ctx.http.get_current_user().unwrap().into()))
        {
            new_message.reply(ctx, &faq.outcome).ok();
        }
    }
}
