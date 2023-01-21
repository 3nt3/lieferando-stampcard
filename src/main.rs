#[macro_use]
extern crate tokio;

use playwright::{Playwright};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?;

    let browser = playwright.chromium().launcher().headless(false).launch().await?;
    let context = browser.context_builder().build().await?;
    // login
    let page = context.new_page().await?;
    page.goto_builder("https://www.lieferando.de/").goto().await?;

    // wait for the page to load
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // accept cookies
    let accept_cookies = page.query_selector("[aria-label='Alle Cookies akzeptieren']").await?;
    if let Some(accept_cookies) = accept_cookies {
        accept_cookies.click_builder().click().await?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // find burger menu with aria-label "Mein Account" and click it
    let burger_menu = page.query_selector("[aria-label='Mein Account']").await?.unwrap();
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
    let email_input = page.query_selector("input[placeholder='E-Mail-Adresse']").await?.unwrap();
    email_input.click_builder().click().await?;
    email_input.type_builder("3nt3.de+lieferando@gmail.com").r#type().await?;

    let password_input = page.query_selector("input[placeholder='Passwort']").await?.unwrap();
    password_input.click_builder().click().await?;
    password_input.type_builder("n0nlxIOgEW5cjck4zoPrDXOA7nUesw9I").r#type().await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let login_button = page.query_selector("button[type='submit']").await?.unwrap();
    login_button.click_builder().click().await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // check if there is an input with "sicherheitscode" as placeholder
    let security_code_input = page.query_selector("input[placeholder='Sicherheitscode']").await?;
    if let Some(input) = security_code_input {
        println!("security code input found");
        input.click_builder().click().await?;
        input.type_builder("123456").r#type().await?;
        let submit_button = page.query_selector("button[type='submit']").await?.unwrap();
        submit_button.click_builder().click().await?;
    }


    page.goto_builder("https://www.lieferando.de/lieferservice/essen/haan-42781#stempelkarten").goto().await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}

