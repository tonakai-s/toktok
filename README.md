# toktok

Status: Early stage

Toktok is supposed to be a self-configured and self-hosted uptime monitoring tool.
You will be able to monitor endpoint and server by simply configuring a YAML file.
With a properly configured YAML, the built-in scheduler and specific executors will check your services for you.
And, of course, if something's not home, you'll be warned!

Toktok purpose is to be simple to configure and start.

---

## Disclaimer
Toktok is not tested in a production environment, feel free to open issues if you pretend use him. (◕ᴥ◕ʋ)

---

## Config constraints
Services of different types have different optional and mandatory fields inside of configuration.

### Service web:
```yaml
configuration:
      type: web
      url: 'https://tuamaeaquelaursa.com'
      expected_http_code: 200
      timeout: 10 # Optional
```

### Service server
```yaml
configuration:
      type: server
      socket: localhost:22 # Could be also a server IP Address
      timeout: 10 # Optional
```

Same with notifications.

### Email notification
```yaml
mailer:
    smtp_credentials: mail.creds.example # File for credentials
    smtp_domain: your.smtp.server.domain.com
    from: from@mail.com
    to: to@mail.com
    cc: # Optional - Must be an array
      - somemail@mail.com
    bcc: # Optional - Must be an array
      - somemail@mail.com
```

For email credentials file, use the format defined in [mail.creds.example](examples/mail.creds.example)

---

## Notes
The interval reset only after a verification has finished, or in case of error only after the notification has been sent.

---

## Logs

All checks are inserted at temp directory of the OS.

Linux: /tmp/toktok
Windows: %TEMP%/toktok

Also, has a tracing wich prints do stdout during the execution, informing if some thread crashed, some channel not permormed his operation as supposed, or if a notification has or not been sent in a service verification error. So when you run it, is a interesting approach redirect the `stdout` to a file yourself.

---

## How to use
To a starter yaml file, you can use this [toktok.example.yaml](examples/toktok.example.yaml).
Notifications are sent only in services reported with error.

You need have Rust lang installed.
If it's not installed, check the installation [here](https://rust-lang.org/), very simple.

Clone this repo:
```bash
$ git clone https://github.com/tonakai-s/toktok.git
```

And start with:
```bash
$ cargo run --release
```

You may want to run with a config file in another path:
```bash
$ cargo run -- -c "<PATH>"
```

Or, if you prefer, you can only build the executable and get it at the target directory:
```bash
$ cargo build --release
```

---

## Next steps

You can check more of what is pretended to be implement and changed in this project at [TODO.md](TODO.md)

---

If you like this project, leave a star, so I know someone is supposed to use it.