use std::process;

use clap::{App, Arg};
use failure::Fallible;
use module::politics::Politics;
use mongodb::options::ClientOptions;
use serde_json::value::Value;
use serenity::{
    async_trait,
    builder::CreateInteractionResponseData,
    client::{Context, EventHandler},
    model::{id::GuildId, interactions::Interaction, prelude::Ready},
    Client,
};
use tracing::{error, instrument};

mod model;
mod module;
mod util;

/// Handles events through `EventHandler`.
#[derive(Debug)]
struct Handler {
    politics: Politics,
    mongo: mongodb::Client,
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx, _data_about_bot))]
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        if let Err(why) = GuildId(872182516449157220)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("parties")
                    .description("Shows all the parties in the current guild.")
            })
            .await
        {
            error!("Failed to register global application commands: {}", why);
            process::exit(1);
        };
    }

    #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result: Option<Fallible<CreateInteractionResponseData>> =
                match command.data.name.as_str() {
                    "parties" => Some(
                        self.politics
                            .parties(&ctx, self.mongo.database("republic-of-discord"), &command)
                            .await,
                    ),
                    _ => None,
                };

            if let Some(result) = result {
                let data = match result {
                    Ok(data) => data,
                    Err(why) => {
                        let mut data = CreateInteractionResponseData::default();
                        data.create_embed(move |embed| {
                            embed.color(0xff1010).description(format!("Error: {}", why))
                        });

                        data
                    }
                };

                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response.0.insert(
                            "data",
                            Value::Object(serenity::utils::hashmap_to_json_map(data.0)),
                        );
                        response
                    })
                    .await
                {
                    error!("Error responding to interaction: {}", why);
                };
            };
        }
    }
}

#[tokio::main]
async fn main() -> Fallible<()> {
    tracing_subscriber::fmt::init();

    let matches = App::new("republic-of-discord")
        .version("0.1.0")
        .author("jackk <jackk@darkrp-is.gay>")
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .takes_value(true)
                .help("Sets the token of the Discord bot"),
        )
        .arg(
            Arg::with_name("application_id")
                .short("a")
                .long("application-id")
                .takes_value(true)
                .help("Sets the application ID of the Discord bot"),
        )
        .arg(
            Arg::with_name("mongodb_uri")
                .short("m")
                .long("mongodb-uri")
                .takes_value(true)
                .help("Sets the MongoDB URI for the application to connect to."),
        )
        .get_matches();

    let token = match matches.value_of("token") {
        Some(value) => value,
        None => panic!("a token must be provided"),
    };

    let application_id = match matches.value_of("application_id") {
        Some(value) => value,
        None => panic!("an application id must be provided"),
    };

    let mongodb_uri = match matches.value_of("mongodb_uri") {
        Some(value) => value,
        None => panic!("a mongodb uri must be provided"),
    };

    // Connect to MongoDB.
    let mongo = mongodb::Client::with_options(ClientOptions::parse(mongodb_uri).await?)?;

    // Create the bot client with the options from the command-line.
    let mut client = Client::builder(token)
        .application_id(
            application_id
                .parse()
                .expect("expected the application_id to be an unsigned integer"),
        )
        .event_handler(Handler { 
            mongo,
            politics: Politics::default(),
        })
        .await?;

    if let Err(why) = client.start().await {
        panic!("{}", why);
    }

    Ok(())
}
