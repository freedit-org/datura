use bincode::config::standard;
use datura::{config::CONFIG, extract::Book, ivec_to_u32, u32_to_ivec};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tracing::{error, info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = &CONFIG.db;
    let config = sled::Config::default().path(db_url).use_compression(true);
    let db = config.open().unwrap();
    info!(%db_url);

    let db_max_id = if let Some((k, _)) = db.last().unwrap() {
        ivec_to_u32(&k)
    } else {
        0
    };
    info!(%db_max_id);

    let min_id = CONFIG.min_id.unwrap_or(db_max_id);
    let max_id = CONFIG.max_id;
    let site = "https://neodb.social/books";

    info!(%min_id);
    info!(%max_id);

    let mut ids = vec![];
    let tree = db.open_tree("books").unwrap();
    for i in min_id..=max_id {
        if !tree.contains_key(u32_to_ivec(i)).unwrap() {
            ids.push(i);
        }
    }

    info!("ids.len = {}", ids.len());

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    for i in ids {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let tree = tree.clone();
        let h = tokio::spawn(async move {
            info!("{}", &i);

            let url = format!("{site}/{i}");
            let mut response = client.get(&url).send().await;
            let mut cnt = 0;
            while response.is_err() {
                error!("{:?}", response);
                response = client.get(&url).send().await;
                cnt += 1;
                if cnt >= 2 {
                    break;
                }
            }

            match response {
                Ok(r) => {
                    if r.status().is_success() {
                        let content = r.text().await.unwrap();
                        let book = Book::from(content.as_ref());
                        let encoded = bincode::encode_to_vec(&book, standard()).unwrap();
                        tree.insert(u32_to_ivec(i), encoded).unwrap();
                        info!("finished {}", &i);
                    } else {
                        error!("{:?}", r);
                    }
                }
                Err(e) => error!("{:?}", e),
            }
            drop(permit);
        });

        handers.push(h);
    }

    for i in handers {
        i.await.unwrap();
    }
}
