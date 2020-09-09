use serenity::client::Context;
use serenity::model::user::OnlineStatus;
use serenity::model::gateway::Activity;

pub async fn ready(ctx: &Context) {
    ctx.set_presence(
        Some(Activity::listening("hello_human")),
        OnlineStatus::DoNotDisturb
    ).await;
}
