use std::{env::args, error::Error, fs::File, path::Path, process::exit, sync::Arc, time::Instant};
use futures::future::join_all;
use reqwest::Client;
use sqlx::{Executor, SqlitePool};
use tokio::{sync::mpsc, task::JoinHandle};
use dane_krs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let start = Instant::now();

    let ilosc_requestow =
    match args().nth(1)
    {
        Some(arg) => match arg.parse::<usize>()
        {
            Ok(num) => num,
            Err(_) =>
            {
                eprintln!("Nie podano liczby, lub podano liczbę ujemną");
                exit(1)
            },
        }
        None => 1000
    };

    let check_db = Path::new("krs.db");
    if !(check_db.exists() && check_db.is_file())
    {
        File::create(check_db).unwrap_or_else(|err|
        {
            eprintln!("Could not create the file ({err})");
            exit(1)
        });
    }

    let pool = SqlitePool::connect("sqlite://krs.db").await?;
    pool.execute(CREATE_SCRIPT).await?;

    let (sender, reciever) = mpsc::channel::<JsonResponse>(100);

    let db_task_handle = tokio::spawn(db_task_handle(reciever, pool));

    let client = Arc::new(Client::new());
    let mut handles = Vec::with_capacity(ilosc_requestow * size_of::<JoinHandle<()>>());
    for i in 0..ilosc_requestow
    {
        let client = client.clone();
        let sender = sender.clone();
        handles.push(tokio::spawn(send_request(i, client.clone(), sender.clone())))
    }
    join_all(handles).await;
    drop(sender);
    db_task_handle.await?;

    println!("{:?}", start.elapsed());
    Ok(())
}