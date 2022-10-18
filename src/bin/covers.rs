use std::{
    collections::HashMap,
    fs::{self, File},
    io::{copy, BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    // read img urls
    let file = File::open("zlib2_covers_zh.txt").unwrap();
    let r = BufReader::new(file);

    let mut urls = HashMap::new();
    for i in r.lines() {
        let i = i.unwrap();
        if !i.is_empty() {
            let (_, fname) = i.rsplit_once('/').unwrap();
            urls.insert(fname.to_owned(), i);
        }
    }

    let path = "zh_covers";
    let dir = PathBuf::from(path);

    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap();
    }

    // skip the downloaded urls
    dbg!(urls.len());
    urls.retain(|k, _| !&dir.join(k).is_file());
    dbg!(urls.len());

    let mut handers = Vec::with_capacity(urls.len());
    let semaphore = Arc::new(Semaphore::new(500));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    for i in urls.into_values() {
        if !i.is_empty() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let h = tokio::spawn(async move {
                println!("{}", &i);

                let mut response = client.get(&i).send().await;
                let mut cnt = 0;
                while response.is_err() {
                    println!("{:?}", response);
                    response = client.get(&i).send().await;
                    cnt += 1;
                    if cnt >= 2 {
                        break;
                    }
                }

                if let Ok(r) = response {
                    if r.status().is_success() {
                        let mut dest = {
                            let fname = r
                                .url()
                                .path_segments()
                                .and_then(|segments| segments.last())
                                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                                .unwrap_or("tmp.bin");

                            println!("file to download: '{}'", fname);

                            let fpath = format!("{path}/{fname}");

                            File::create(fpath).unwrap()
                        };
                        let content = r.bytes().await.unwrap();
                        copy(&mut content.as_ref(), &mut dest).unwrap();
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
    }

    for i in handers {
        i.await.unwrap();
    }
}
