# Lynx

> [!CAUTION]
> **This project is made for me, my needs, and my infrastructure.**
>
> No support will be offered for this software. Breaking changes to functionalty or features may be made any time.

Declarative and lightweight URL shortner service.

## Setup

### With Docker

1. Copy [compose.yml](./compose.yml) to a local file named `compose.yml` or add the
   service to your existing stack and fill in the environment variables.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

2. Start the stack

```sh
docker compose up -d
```

### With Cargo

1. Install with

    ```sh
    cargo install --git https://codeberg.org/Blooym/lynx.git
    ```

2. Set configuration values as necessary. Information about configuration options can be found in the [configuration](#configuration) section.

3. Run the server
    ```sh
    lynx <flags>
    ```

## Link Configuration

Links are configured declaratively via a `.toml` file that is passed to the server via the `--config` flag. This file is automatically watched for changes and Lynx will automatically reload all links whenever it is changed. Whenever an invalid change is made to the configuration, the last working version will be used instead.

### Schema

The configuration file schema is as follows:

```
[links.<short_url_id>]
# Where this id should redirect to. Should be a fully qualified URL with a scheme.
redirect = <url>
# Whether the link is currently disabled and should be treated like it does not exist.
# Defaults to false.
disabled = <bool>
# A UNIX-Timestamp in seconds specifying when the link will be invalid.
# Defaults to never.
expire_after = <unix_timestamp_ms>
```

### Examples

Redirect `/example` to `https://example.com`

```toml
[links.example]
redirect = "https://example.com"
```

Rediect `/example` to `https://example.com` make the shortend URL invalid after `Mon Sep 29 2025 23:54:57 UTC+0`

```toml
[links.example]
redirect = "https://example.com"
expire_after = 1759190097
```

### Notes

Lynx reserves the `api` keyword for future internal use. Any configuration containing it will automatically be rejected.

## Server Configuration

Lynx is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `lynx --help` to get an up-to-date including default values.

| Name               | Description                                                                                                                      | Flag        | Env                 | Default          |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------- | ----------- | ------------------- | ---------------- |
| Address            | Internet socket address that the server should run on.                                                                           | `--address` | `LYNX_ADDRESS` | `127.0.0.1:5621` |
| Configuration Path | Path to the configuration file. This file will be watched for changes and the configuration will be live-reloaded automatically. | `--config`  | `LYNX_CONFIG`       | -                |
