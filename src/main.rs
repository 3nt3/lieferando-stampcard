#[macro_use]
extern crate tokio;

use playwright::Playwright;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dbg!(fetch_inbox_top()?);
    return Ok(());

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
        dbg!(&text);
        if text.contains("Anmelden") {
            button.click_builder().click().await?;
            break;
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // find input with placeholder "E-Mail"
    let email_input = page
        .query_selector("input[placeholder='E-Mail-Adresse']")
        .await?
        .unwrap();
    email_input.click_builder().click().await?;
    email_input
        .type_builder("3nt3.de+lieferando@gmail.com")
        .r#type()
        .await?;

    let password_input = page
        .query_selector("input[placeholder='Passwort']")
        .await?
        .unwrap();
    password_input.click_builder().click().await?;
    password_input
        .type_builder("n0nlxIOgEW5cjck4zoPrDXOA7nUesw9I")
        .r#type()
        .await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let login_button = page.query_selector("button[type='submit']").await?.unwrap();
    login_button.click_builder().click().await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // check if there is an input with "sicherheitscode" as placeholder
    let security_code_input = page
        .query_selector("input[placeholder='Sicherheitscode']")
        .await?;
    if let Some(input) = security_code_input {
        println!("security code input found");
        input.click_builder().click().await?;
        input.type_builder("123456").r#type().await?;
        let submit_button = page.query_selector("button[type='submit']").await?.unwrap();
        submit_button.click_builder().click().await?;
    }

    page.goto_builder("https://www.lieferando.de/lieferservice/essen/haan-42781#stempelkarten")
        .goto()
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}

fn fetch_inbox_top() -> imap::error::Result<Option<String>> {
    let domain = "imap.gmail.com";
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login("3nt3.de@gmail.com", "qldaywryjtxrujgy")
        .map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // find messages with relevant subject
    let message_ids =
        imap_session.search("SUBJECT \"Dein Lieferando.de Sicherheitscode zum Einloggen.\"")?;
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
            let message = message.body().unwrap();
            let message = String::from_utf8_lossy(message);
            Ok(Some(message.to_string()))
        }
    };
}
