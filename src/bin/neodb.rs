use datura::{
    config::CONFIG,
    download::Web,
    extract::{Album, Book, Movie},
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
use tracing::info;
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

    let site = "https://neodb.social/movies";
    let movies_tree = db.open_tree("movies").unwrap();
    let movie_404_tree = db.open_tree("movie_404").unwrap();
    let ids = Movie::check_ids(&movies_tree, &movie_404_tree, site).await;

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));
    for id in ids {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let movies_tree = movies_tree.clone();
        let movie_404_tree = movie_404_tree.clone();

        let h = tokio::spawn(async move {
            Movie::get_data(site, id, &movies_tree, &movie_404_tree).await;
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

    let movie_covers_tree = db.open_tree("movie_covers").unwrap();
    let ids = Movie::check_ids(&movie_covers_tree, &movie_404_tree, site).await;

    let mut handers = vec![];
    for id in ids {
        let movies_tree = movies_tree.clone();
        let movie_covers_tree = movie_covers_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let h = tokio::spawn(async move {
            Movie::dl_cover(
                id,
                &movies_tree,
                &movie_covers_tree,
                &CONFIG.movie_cover_path,
            )
            .await;
            drop(permit);
        });
        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }

    // books
    let site = "https://neodb.social/books";
    let books_tree = db.open_tree("books").unwrap();
    let book_404_tree = db.open_tree("book_404").unwrap();
    let ids = Book::check_ids(&books_tree, &book_404_tree, site).await;

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));

    for id in ids {
        let book_404_tree = book_404_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let books_tree = books_tree.clone();
        let h = tokio::spawn(async move {
            Book::get_data(site, id, &books_tree, &book_404_tree).await;
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

    let book_covers_tree = db.open_tree("book_covers").unwrap();
    let ids = Book::check_ids(&book_covers_tree, &book_404_tree, site).await;

    let mut handers = vec![];
    for id in ids {
        let books_tree = books_tree.clone();
        let book_covers_tree = book_covers_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let h = tokio::spawn(async move {
            Book::dl_cover(id, &books_tree, &book_covers_tree, &CONFIG.book_cover_path).await;
            drop(permit);
        });
        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }

    // album
    let site = "https://neodb.social/music/album";
    let albums_tree = db.open_tree("albums").unwrap();
    let album_404_tree = db.open_tree("album_404").unwrap();
    let ids = Album::check_ids(&albums_tree, &album_404_tree, site).await;

    let mut handers = vec![];
    let semaphore = Arc::new(Semaphore::new(100));

    for id in ids {
        let album_404_tree = album_404_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let albums_tree = albums_tree.clone();
        let h = tokio::spawn(async move {
            Album::get_data(site, id, &albums_tree, &album_404_tree).await;
            drop(permit);
        });

        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }

    // download cover
    let dir = PathBuf::from(&CONFIG.album_cover_path);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }

    let album_covers_tree = db.open_tree("album_covers").unwrap();
    let ids = Album::check_ids(&album_covers_tree, &album_404_tree, site).await;

    let mut handers = vec![];
    for id in ids {
        let albums_tree = albums_tree.clone();
        let album_covers_tree = album_covers_tree.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let h = tokio::spawn(async move {
            Album::dl_cover(
                id,
                &albums_tree,
                &album_covers_tree,
                &CONFIG.album_cover_path,
            )
            .await;
            drop(permit);
        });
        handers.push(h);
    }

    for h in handers {
        h.await.unwrap();
    }
}
