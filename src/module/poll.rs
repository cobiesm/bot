use std::error::Error;

use regex::Regex;
use serenity::{
    builder::{CreateSelectMenu, CreateSelectMenuOption},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        interactions::{Interaction, InteractionResponseType, InteractionType},
    },
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
            let mut options_str = Vec::new();
            cm.components(|cc| {
                let options_list = args
                    .iter()
                    .quoted()
                    .filter_map(|arg: Result<String, _>| {
                        if let Ok(label) = arg {
                            options_str.push(format!("{}: 0", label.to_string()));
                            let mut option = CreateSelectMenuOption::default();
                            option.label(&label).value(label);
                            Some(option)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut menu = CreateSelectMenu::default();
                menu.min_values(1)
                    .max_values(1)
                    .placeholder(">>>>>")
                    .custom_id(1)
                    .options(|options| options.set_options(options_list));

                cc.create_action_row(|action_row| action_row.add_select_menu(menu))
            })
            .content(format!("poll:\n{}", options_str.join("\n")))
        })
        .await?;

    Ok(())
}

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
    if interaction.kind() != InteractionType::MessageComponent {
        return;
    }

    let interaction = interaction.message_component().unwrap();
    let message = interaction.message.clone();
    let selection = interaction.data.values.first().unwrap();

    interaction
        .create_interaction_response(&ctx, |resp| {
            if message.reactions.iter().any(|reaction| {
                reaction.reaction_type
                    == serenity::model::channel::ReactionType::Unicode("⛔".to_string())
            }) {
                return resp.kind(InteractionResponseType::Pong);
            }

            resp.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|respdata| {
                    respdata.content(update(message.content, selection.into()).unwrap())
                    //todo!()
                })
        })
        .await
        .unwrap();
    interaction.defer(&ctx).await.unwrap();
}

fn update(poll: String, selection: String) -> Result<String, Box<dyn Error>> {
    let expr = Regex::new(&format!(r"(?i){}: (\d+)", selection))?;
    if let Some(captures) = expr.captures(&poll) {
        let number = captures[1].parse::<usize>()?;
        Ok(expr
            .replace(&poll, format!("{}: {}", selection, number + 1))
            .to_string())
    } else {
        Err("fuck".into())
    }
}
