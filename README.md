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

```toml
[links.<short_url_id>]
# Where this id should redirect to. Should be a fully qualified URL with a scheme.
redirect = <url>

# Whether the link is currently disabled and should be treated like it does not exist.
# Defaults to false.
disabled = <bool>

# A UNIX-Timestamp in seconds specifying when the link will be invalid.
# Defaults to never.
invalid_after = <unix_timestamp_seconds>

# Controls how path components after the short link ID are treated.
# Options: "None", "Path", "PathPreserveQuery"
# Defaults to "None".
append_mode = <string>
```

#### Append Modes

- **`None`** (default): Don't append any components to the redirect URL. The redirect URL is used as-is regardless of any trailing path.
- **`Path`**: Append the path component after the short link ID to the redirect URL. Query parameters from the redirect URL are not preserved.
- **`PathPreserveQuery`**: Append the path component after the short link ID to the redirect URL and preserve any query parameters from the redirect URL.

### Examples

#### Basic redirect

Redirect `/example` to `https://example.com`:

```toml
[links.example]
redirect = "https://example.com"
```

#### Redirect with expiration

Redirect `/example` to `https://example.com` and make the shortened URL invalid after `Mon Sep 29 2025 23:54:57 UTC+0`:

```toml
[links.example]
redirect = "https://example.com"
invalid_after = 1759190097
```

#### Path appending

Allow `/docs/api/reference` to redirect to `https://example.com/documentation/api/reference`:

```toml
[links.docs]
redirect = "https://example.com/documentation"
append_mode = "Path"
```

To preserve query parameters from the redirect URL, use PathPreserveQuery. For example, `/search/results` would redirect to `https://example.com/search/results?source=shortlink`:

```toml
[links.search]
redirect = "https://example.com/search?source=shortlink"
append_mode = "PathPreserveQuery"
```

#### Disabled link

Create a link that is temporarily disabled:

```toml
[links.maintenance]
redirect = "https://example.com"
disabled = true
```

### Notes

- The `api` keyword is reserved for future internal use and cannot be used as a Link ID.
- Link IDs cannot contain whitespace, forward slashes (`/`), or backslashes (`\`) and will fail to validate.

## Server Configuration

Lynx is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `lynx --help` to get an up-to-date including default values.

| Name               | Description                                                                  | Flag        | Env            | Default          |
| ------------------ | ---------------------------------------------------------------------------- | ----------- | -------------- | ---------------- |
| Address            | Internet socket address that the server should run on.                       | `--address` | `LYNX_ADDRESS` | `127.0.0.1:5621` |
| Configuration Path | Path to the configuration file. Changes will automatically trigger a reload. | `--config`  | `LYNX_CONFIG`  |                  |
