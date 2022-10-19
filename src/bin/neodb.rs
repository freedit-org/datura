use bincode::config::standard;
use datura::{
    config::CONFIG,
    extract::{Book, Movie},
    ivec_to_u32, u32_to_ivec, CLIENT,
};
use reqwest::StatusCode;
use std::{path::PathBuf, sync::Arc, time::Duration};
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

    //movies
    let movies_tree = db.open_tree("movies").unwrap();
    let db_max_id = if let Some((k, _)) = movies_tree.last().unwrap() {
        ivec_to_u32(&k)
    } else {
        0
    };
    info!(%db_max_id);

    let min_id = CONFIG.movie_min_id.unwrap_or(db_max_id);
    let max_id = CONFIG.movie_max_id;
    let site = "https://neodb.social/movies";

    info!(%min_id);
    info!(%max_id);

    let movie_404_tree = db.open_tree("movie_404").unwrap();
    let mut ids = vec![];
    for id in min_id..=max_id {
        if !movies_tree.contains_key(u32_to_ivec(id)).unwrap()
            && !movie_404_tree.contains_key(u32_to_ivec(id)).unwrap()
        {
            ids.push(id);
        }
    }

    info!("Movies to be gotten = {}", ids.len());

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    for id in ids {
        let movie_404_tree = movie_404_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let movies_tree = movies_tree.clone();

        let h = tokio::spawn(async move {
            let url = format!("{site}/{id}");
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
                        let movie = Movie::from(content.as_ref());
                        let encoded = bincode::encode_to_vec(&movie, standard()).unwrap();
                        movies_tree.insert(u32_to_ivec(id), encoded).unwrap();
                        if id % 100 == 0 {
                            info!("finished {}", &id);
                        }
                    } else if r.status() == StatusCode::NOT_FOUND {
                        error!("{} 404 not found", id);
                        movie_404_tree.insert(u32_to_ivec(id), &[]).unwrap();
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

    for h in handers {
        h.await.unwrap();
    }

    // download cover
    let dir = PathBuf::from(&CONFIG.movie_cover_path);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    let mut ids = vec![];
    let movie_covers_tree = db.open_tree("movie_covers").unwrap();
    for id in min_id..=max_id {
        if !movie_covers_tree.contains_key(u32_to_ivec(id)).unwrap()
            && !movie_404_tree.contains_key(u32_to_ivec(id)).unwrap()
        {
            ids.push(id);
        }
    }

    info!("ids.len = {}", ids.len());

    let mut handers = vec![];
    for id in ids {
        let movies_tree = movies_tree.clone();
        let movie_covers_tree = movie_covers_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let h = tokio::spawn(async move {
            info!(id);
            if let Some(v) = movies_tree.get(u32_to_ivec(id)).unwrap() {
                let (movie, _): (Movie, usize) =
                    bincode::decode_from_slice(&v, standard()).unwrap();
                if let Some(cover) = movie.cover {
                    let url = format!("https://neodb.social{cover}");
                    let ext = url.rsplit_once('.').unwrap().1;
                    let mut response = CLIENT.get(&url).send().await;

                    let mut cnt = 0;
                    while response.is_err() {
                        error!("{:?}", response);
                        response = CLIENT.get(&url).send().await;
                        cnt += 1;
                        if cnt >= 2 {
                            break;
                        }
                    }

                    if let Ok(r) = response {
                        if r.status().is_success() {
                            let fpath = format!("{}/{}.{}", &CONFIG.movie_cover_path, id, ext);
                            let content = r.bytes().await.unwrap();
                            tokio::fs::write(fpath, content).await.unwrap();
                            movie_covers_tree.insert(u32_to_ivec(id), &[]).unwrap();
                            if id % 100 == 0 {
                                info!("finished {}", &id);
                            }
                        } else {
                            error!("{:?}", r);
                        }
                    } else {
                        error!("{:?}", response);
                    }
                }
            }
            drop(permit);
        });
        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }

    // books
    let books_tree = db.open_tree("books").unwrap();
    let db_max_id = if let Some((k, _)) = books_tree.last().unwrap() {
        ivec_to_u32(&k)
    } else {
        0
    };
    info!(%db_max_id);

    let min_id = CONFIG.book_min_id.unwrap_or(db_max_id);
    let max_id = CONFIG.book_max_id;
    let site = "https://neodb.social/books";

    info!(%min_id);
    info!(%max_id);

    let book_404_tree = db.open_tree("book_404").unwrap();
    let mut ids = vec![];
    for id in min_id..=max_id {
        if !books_tree.contains_key(u32_to_ivec(id)).unwrap()
            && !book_404_tree.contains_key(u32_to_ivec(id)).unwrap()
        {
            ids.push(id);
        }
    }

    info!("ids.len = {}", ids.len());

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    for id in ids {
        let book_404_tree = book_404_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let books_tree = books_tree.clone();

        let h = tokio::spawn(async move {
            let url = format!("{site}/{id}");
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
                        books_tree.insert(u32_to_ivec(id), encoded).unwrap();
                        if id % 100 == 0 {
                            info!("finished {}", &id);
                        }
                    } else if r.status() == StatusCode::NOT_FOUND {
                        error!("{} 404 not found", id);
                        book_404_tree.insert(u32_to_ivec(id), &[]).unwrap();
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

    for h in handers {
        h.await.unwrap();
    }

    // download cover
    let dir = PathBuf::from(&CONFIG.book_cover_path);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    let mut ids = vec![];
    let book_covers_tree = db.open_tree("book_covers").unwrap();
    for id in min_id..=max_id {
        if !book_covers_tree.contains_key(u32_to_ivec(id)).unwrap()
            && !book_404_tree.contains_key(u32_to_ivec(id)).unwrap()
        {
            ids.push(id);
        }
    }

    info!("ids.len = {}", ids.len());

    let mut handers = vec![];
    for id in ids {
        let books_tree = books_tree.clone();
        let book_covers_tree = book_covers_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let h = tokio::spawn(async move {
            if let Some(v) = books_tree.get(u32_to_ivec(id)).unwrap() {
                let (book, _): (Book, usize) = bincode::decode_from_slice(&v, standard()).unwrap();
                if let Some(cover) = book.cover {
                    let url = format!("https://neodb.social{cover}");
                    let ext = url.rsplit_once('.').unwrap().1;
                    let mut response = CLIENT.get(&url).send().await;

                    let mut cnt = 0;
                    while response.is_err() {
                        error!("{:?}", response);
                        response = CLIENT.get(&url).send().await;
                        cnt += 1;
                        if cnt >= 2 {
                            break;
                        }
                    }

                    if let Ok(r) = response {
                        if r.status().is_success() {
                            let fpath = format!("{}/{}.{}", &CONFIG.book_cover_path, id, ext);
                            let content = r.bytes().await.unwrap();
                            tokio::fs::write(fpath, content).await.unwrap();
                            book_covers_tree.insert(u32_to_ivec(id), &[]).unwrap();
                            if id % 100 == 0 {
                                info!("finished {}", &id);
                            }
                        } else {
                            error!("{:?}", r);
                        }
                    } else {
                        error!("{:?}", response);
                    }
                }
            }
            drop(permit);
        });
        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }
}
