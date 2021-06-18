use futures::prelude::*;
use irc::client::prelude::*;
use serenity::client::Context;
use tokio::sync::Mutex;

static IRC_CHANNEL: &str = "##hello_world";

lazy_static! {
    static ref IRC_CLIENT: Mutex<Option<Client>> = Mutex::new(None);
    static ref IRC_PASS: String = std::env::var("IRCPASS").expect("$IRCPASS");
}

async fn send_message(message: String) {
    let client = IRC_CLIENT.lock().await;
    let client = client.as_ref().expect("IRC Client");

    client
        .send(Command::PRIVMSG(String::from(IRC_CHANNEL), message))
        .expect("IRC Send PRIVMSG");
}

pub async fn message(ctx: &Context, message: &serenity::model::channel::Message) {
    if message.channel_id != 589415209580625933 || message.content.is_empty() {
        return;
    }

    let attachments = message
        .attachments
        .iter()
        .map(|att| format!("<{}> ", att.proxy_url))
        .collect::<String>();

    let content = message.content_safe(&ctx).await;
    let mut content = content.split('\n');

    send_message(format!(
        "[{}] {}{}",
        message.author.name,
        attachments,
        content.next().unwrap()
    ))
    .await;

    for subcontent in content {
        send_message(format!("[{}] {}", message.author.name, subcontent)).await;
    }
}

pub async fn ready(ctx: &Context) {
    let irc_client = Client::from_config(Config {
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
    irc_client
        .send_cap_req(&[Capability::Sasl])
        .expect("IRC Cap");
    irc_client.send(Command::PASS(IRC_PASS.to_string())).unwrap();
    irc_client.send(Command::NICK("hwbot".to_string())).unwrap();
    irc_client.send(Command::USER(
            "menfie".to_string(),
            "0".to_owned(),
            "menfie".to_string(),
            )).unwrap();

    *IRC_CLIENT.lock().await = Some(irc_client);

    let ctx = ctx.clone();
    tokio::spawn(async move {
        let mut stream;
        {
            let mut client = IRC_CLIENT.lock().await;
            let client = client.as_mut().expect("IRC Client");
            stream = client.stream().expect("IRC Stream");
        }

        let hook = ctx
            .http
            .as_ref()
            .get_webhook(848577486744322069)
            .await
            .unwrap();
        while let Some(message) = stream.next().await.transpose().expect("IRC Message") {
            match &message.command {
                Command::CAP(_, ref subcommand, _, _) => {
                    if subcommand.to_str() == "ACK" {
                        println!("Recieved ack for sasl");
                        IRC_CLIENT
                            .lock()
                            .await
                            .as_ref()
                            .expect("IRC Client")
                            .send_sasl_plain()
                            .expect("IRC SASL plain");
                    }
                },
                Command::AUTHENTICATE(_) => {
                    println!("Got signal to continue authenticating");
                    let client = IRC_CLIENT.lock().await;
                    let client = client.as_ref().expect("IRC Client");
                    client.send(Command::AUTHENTICATE(base64::encode(format!("{}\x00{}\x00{}", "menfie", "menfie", IRC_PASS.to_string())))).unwrap();
                    client.send(Command::CAP(None, "END".parse().unwrap(), None, None)).unwrap();
                },
                Command::Response(code, _) => {
                    if code == &Response::RPL_SASLSUCCESS {
                        println!("Successfully authenticated");
                        IRC_CLIENT
                            .lock()
                            .await
                            .as_ref()
                            .expect("IRC Client")
                            .send(Command::CAP(None, "END".parse().unwrap(), None, None))
                            .unwrap();
                    }
                },
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

                        hook.execute(&ctx, false, |w| w.content(msg).username(sender))
                            .await
                            .ok();
                    }
                }
                _ => {}
            }
        }
    });
}
