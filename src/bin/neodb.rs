use std::{
    fs::{self, File},
    io::copy,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let max_id: usize = 511650;
    let site = "https://neodb.social/books";

    let mut ids = Vec::with_capacity(max_id);
    for i in 1..=max_id {
        ids.push(i);
    }

    let path = "neodb";
    let dir = PathBuf::from(path);
    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap();
    }

    dbg!(ids.len());
    ids.retain(|id| !&dir.join(format!("{id}.html")).is_file());
    dbg!(ids.len());

    let mut handers = Vec::with_capacity(max_id);
    let semaphore = Arc::new(Semaphore::new(100));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    for i in ids {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let h = tokio::spawn(async move {
            println!("{}", &i);

            let url = format!("{site}/{i}");
            let mut response = client.get(&url).send().await;
            let mut cnt = 0;
            while response.is_err() {
                println!("{:?}", response);
                response = client.get(&url).send().await;
                cnt += 1;
                if cnt >= 2 {
                    break;
                }
            }

            if let Ok(r) = response {
                if r.status().is_success() {
                    let mut dest = {
                        let fpath = format!("{path}/{i}.html");

                        File::create(fpath).unwrap()
                    };
                    let content = r.text().await.unwrap();
                    copy(&mut content.as_bytes(), &mut dest).unwrap();
                    println!("finished {}", &i);
                } else {
                    println!("{:?}", r);
                }
            } else {
                println!("{:?}", response);
            }
            drop(permit);
        });

        handers.push(h);
    }

    for i in handers {
        i.await.unwrap();
    }
}
