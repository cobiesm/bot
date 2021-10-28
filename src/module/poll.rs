use serenity::{
    builder::{CreateSelectMenu, CreateSelectMenuOption},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

lazy_static! {}

#[command]
#[min_args(3)]
pub async fn poll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = ctx
        .http
        .get_channel(msg.channel_id.into())
        .await?
        .guild()
        .expect("Guild Channel");
    channel
        .send_message(ctx, |cm| {
            cm.components(|cc| {
                let options_list = args
                    .iter()
                    .quoted()
                    .filter_map(|arg: Result<String, _>| {
                        if let Ok(label) = arg {
                            let mut option = CreateSelectMenuOption::default();
                            option.label(&label).value(label + "s");
                            Some(option)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut menu = CreateSelectMenu::default();
                menu.min_values(0)
                    .max_values(1)
                    .placeholder(">>>>>")
                    .custom_id(1)
                    .options(|options| options.set_options(options_list));

                cc.create_action_row(|action_row| action_row.add_select_menu(menu))
            })
            .content("poll")
        })
        .await?;
    Ok(())
}
