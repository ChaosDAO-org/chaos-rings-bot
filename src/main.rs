use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::AttachmentType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::prelude::*;

use crate::commands::ring::RingError;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(
            &guild_id,
            &ctx.http,
            |commands| {
                commands.create_application_command(commands::ring::register)
            })
            .await;

        println!("I now have the following guild slash commands: {:#?}", commands);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {

            Self::respond_ack(&ctx, &command).await;

            let user_image = command.data.options.get(0)
                .and_then(|attachment| attachment.resolved.as_ref())
                .and_then(|attachment|
                    if let CommandDataOptionValue::Attachment(avatar) = attachment {
                        Some(avatar)
                    } else {
                        None
                    });

            let member = command.member.as_ref();

            if member.is_none() {
                println!("No user info found.");
            } else if user_image.is_none() {
                println!("No user image (attachment) found.");
            } else {
                let response = commands::ring::run(member.unwrap(), &user_image.unwrap()).await;
                match response {
                    Ok(avatar) => {
                        Self::respond_with_attachment(&ctx, &command, avatar).await;
                    }
                    Err(err) => {
                        println!("Failed to create avatar: {}", err);
                        if let RingError::UserRecoverableError(_reason) = err {
                            // TODO respond with an error message indicating a problem, e.g. maybe the user has no proper role
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