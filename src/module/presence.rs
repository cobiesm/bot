use serenity::client::Context;
use serenity::model::user::OnlineStatus;
use serenity::model::gateway::Activity;

pub async fn ready(ctx: &Context) {
    ctx.set_presence(
        Some(Activity::listening(".help | loli hunter")),
        OnlineStatus::DoNotDisturb
    ).await;
}
