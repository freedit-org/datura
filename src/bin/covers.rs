use std::{
    collections::HashMap,
    fs::{self, File},
    io::{copy, BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
};

use datura::CLIENT;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    // read img urls
    let mut args = std::env::args();
    let fpath = args.nth(1).expect("file path not found");
    let file = File::open(fpath).unwrap();
    let r = BufReader::new(file);

    let mut urls = HashMap::new();
    for i in r.lines() {
        let i = i.unwrap();
        if !i.is_empty() {
            let (_, fname) = i.rsplit_once('/').unwrap();
            urls.insert(fname.to_owned(), i);
        }
    }

    let out_path = args.next().expect("file path not found");
    let dir = PathBuf::from(&out_path);
    let out_path = Arc::new(out_path);

    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap();
    }

    // skip the downloaded urls
    dbg!(urls.len());
    urls.retain(|k, _| !&dir.join(k).is_file());
    dbg!(urls.len());

    let mut handers = Vec::with_capacity(urls.len());
    let semaphore = Arc::new(Semaphore::new(500));

    for i in urls.into_values() {
        if !i.is_empty() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let out_path = out_path.clone();
            let h = tokio::spawn(async move {
                println!("{}", &i);

                let mut response = CLIENT.get(&i).send().await;
                let mut cnt = 0;
                while response.is_err() {
                    println!("{:?}", response);
                    response = CLIENT.get(&i).send().await;
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

                            let fpath = format!("{out_path}/{fname}");

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
