
# 📬 Relay

Relay messages into your inbox via HTTP.

## 🛠️ Installation

### Cargo

```sh
$ cargo install --path .
```

### Nix

#### Declarative

```nix
environment.systemPackages = [
  inputs.relay.packages.<arch>.default
];
```

#### Imperative

```sh
$ nix profile install github:kludge-cs/relay
```

## 📝 Environment Variables

| Name        | Description                   | Default     |
| ----------- | ----------------------------- | ----------- |
| `HOST`      | Host to bind to               | `127.0.0.1` |
| `PORT`      | Port to bind to               | `8080`      |
| `SMTP_PORT` | SMTP port to connect to       | `587`       |
| `SMTP_HOST` | SMTP host to connect to       | error       |
| `SMTP_USER` | SMTP user to connect as       | error       |
| `SMTP_PASS` | SMTP password to connect with | error       |

## 📝 Usage

TODO: Write Nix module.

## 🧩 Development

```sh
$ nix develop # If Nix
```
