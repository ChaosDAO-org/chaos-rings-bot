use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::gateway::Ready;
use serenity::model::prelude::AttachmentType;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::prelude::*;

use crate::commands::ring::UserRecoverableError;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let command = Command::create_global_application_command(
            &ctx.http,
            |command| { commands::ring::register(command) },
        ).await;

        println!("Registered command: {:#?}", command);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            Self::respond_ack(&ctx, &command).await;

            let user_image = command.data.options.get(0)
                .and_then(|data_option| data_option.resolved.as_ref())
                .and_then(|option_value|
                    if let CommandDataOptionValue::Attachment(avatar) = option_value {
                        Some(avatar)
                    } else {
                        None
                    });

            let member = command.member.as_ref();

            if member.is_none() {
                Self::respond_with_error(&ctx, &command, "No user info found.").await;
                return;
            }

            if user_image.is_none() {
                Self::respond_with_error(&ctx, &command, "No user image (attachment) found.").await;
                return;
            }

            let response = commands::ring::run(member.unwrap(), user_image.unwrap()).await;
            match response {
                Ok(avatar) => {
                    Self::respond_with_attachment(&ctx, &command, avatar).await;
                }
                Err(err) => {
                    println!("Failed to create an avatar: {}", err);
                    match err.downcast_ref::<UserRecoverableError>() {
                        Some(user_recoverable_error) => {
                            Self::respond_with_error(&ctx, &command, &format!("{}", &user_recoverable_error)).await;
                        }
                        None => {
                            Self::respond_with_error(&ctx, &command, "Unexpected error").await;
                        }
                    }
                }
            }
        }
    }
}

impl Handler {
    async fn respond_ack(ctx: &Context, command: &ApplicationCommandInteraction) {
        if let Err(why) = &command
            .create_interaction_response(
                &ctx.http,
                |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(
                            |message| {
                                message.ephemeral(true);
                                message.content("Preparing your avatar...")
                            })
                })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }

    #[allow(clippy::needless_lifetimes)]
    async fn respond_with_attachment<'a, 'b>(ctx: &'a Context, command: &ApplicationCommandInteraction, attachment: AttachmentType<'b>) {
        if let Err(why) = command.create_followup_message(
            &ctx.http,
            |response| {
                response.ephemeral(true);
                response.add_file(attachment)
            })
            .await
        {
            println!("Cannot send back an updated avatar: {}", why);
        }
    }

    async fn respond_with_error(ctx: &Context, command: &ApplicationCommandInteraction, err_msg: &str) {
        if let Err(why) = command.create_followup_message(
            &ctx.http,
            |response| {
                response.ephemeral(true);
                response.content(err_msg.to_string())
            })
            .await
        {
            println!("Cannot send back an error message: {}", why);
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}