use failure::Fallible;
use mongodb::{Collection, bson::{Document, doc}};
use serde::de::DeserializeOwned;
use serenity::{async_trait, futures::TryStreamExt};

/// Implements ORM-related functions. This includes `all`, `find`, `find_one`, and `save`.
///
/// ## Example
///
/// ```no_run
/// struct Author {
///     id: String,
///     first_name: String,
///     last_name: String,
/// }
///
/// impl Orm<Author> for Author {
/// }
///
/// fn main() {
///     let author = Author::find_one(&collection, doc!{"id": "some_uuid"}).await?;
///     author.first_name = "Bill";
///     author.save(&collection).await?;
/// }
/// ```
#[async_trait]
pub trait Orm<Item>
where
    Item: Send + Sync + DeserializeOwned + Unpin,
{
    /// Returns all the documents in the typed collection, `c`, passed.
    async fn all(c: &Collection<Item>) -> Fallible<Vec<Item>> {
        Self::find(c, doc!{}).await
    }

    /// Returns all the documents in the typed colletion, `c`, that abide to the filter.
    async fn find(c: &Collection<Item>, filter: Document) -> Fallible<Vec<Item>> {
        let mut cursor = c.find(filter, None).await?;
        let mut items = Vec::new();

        while let Some(item) = cursor.try_next().await? {
            items.push(item);
        }

        Ok(items)
    }
}
