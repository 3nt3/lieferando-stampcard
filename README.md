# lieferando-stampcard

Scrapes lieferando for stampcards

## Example `config.toml`

```toml
headless = false # doesn't work when true for some reason

[lieferando]
email = "asdf@asdf.com"
password = "whatever"
restaurant_name = "Pizza Royal"

[email]
server = "imap.gmail.com"
username = "asdf@asdf.com"
password = "whatever"
```

## Usage

```
$ cargo r
🟢⚪⚪⚪⚪ (+4.75€ vouchers)
```

signaling that your current stamp card is 1/5 full and you have a finished one with a 4.75€ voucher
