use std::{error::Error, fs::File, mem::transmute, path::Path, process::exit, time::Instant};
use clap::Parser;
use futures::future::join_all;
use reqwest::Client;
use sqlx::{Executor, SqlitePool};
use tokio::{sync::mpsc, task::JoinHandle};
use dane_krs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let start = Instant::now();

    let Args
    {
        number_of_requests,
        requests_per_loop
    } = Args::parse();

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

    let (sender, reciever) = mpsc::channel::<JsonResponse>(1);

    let db_task_handle = tokio::spawn(db_task(reciever, pool));

    let client = Client::new();
    let client_ref = unsafe { transmute::<&Client, &'static Client>(&client) };
    
    let mut iter = 0..number_of_requests;
    while !iter.is_empty()
    {
        let mut n_handles = Vec::with_capacity(requests_per_loop * size_of::<JoinHandle<()>>());
        for i in iter.by_ref().take(requests_per_loop)
        {
            n_handles.push(tokio::spawn(send_request(i, client_ref, sender.clone())));
        }
        join_all(n_handles).await;
    }

    drop(sender);
    db_task_handle.await?;

    println!("{:?}", start.elapsed());
    Ok(())
}