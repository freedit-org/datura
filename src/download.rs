use crate::extract::{Book, Movie};
use async_trait::async_trait;
use bincode::{config::standard, Decode, Encode};
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use sled::{IVec, Tree};
use std::{fmt::Debug, time::Duration};
use tracing::{error, info, instrument};

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
});

pub trait Cover {
    fn cover(self) -> Option<String>;
}

impl Cover for Movie {
    fn cover(self) -> Option<String> {
        self.cover
    }
}

impl Cover for Book {
    fn cover(self) -> Option<String> {
        self.cover
    }
}

#[async_trait]
pub trait Web: Debug + Encode + Decode + Cover + for<'a> From<&'a str> {
    #[instrument(skip(db, filter_db))]
    fn check_ids(min_id: Option<u32>, max_id: u32, db: &Tree, filter_db: &Tree) -> Vec<u32> {
        let min_id = if let Some(min_id) = min_id {
            min_id
        } else if let Some((k, _)) = db.last().unwrap() {
            let db_max_id = ivec_to_u32(&k);
            info!(%db_max_id);
            db_max_id
        } else {
            1
        };

        let ids: Vec<u32> = (min_id..=max_id)
            .filter(|id| {
                !db.contains_key(u32_to_ivec(*id)).unwrap()
                    && !filter_db.contains_key(u32_to_ivec(*id)).unwrap()
            })
            .collect();

        info!("to be gotten = {}", ids.len());
        ids
    }

    #[instrument(skip(db, db_404))]
    async fn get_data(site: &str, id: u32, db: &Tree, db_404: &Tree) {
        let url = format!("{site}/{id}");
        let mut response = CLIENT.get(&url).send().await;
        let mut cnt = 0;
        while response.is_err() {
            error!("{}", response.unwrap_err());
            response = CLIENT.get(&url).send().await;
            cnt += 1;
            if cnt >= 2 {
                break;
            }
        }

        match response {
            Ok(r) => {
                if r.status().is_success() {
                    match r.text().await {
                        Ok(content) => {
                            let one = Self::from(content.as_ref());
                            let encoded = bincode::encode_to_vec(&one, standard()).unwrap();
                            db.insert(u32_to_ivec(id), encoded).unwrap();
                            if id % 100 == 0 {
                                info!("finished.");
                            }
                        }
                        Err(e) => error!(%e),
                    }
                } else if r.status() == StatusCode::NOT_FOUND {
                    error!("404 not found");
                    db_404.insert(u32_to_ivec(id), &[]).unwrap();
                } else {
                    error!(?r);
                }
            }
            Err(e) => error!(%e),
        }
    }

    fn get_cover(id: u32, db: &Tree) -> Option<String> {
        if let Some(v) = db.get(u32_to_ivec(id)).unwrap() {
            let (one, _): (Self, usize) = bincode::decode_from_slice(&v, standard()).unwrap();
            one.cover()
        } else {
            None
        }
    }

    #[instrument(skip(db, db_cover, cover_path))]
    async fn dl_cover(id: u32, db: &Tree, db_cover: &Tree, cover_path: &str) {
        if let Some(cover) = Self::get_cover(id, db) {
            let url = format!("https://neodb.social{cover}");
            let ext = url.rsplit_once('.').unwrap().1;
            let mut response = CLIENT.get(&url).send().await;
            let mut cnt = 0;
            while response.is_err() {
                error!(?response);
                response = CLIENT.get(&url).send().await;
                cnt += 1;
                if cnt >= 2 {
                    break;
                }
            }

            match response {
                Ok(r) => {
                    if r.status().is_success() {
                        let fpath = format!("{}/{}.{}", cover_path, id, ext);
                        match r.bytes().await {
                            Ok(content) => {
                                tokio::fs::write(fpath, content).await.unwrap();
                                db_cover.insert(u32_to_ivec(id), &[]).unwrap();
                                if id % 100 == 0 {
                                    info!("finished {}", &id);
                                }
                            }
                            Err(e) => error!(%e),
                        }
                    } else {
                        error!(?r);
                    }
                }
                Err(e) => error!(%e),
            }
        };
    }
}

impl Web for Movie {}
impl Web for Book {}

/// convert `u32` to [IVec]
fn u32_to_ivec(number: u32) -> IVec {
    IVec::from(number.to_be_bytes().to_vec())
}

/// convert [IVec] to u32
fn ivec_to_u32(iv: &IVec) -> u32 {
    u32::from_be_bytes(iv.to_vec().as_slice().try_into().unwrap())
}
