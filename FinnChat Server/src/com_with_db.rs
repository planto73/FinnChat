use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;

async fn get_collection() -> Collection<Document> {
    let uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    let client = Client::with_uri_str(uri).await.unwrap();

    let db = client.database("FinnChat");
    db.collection::<Document>("msg")
}

pub async fn get_messages() -> String {
    let collection = get_collection().await;
    collection.find(None, None).await.unwrap();

    if let Ok(cursor) = collection.find(None, None).await {
        let list = cursor.try_collect().await.unwrap_or_else(|_| vec![]);

        let mut messages = String::new();
        for item in list {
            let user = item.get_str("usr").expect("Database corrupted!");
            let msg = item.get_str("msg").expect("Database corrupted!");
            let res = format!("{}: {}", user, msg);
            messages += &[&res, "\n"].join("");
        }
        messages
    } else {
        String::new()
    }
}

pub async fn update_db(name: String, msg: &str) {
    let collection = get_collection().await;
    let res = doc! {"usr":name, "msg":msg};
    collection
        .insert_one(res, None)
        .await
        .expect("Failed to update db!");
}
