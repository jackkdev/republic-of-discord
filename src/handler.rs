use chrono::{DateTime, Utc};
use failure::Fallible;
use serde_json::Value;
use serenity::{async_trait, builder::CreateInteractionResponseData, client::{Context, EventHandler}, model::{id::GuildId, interactions::{Interaction, application_command::{ApplicationCommand, ApplicationCommandInteraction}}, prelude::Ready}};
use tracing::{debug, error, info, instrument, warn};

/// Core `EventHandler` for the bot. The `Handler` also contains all the modules that the bot has
/// to offer. Most importantly, these modules are then used in Discord integrations to create
/// interactive commands.
pub struct Handler {
    context: HandlerContext,
}

impl Handler {
    /// Creates a new `Handler` with the `HandlerContext` passed.
    pub fn with_context(context: HandlerContext) -> Self {
        Self { context }
    }

    #[instrument(skip(self, ctx, _interaction))]
    async fn on_application_command(
        &self,
        ctx: &Context,
        _interaction: &Interaction,
        command: &ApplicationCommandInteraction,
    ) -> Fallible<()> {
        let data = match command.data.name.as_str() {
            _ => {
                warn!(
                    "An unimplemented command has been executed: {}",
                    command.data.name
                );

                // Send an error message for commands that aren't implemented.
                let mut data = CreateInteractionResponseData::default();
                data.create_embed(|embed| {
                    embed
                        .title("An error has occurred,")
                        .description("`This command is not yet implemented.`")
                        .color(0xff5036)
                        .footer(|footer| {
                            footer
                                .text("Republic of Discord")
                                .icon_url("https://i.imgur.com/mwRuYOs.png")
                        })
                        .timestamp(Utc::now().to_rfc3339())
                });

                data
            }
        };

        command
            .create_interaction_response(&ctx.http, |response| {
                response.0.insert(
                    "data",
                    Value::Object(serenity::utils::hashmap_to_json_map(data.0)),
                );
                response
            })
            .await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx, _data_about_bot))]
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        // Create the global application comands.
        if let Err(why) = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
        }).await {
            error!("Failed to create global application commands: {}", why);
            std::process::exit(1);
        }

        info!("Created global application commands");

        // Create the development-guild application commands.
        if let Err(why) = GuildId(self.context.development_guild_id).set_application_commands(&ctx.http, |commands| {
            commands
        }).await {
            error!("Failed to create development-guild application commands: {}", why);
            std::process::exit(1);
        }

        info!("Created development-guild application commands");
    }

    #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match &interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(why) = self
                    .on_application_command(&ctx, &interaction, &command)
                    .await
                {
                    error!("Error executing command: {}", why);
                }
            }
            _ => {}
        }
    }
}

/// Contains the data passed to all interaction handlers.
pub struct HandlerContext {
    pub development_guild_id: u64,

    pub mongo_client: mongodb::Client,
}
