#![feature(let_chains)]
#![macro_use]
extern crate tokio;

use regex::Regex;
use scraper::{Html, Selector};
use std::fs;

mod config;

use playwright::Playwright;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = toml::from_str::<config::Config>(&fs::read_to_string("config.toml")?)?;

    let playwright = Playwright::initialize().await?;
    playwright.prepare()?;

    let browser = playwright
        .chromium()
        .launcher()
        .headless(false)
        .launch()
        .await?;
    let context = browser.context_builder().build().await?;
    // login
    let page = context.new_page().await?;
    println!("go to lieferando");
    page.goto_builder("https://www.lieferando.de/")
        .goto()
        .await?;

    // wait for the page to load
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // accept cookies
    let accept_cookies = page
        .query_selector("[aria-label='Alle Cookies akzeptieren']")
        .await?;
    if let Some(accept_cookies) = accept_cookies {
        accept_cookies.click_builder().click().await?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // find burger menu with aria-label "Mein Account" and click it
    let burger_menu = page
        .query_selector("[aria-label='Mein Account']")
        .await?
        .unwrap();
    burger_menu.click_builder().click().await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // find login button that contains a span with 'Anmelden' text and click it
    let buttons = page.query_selector_all("button").await?;

    // for some reason I can't use has-text here
    for button in buttons {
        let text = button.inner_text().await?;
        if text.contains("Anmelden") {
            button.click_builder().click().await?;
            break;
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    println!("loggin in");
    // find input with placeholder "E-Mail"
    let email_input = page
        .query_selector("input[placeholder='E-Mail-Adresse']")
        .await?
        .unwrap();
    email_input.click_builder().click().await?;
    email_input
        .type_builder(&config.lieferando.email)
        .r#type()
        .await?;

    let password_input = page
        .query_selector("input[placeholder='Passwort']")
        .await?
        .unwrap();
    password_input.click_builder().click().await?;
    password_input
        .type_builder(&config.lieferando.password)
        .r#type()
        .await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let login_button = page.query_selector("button[type='submit']").await?.unwrap();
    login_button.click_builder().click().await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("getting otp");

    // check if there is an input with "sicherheitscode" as placeholder
    let security_code_input = page
        .query_selector("input[placeholder='Sicherheitscode']")
        .await?;

    let mut last: Option<String> = None;
    for _ in 0..5 {
        if security_code_input.is_none() {
            break;
        }
        let email = fetch_inbox_top(
            &config.email.server,
            &config.email.username,
            &config.email.password,
        )?;
        if email.is_none() {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
        let email = email.unwrap();
        let otp = extract_otp_from_email(&email)?.unwrap();
        println!("otp: {otp}");
        if let Some(last_otp) = last {
            if last_otp != otp {
                last = Some(otp);
                break;
            }
        }
        last = Some(otp);

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    if let Some(input) = security_code_input && let Some(otp) = last {
        println!("security code input found");
        input.click_builder().click().await?;
        input.type_builder(&otp).r#type().await?;
        let submit_button = page.query_selector("button[type='submit']").await?.unwrap();
        submit_button.click_builder().click().await?;
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("logged in, getting stamp cards");
    page.goto_builder("https://www.lieferando.de/lieferservice/essen/haan-42781#stempelkarten")
        .goto()
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // extract info from stamp cards
    let mut pizza_royal = 0;
    let mut pizza_royal_vouchers = 0.0;

    let stampcards = page.query_selector_all("[data-qa='stamp-card']").await?;
    for stampcard in stampcards {
        let img = stampcard.query_selector("img").await?.unwrap();
        let restaurant_name = img.get_attribute("alt").await?.unwrap();

        if restaurant_name != "Pizza Royal" {
            continue;
        }

        let only_n_left = stampcard
            .query_selector("[data-qa='stamps-to-go-text']")
            .await?;
        if let Some(only_n_left) = only_n_left {
            let only_n_left = only_n_left.inner_text().await?;
            let re = Regex::new(r"Nur noch (\d+) Bestellungen")?;
            let captures = re.captures(&only_n_left).unwrap();
            pizza_royal += 5 - captures[1].parse::<u32>()?;
        } else {
            let copy_voucher_code_button = stampcard
                .query_selector("[data-qa='reveal-and-copy-voucher-button']")
                .await?
                .unwrap();
            let voucher_text = copy_voucher_code_button.inner_text().await?;

            let re = Regex::new(r"Dein (\d*,\d*).€ Gutschein").unwrap();
            let caps = re.captures(&voucher_text).unwrap();
            let voucher_value = caps.get(1).unwrap().as_str().replace(",", ".");
            pizza_royal_vouchers += voucher_value.parse::<f32>()?;
        }
    }
    let mut circles = String::new();
    for _ in 0..pizza_royal {
        circles.push('🟢');
    }
    // pad to length of 5
    for _ in pizza_royal..5 {
        circles.push('⚪');
    }

    if pizza_royal_vouchers > 0.0 {
        println!("{circles} (+{pizza_royal_vouchers}€ vouchers)");
    } else {
        println!("{circles}");
    }

    Ok(())
}

fn fetch_inbox_top(
    server: impl AsRef<str>,
    username: impl AsRef<str>,
    password: impl AsRef<str>,
) -> imap::error::Result<Option<String>> {
    let server = server.as_ref();
    let username = username.as_ref();
    let password = password.as_ref();

    println!("connecting to {server}...");
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((server.clone(), 993), &server, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client.login(username, password).map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // chrono get date in format 01-Jan-2021
    let now = chrono::Utc::now();
    let date = now.format("%d-%b-%Y");

    let command = format!(
        "SUBJECT \"Dein Lieferando.de Sicherheitscode zum Einloggen.\" SINCE \"{}\"",
        &date.to_string()
    );
    println!("searching for messages...");

    // find messages with relevant subject
    let message_ids = imap_session.search(command)?;
    let message_ids = message_ids.iter();

    // sort messages by date
    let mut message_ids_sorted: Vec<_> = message_ids.collect();
    message_ids_sorted.sort_by_key(|&id| {
        let date = imap_session.fetch(id.to_string(), "INTERNALDATE").unwrap();
        let date = date.iter().next().unwrap();
        let date = date.internal_date().unwrap();
        date
    });

    let last = message_ids_sorted.last().unwrap();

    // fetch everything about that message
    let messages = imap_session.fetch(last.to_string(), "RFC822")?;
    let message = messages.iter().next();
    imap_session.logout()?;

    return match message {
        None => Ok(None),
        Some(message) => {
            println!("found a message!");
            let message = message.body().unwrap();
            let message = String::from_utf8_lossy(message);
            Ok(Some(message.to_string()))
        }
    };
}

fn extract_otp_from_email(email: &str) -> anyhow::Result<Option<String>> {
    // Some(otp.to_string())
    // find strong element
    let document = Html::parse_document(email);
    let selector = Selector::parse("strong");
    if let Err(why) = selector {
        eprintln!("error parsing: {why}");
        return Err(anyhow::anyhow!("error parsing: {why}"));
    }
    let selector = selector.unwrap();
    let mut otp: Option<String> = None;
    let element = document.select(&selector).next();
    if let Some(element) = element {
        otp = Some(element.text().collect());
    }
    Ok(otp)
}
