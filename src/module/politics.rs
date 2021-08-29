use crate::{model::Party, util::Orm};
use failure::Fallible;
use mongodb::Database;
use serenity::{
    builder::CreateInteractionResponseData, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
};
use tracing::{error, instrument};

#[derive(Debug, Default)]
pub struct Politics;

impl Politics {
    #[instrument(skip(self, ctx, db, _command))]
    pub async fn parties(
        &self,
        ctx: &Context,
        db: Database,
        _command: &ApplicationCommandInteraction,
    ) -> Fallible<CreateInteractionResponseData> {
        let mut data = CreateInteractionResponseData::default();

        let collection = db.collection::<Party>("parties");
        let parties = Party::all(&collection).await?;

        let mut fields: Vec<(String, i32, bool)> = Vec::new();
        for party in parties {
            let role = party.role(&ctx).await;

            // Skip entries with broken roles/guilds.
            if role.is_none() {
                error!("Failed to find role for party");
                continue;
            }

            let role = role.unwrap();
            let members = party.members(&ctx).await;

            let count = if let Some(members) = members {
                members.len() as i32
            } else {
                0
            };

            // Add the field to the embed.
            fields.push((role.name, count, true));
        }

        data.create_embed(|embed| embed.fields(fields));

        Ok(data)
    }
}
