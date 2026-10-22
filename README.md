# iptv-debug

A small command-line tool that shows which Xtream Codes servers a Purple TV
login code points to.

IPTV apps built on the Purple TV platform let you sign in with a short code
plus a username and password. Behind the scenes the app looks the code up,
receives a list of servers ("DNS") and tries your credentials against each one.
This tool does the same lookup and prints that server list, so you can use the
server URL with any Xtream-compatible player (TiviMate, IPTV Smarters, etc.).

## How it works

1. Requests `https://purpletvapi.b-cdn.net/appdetail?version=v2&app_type=ios&code=<CODE>`,
   the same call the app makes at login.
2. The response is JSON wrapped in several layers of base64; the tool decodes
   layers until the JSON parses (up to `--max-layers`).
3. Prints every server in the `app_dns` list (including per-content overrides
   for live, EPG, movies, series and catch-up when they differ), the provider's
   test server, and the de-duplicated set of domains.

## Requirements

- [Rust](https://rustup.rs/) (edition 2024, Rust 1.85 or newer)

## Usage

Run from a clone of this repository:

```sh
git clone https://github.com/OWNER/iptv-debug.git
cd iptv-debug
cargo run -- <CODE>
```

Or install it directly from GitHub:

```sh
cargo install --git https://github.com/OWNER/iptv-debug.git
iptv-debug <CODE>
```

### Options

```
Usage: iptv-debug [OPTIONS] <CODE>

Arguments:
  <CODE>  Login code entered in the app

Options:
      --max-layers <MAX_LAYERS>  Maximum number of base64 layers to peel off the response [default: 10]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Example output

```
App: ExampleTV (mode: Xstream)

Servers:
  Main            http://server-one.example
  Backup          http://server-two.example
  (test server)   http://demo.example

Unique domains (3):
  http://demo.example
  http://server-one.example
  http://server-two.example
```

## Using the result

Your username and password are not returned by the lookup; they are the ones
you already use in the app. Not every listed server will accept them, so try
each one. A working login on an Xtream server gives you:

| Player field | Value                                  |
|--------------|----------------------------------------|
| Server URL   | one of the domains above, e.g. `http://server-one.example` |
| Username     | your username                          |
| Password     | your password                          |

For players that only accept a playlist URL:

```
<SERVER>/get.php?username=<USER>&password=<PASS>&type=m3u_plus&output=ts
```

EPG:

```
<SERVER>/xmltv.php?username=<USER>&password=<PASS>
```

Keep in mind that most subscriptions limit concurrent connections, so the
original app and a second player may not be able to stream at the same time.

## Limitations

- Only works for apps built on the Purple TV platform. Providers without code
  login need a different approach (e.g. capturing the app's traffic with a
  proxy such as Fluxzy and looking for `player_api.php` requests).
- The `appdetail` endpoint and response format are undocumented and may change.
