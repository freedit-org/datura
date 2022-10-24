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
    async fn check_ids(db: &Tree, filter_db: &Tree, site: &str) -> Vec<u32> {
        let max_id = Self::find_newest_id(db, site).await;
        let ids: Vec<u32> = (1..=max_id)
            .filter(|id| {
                !db.contains_key(u32_to_ivec(*id)).unwrap()
                    && !filter_db.contains_key(u32_to_ivec(*id)).unwrap()
            })
            .collect();

        info!("to be gotten = {}", ids.len());
        ids
    }

    fn last_id(db: &Tree) -> u32 {
        if let Some((k, _)) = db.last().unwrap() {
            let last_id = ivec_to_u32(&k);
            info!(%last_id);
            last_id
        } else {
            1
        }
    }

    #[instrument(skip(db))]
    async fn find_newest_id(db: &Tree, site: &str) -> u32 {
        let mut low = Self::last_id(db);
        let mut high = low + 500;
        while Self::is_ok(site, high).await.unwrap() {
            high += 500;
        }

        while high - low > 2 {
            let mid = low + (high - low) / 2;
            if Self::is_ok(site, mid).await.unwrap() {
                low = low + (high - low) / 2;
            } else {
                high = low + (high - low) / 2;
            };
        }

        let newest_id = if Self::is_ok(site, low + 1).await.unwrap() {
            low + 1
        } else {
            low
        };

        info!(%newest_id);
        newest_id
    }

    #[instrument]
    async fn is_ok(site: &str, id: u32) -> Result<bool, reqwest::Error> {
        let url = format!("{site}/{id}");
        let res = CLIENT.get(&url).send().await?.status().is_success();
        info!(%res);
        Ok(res)
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
