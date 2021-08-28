use failure::Fallible;
use serde::{Deserialize, Serialize};
use serenity::{client::{Cache, Context}, http::Http, model::{
        guild::{Guild, Member, Role},
        id::{GuildId, RoleId},
    }};

use crate::util::Orm;

/// Represents a political party existing on a guild.
#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    pub description: String,

    guild_id: String,
    #[serde(skip)]
    guild: Option<Guild>,

    role_id: String,
    #[serde(skip)]
    role: Option<Role>,
    
    emoji_id: String,
}

impl Party {
    /// Returns an `Option` containing a `Guild`.
    pub async fn guild(&self, ctx: &Context) -> Option<Guild> {
         GuildId(
            self.guild_id
                .parse()
                .expect("failed to parse guild_id as u64"),
        )
        .to_guild_cached(ctx)
        .await
    }

    /// Returns an `Option` containing a `Role`.
    pub async fn role(&self, ctx: &Context) -> Option<Role> {
        RoleId(
            self.role_id
                .parse()
                .expect("failed to parse role_id as u64"),
        )
        .to_role_cached(&ctx)
        .await
    }

    pub async fn members(&self, ctx: &Context) -> Option<Vec<Member>> {
        let guild = self.guild(&ctx).await;

        if guild.is_none() {
            return None;
        }

        let guild = guild.unwrap();

        let members = match guild.members(&ctx.http, None, None).await {
            Ok(members) => members,
            Err(why) => return None,
        };

        let mut data = Vec::new();
        let roleid = RoleId(self.role_id.parse().unwrap());
        for member in members {
            if member.roles.contains(&roleid) {
                data.push(member);
            }
        }

        Some(data)
    }
}

impl Orm<Party> for Party {}
