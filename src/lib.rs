use tokio::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{query, Pool, Sqlite};

pub const CREATE_SCRIPT: &str = include_str!("create-table.sql");
pub const ADD_SCRIPT: &str = include_str!("add-db.sql");

#[derive(Deserialize)]
pub struct JsonResponse {odpis: Odpis}
#[derive(Deserialize)]
struct Odpis
{
    #[serde(rename = "naglowekA")]
    naglowek: Naglowek,
    dane: Dane
}
#[derive(Deserialize)]
struct Naglowek
{
    #[serde(rename = "numerKRS")]
    numer_krs: String
}
#[derive(Deserialize)]
struct Dane
{
    #[serde(rename = "dzial1")]
    dzial: Dzial
}
#[derive(Deserialize)]
struct Dzial
{
    #[serde(rename = "danePodmiotu")]
    dane_podmiotu: DanePodmiotu,
    #[serde(rename = "siedzibaIAdres")]
    siedziba_i_adres: SiedzibaIAdres
}
#[derive(Deserialize)]
struct DanePodmiotu
{
    #[serde(rename = "formaPrawna")]
    forma_prawna: Option<String>,
    identyfikatory: Identyfikatory,
    nazwa: String
}
#[derive(Deserialize)]
struct Identyfikatory
{
    regon: Option<String>,
    nip: Option<String>
}
#[derive(Deserialize)]
struct SiedzibaIAdres
{
    siedziba: Siedziba,
    adres: Adres,
    #[serde(rename = "adresDoDoreczenElektronicznychWpisanyDoBAE")]
    adres_doreczen_elektronicznych: Option<String>
}
#[derive(Deserialize)]
struct Siedziba
{
    kraj: Option<String>,
    wojewodztwo: Option<String>,
    powiat: Option<String>,
    gmina: Option<String>,
    miejscowosc: Option<String>
}
#[derive(Deserialize)]
struct Adres
{
    ulica: Option<String>,
    #[serde(rename = "nrDomu")]
    nr_domu: Option<String>,
    #[serde(rename = "kodPocztowy")]
    kod_pocztowy: Option<String>,
    poczta: Option<String>
}

pub async fn db_task_handle(mut reciever: Receiver<JsonResponse>, pool: Pool<Sqlite>)
{
    while let Some(data) = reciever.recv().await
    {
        let JsonResponse {
            odpis: Odpis {
                naglowek: Naglowek { numer_krs },
                dane: Dane {
                    dzial: Dzial {
                        dane_podmiotu: DanePodmiotu {
                            forma_prawna,
                            identyfikatory: Identyfikatory { regon, nip },
                            nazwa
                        },
                        siedziba_i_adres: SiedzibaIAdres {
                            siedziba: Siedziba {
                                kraj,
                                wojewodztwo,
                                powiat,
                                gmina,
                                miejscowosc
                            },
                            adres: Adres {
                                ulica,
                                nr_domu,
                                kod_pocztowy,
                                poczta
                            },
                            adres_doreczen_elektronicznych
                        }
                    }
                }
            }
        } = data;
        match query(ADD_SCRIPT)
            .bind(numer_krs)
            .bind(forma_prawna)
            .bind(regon)
            .bind(nip)
            .bind(nazwa)
            .bind(kraj)
            .bind(wojewodztwo)
            .bind(powiat)
            .bind(gmina)
            .bind(miejscowosc)
            .bind(ulica)
            .bind(nr_domu)
            .bind(kod_pocztowy)
            .bind(poczta)
            .bind(adres_doreczen_elektronicznych)
            .execute(&pool)
            .await
        {
            Ok(_) => (),
            Err(err) => eprintln!("Could not save to the DB ({err})")
        }
    }
}

pub async fn send_request(number: usize, client: Arc<Client>, sender: Sender<JsonResponse>)
{
    match client.get(format!("https://api-krs.ms.gov.pl/api/krs/OdpisAktualny/{number}?rejestr=P&format=json"))
        .send()
        .await
    {
        Ok(response) =>
        {
            if response.status() != 200 {return;}
            match response.json::<JsonResponse>().await
            {
                Ok(json) =>
                {
                    println!("Found {number} 🙂");
                    sender.send(json).await.unwrap();
                }
                Err(err) => eprintln!("{}", err.to_string().trim())
            }
        }
        Err(_) => Box::pin(send_request(number, client, sender)).await
    }
}