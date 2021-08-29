use clap::{App, Arg};
use failure::Fallible;
use mongodb::options::ClientOptions;
use serenity::Client;

use crate::handler::{Handler, HandlerContext};
use crate::module::politics::Politics;

mod handler;
mod model;
mod module;
mod util;

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
        .arg(
            Arg::with_name("development_guild_id")
                .short("g")
                .long("development-guild-id")
                .takes_value(true)
                .help("Sets the Guild which has the hot-reload development commands.")
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

    let development_guild_id: u64 = match matches.value_of("development_guild_id") {
        Some(value) => value,
        None => panic!("a development guild id must be provided"),
    }.parse().expect("expected development_guild_id to be a u64");

    // Connect to MongoDB.
    let mongo_client = mongodb::Client::with_options(ClientOptions::parse(mongodb_uri).await?)?;

    // Create the bot client with the options from the command-line.
    let mut client = Client::builder(token)
        .application_id(
            application_id
                .parse()
                .expect("expected the application_id to be an unsigned integer"),
        )
        .event_handler(Handler::with_context(HandlerContext {
            development_guild_id,
            mongo_client,
        }))
        .await?;

    if let Err(why) = client.start().await {
        panic!("{}", why);
    }

    Ok(())
}
