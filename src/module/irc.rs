use std::time::Duration;

use futures::prelude::*;
use irc::client::prelude::*;
use irc::client::ClientStream;
use serenity::client::Context;
use serenity::model::event::MessageUpdateEvent;
use strsim::normalized_damerau_levenshtein;
use tokio::sync::Mutex;

static IRC_CHANNEL: &str = "##hello_world";

lazy_static! {
    static ref IRC_CLIENT: Mutex<Option<Client>> = Mutex::new(None);
    static ref IRC_PASS: String = std::env::var("IRCPASS").expect("$IRCPASS");
}

async fn send_message(ctx: &Context, message: String) {
    let client = IRC_CLIENT.lock().await;
    let client = client.as_ref().expect("IRC Client");

    match client.send(Command::PRIVMSG(String::from(IRC_CHANNEL), message)) {
        Err(irc::error::Error::AsyncChannelClosed) => ready(ctx).await,
        Err(e) => println!("IRC Send PRIVMSG: {}", e),
        Ok(()) => (),
    };
}

pub async fn message(ctx: &Context, message: &serenity::model::channel::Message) {
    if message.channel_id != 589415209580625933 {
        return;
    }

    let attachments = message
        .attachments
        .iter()
        .map(|att| format!("<{}> ", att.proxy_url))
        .collect::<String>();

    let content = message.content_safe(ctx).await;
    let mut content = content.split('\n');

    send_message(
        ctx,
        format!(
            "[13{}00] {}{}",
            message.author.name,
            attachments,
            content.next().unwrap_or(&String::new())
        ),
    )
    .await;

    for subcontent in content {
        send_message(ctx, format!("[13{}00] {}", message.author.name, subcontent)).await;
    }
}

pub async fn message_update(
    ctx: &Context,
    old: Option<serenity::model::channel::Message>,
    new: Option<serenity::model::channel::Message>,
    _event: MessageUpdateEvent,
) {
    if let Some(mut new) = new {
        if let Some(old) = old {
            if normalized_damerau_levenshtein(&old.content, &new.content) > 0.98
                || new.content.contains(&old.content)
            {
                return;
            }
        }

        new.content += "~";
        message(ctx, &new).await;
    }
}

async fn authenticate(stream: &mut ClientStream) -> Result<(), irc::error::Error> {
    let mut irc_client = IRC_CLIENT.lock().await;
    let irc_client = irc_client.as_mut().expect("IRC Client");

    irc_client.send_cap_req(&[Capability::Sasl])?;

    irc_client.send(Command::PASS(IRC_PASS.to_string()))?;

    irc_client.send(Command::NICK("hwbot".to_string()))?;

    irc_client.send(Command::USER(
        "menfie".to_string(),
        "0".to_owned(),
        "menfie".to_string(),
    ))?;

    while let Some(message) = stream.next().await.transpose()? {
        match &message.command {
            Command::CAP(_, ref subcommand, _, _) => {
                if subcommand.to_str() == "ACK" {
                    println!("Recieved ack for sasl");
                    irc_client.send_sasl_plain()?;
                }
            }
            Command::AUTHENTICATE(_) => {
                println!("Got signal to continue authenticating");
                irc_client.send(Command::AUTHENTICATE(base64::encode(format!(
                    "{}\x00{}\x00{}",
                    "menfie",
                    "menfie",
                    *IRC_PASS
                ))))?;

                irc_client.send(Command::CAP(None, "END".parse().unwrap(), None, None))?;
            }
            Command::Response(code, _) => {
                if code == &Response::RPL_SASLSUCCESS {
                    println!("Successfully authenticated");
                    irc_client.send(Command::CAP(None, "END".parse().unwrap(), None, None))?;
                    break;
                }
            }
            Command::ERROR(err) => {
                println!("{}", err);
            }
            _ => {}
        }
    }

    println!("Authentication ended");
    Ok(())
}

pub async fn ready(ctx: &Context) {
    async fn new_client() -> ClientStream {
        if let Some(irc_client) = IRC_CLIENT.lock().await.as_ref() {
            irc_client.send_quit("Yeniden bağlanıyorum...").ok();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        let mut irc_client = Client::from_config(Config {
            nickname: Some(String::from("hwbot")),
            realname: Some(String::from("hwbot")),
            username: Some(String::from("menfie")),
            password: Some(IRC_PASS.to_string()),
            owners: vec![String::from("menfie")],
            server: Some(String::from("irc.libera.chat")),
            port: Some(6697),
            use_tls: Some(true),
            channels: vec![String::from(IRC_CHANNEL)],
            umodes: Some(String::from("+iR")),
            ..Config::default()
        })
        .await
        .expect("IRC Client");

        let stream = irc_client.stream().expect("IRC Stream");
        *IRC_CLIENT.lock().await = Some(irc_client);
        stream
    }

    let mut stream = new_client().await;

    while let Err(e) = authenticate(&mut stream).await {
        println!("Got an error while authenticating will retry: {}", e);
        tokio::time::sleep(Duration::from_secs(2)).await;
        stream = new_client().await;
    }

    let ctx = ctx.clone();
    tokio::spawn(async move {
        let hook = ctx
            .http
            .as_ref()
            .get_webhook(848577486744322069)
            .await
            .unwrap();
        while let Some(message) = stream.next().await.transpose().expect("IRC Message") {
            match &message.command {
                Command::ERROR(err) => {
                    println!("{}", err);
                }
                Command::PRIVMSG(chan, msg) => {
                    if chan != IRC_CHANNEL {
                        continue;
                    }

                    if let Some(sender) = message.source_nickname() {
                        if sender
                            == IRC_CLIENT
                                .lock()
                                .await
                                .as_ref()
                                .expect("IRC Client")
                                .current_nickname()
                        {
                            continue;
                        }

                        let msg_stripped =
                            msg.replace("everyone", "every0ne").replace("here", "her3");

                        hook.execute(&ctx, false, |w| w.content(msg_stripped).username(sender))
                            .await
                            .ok();
                    }
                }
                _ => {}
            }
        }
    });
}
