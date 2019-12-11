use azure_sdk_cosmos::prelude::*;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyStruct {
    id: String,
    color: String,
    myvalue: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyStruct2 {
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // First we retrieve the account name and master key from environment variables.
    // We expect master keys (ie, not resource constrained)
    let master_key =
        std::env::var("COSMOS_MASTER_KEY").expect("Set env variable COSMOS_MASTER_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    let authorization_token =
        AuthorizationToken::new(account.clone(), TokenType::Master, &master_key)?;

    let client = Client2Builder::new(authorization_token)?;

    let dbs = client.list().finalize().await?;

    for db in dbs.databases {
        println!("database == {:?}", db);
        let database = client.with_database(&db);

        let collections = database.list().finalize().await?;
        for collection in collections.collections {
            println!("collection == {:?}", collection);
            let collection = database.with_collection(&collection);

            let documents = collection.list().get_as_json().await?;
            println!("\ndocuments as json == {:?}", documents);

            // get by id
            let doc = collection
                .get()
                .with_document_id("3321000d-8d9f-f6e9-24e5-d1a3e217eb1a")
                .with_partition_key(&vec!["cyan"])
                .with_query_cross_partition(true)
                .get_as_entity::<MyStruct>()
                .await?;
            println!("\ndocument retrieved == {:?}", doc);

            let doc = collection
                .get()
                .with_document_id("3321000d-8d9f-f6e9-24e5-d1a3e217eb1a")
                .with_partition_key(&vec!["cyan"])
                .with_query_cross_partition(true)
                .get_as_json()
                .await?;
            println!("\ndocument retrieved == {:?}", doc);

            let documents = collection.list().get_as_entity::<MyStruct>().await?;
            println!("\ndocuments deserialized == {:?}", documents);

            // we need this binding to extend the lifespan
            // of the request. This is a drawback of the non lexical
            // lifetimes.
            let o = collection.list().with_max_item_count(2);
            let mut stream = Box::pin(o.stream_as_entity::<MyStruct>());

            println!("\nstreaming");
            while let Some(res) = stream.next().await {
                println!("item ==> {:?}", res);
            }
        }
    }

    Ok(())
}
