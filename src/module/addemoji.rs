use base64::encode;
use serde_json::json;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::framework::standard::{
    CommandResult,
    CommandError,
    macros::command,
    Args,
    Delimiter,
};

static ERR_EMPTY: &str = "verdiğin linkte bi bok bulamadım.";
static ERR_INVLINK: &str = "böyle bir link yok belki de hiç olmadı.";
static ERR_BIG: &str = "yalnız hocam benim sınır 256kb.";
static ERR_NOTSUP: &str = "png ve gif dışındakiler tarzım değil.";
static ERR_SMNAME: &str = "hafız az daha uzun isim girebilicen mi.";

#[command]
#[num_args(2)]
#[description = "Emoji eklemek belki."]
#[example = "yarra https://yarra.me/yarra.gif"]
#[bucket = "addemoji"]
#[aliases(emoji,emojiekle)]
pub async fn addemoji(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
    let name = match args.advance().single::<String>() {
        Ok(name) if name.len() > 1 => name,
        Ok(_) => { return Err(ERR_SMNAME.into()) },
        Err(e) => { return Err(CommandError::from(e)) },
    };

    let image_url = match args.single::<String>() {
        Ok(emoji) => emoji,
        Err(e) => { return Err(CommandError::from(e)) },
    };

    let image_raw = match reqwest::get(&image_url).await {
        Ok(resp) =>  match resp.bytes().await {
            Ok(bytes) if bytes.len() <= 255_999 => bytes,
            Ok(_) => { return Err(ERR_BIG.into()) },
            Err(_) => { return Err(CommandError::from(ERR_EMPTY)) }
        },
        Err(_) => { return Err(CommandError::from(ERR_INVLINK)) }
    };

    let mut ext = String::from_utf8_lossy(&image_raw.slice(0..4).to_vec()).to_string();

    println!("{:?}", image_raw);

    ext = match ext.as_str() {
        "GIF8" => "gif".into(),
        "�PNG" => "png".into(),
        _ => { return Err(format!("{} ~~{}~~", ERR_NOTSUP, ext).into()); }
    };

    let emoji = json!({
        "name": name,
        "image": format!("data:image/{};base64,{}", ext, encode(image_raw))
    });

    ctx.http.create_emoji(msg.guild_id.unwrap().into(), &emoji).await
        .map_err(|e| { CommandError::from(e) }).map(|_| ())
}
