use serenity::client::Context;
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

pub async fn ready(ctx: &Context) {
    ctx.set_presence(
        Some(Activity::listening("hello_human")),
        OnlineStatus::DoNotDisturb,
    )
    .await;
}
