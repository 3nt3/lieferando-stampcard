# lieferando-stampcard

Scrapes lieferando for Pizza Royal stampcards

## Example `config.toml`

```toml
[lieferando]
email = "asdf@asdf.com"
password = "whatever"

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
