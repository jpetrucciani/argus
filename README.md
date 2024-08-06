# argus

[![uses nix](https://img.shields.io/badge/uses-nix-%237EBAE4)](https://nixos.org/)

`argus` is a HTTP request logger built with Rust. It provides a flexible way to log incoming HTTP requests and customize response headers/body/codes. It also provides prometheus metrics by default on `/metrics`!

It is named for [Argus Panoptes](https://en.wikipedia.org/wiki/Argus_Panoptes), a many-eyed giant in Greek Mythology, sometimes referred to as `All-seeing`.

## Features

- Log incoming HTTP requests with customizable filters
- Customize response headers, body, and status code
- Collect and expose Prometheus-compatible metrics
- Flexible configuration through command-line arguments or environment variables

## Installation

To build and run `argus`, you need to have [nix](https://nixos.org/) + [direnv](https://github.com/direnv/direnv), or Rust and Cargo installed on your system.

1. Clone this repository:

```bash
git clone https://github.com/yourusername/argus.git
cd argus
```

1. Build the project:

```bash
cargo build --release
```

## Usage

Run Argus with default settings:

```bash
./target/release/argus
```

Or use command-line options to customize its behavior:

```bash
./target/release/argus --listen-addr 127.0.0.1 --port 3000 --response-body "Custom response"
```

## Options

Argus can be configured using command-line arguments or environment variables. Here's a table of all available options:

| Flag                   | Environment Variable       | Default   | Description                                                 |
| ---------------------- | -------------------------- | --------- | ----------------------------------------------------------- |
| `--listen-addr`        | `ARGUS_IP`                 | `0.0.0.0` | IP address to listen on                                     |
| `--port`               | `ARGUS_PORT`               | `8080`    | Port to listen on                                           |
| `--response-headers`   | `ARGUS_RESPONSE_HEADERS`   | (None)    | Custom response headers (format: "Key1:Value1,Key2:Value2") |
| `--response-body`      | `ARGUS_RESPONSE_BODY`      | (None)    | Custom response body text                                   |
| `--response-body-file` | `ARGUS_RESPONSE_BODY_FILE` | (None)    | Path to file containing custom response body                |
| `--filter-routes`      | `ARGUS_FILTER_ROUTES`      | (None)    | Routes to filter (comma-separated)                          |
| `--filter-methods`     | `ARGUS_FILTER_METHODS`     | (None)    | HTTP methods to filter (comma-separated)                    |
| `--response-status`    | `ARGUS_RESPONSE_STATUS`    | `200`     | Custom response status code                                 |
| `--disable-metrics`    | `ARGUS_DISABLE_METRICS`    |           | Disable Prometheus metrics collection                       |

## Metrics

When metrics are enabled (default), Argus exposes Prometheus-compatible metrics at the `/metrics` endpoint. The following metrics are available:

- `total_requests`: Total number of requests received
- `requests_by_method`: Number of requests by HTTP method
- `responses_by_status`: Number of responses by HTTP status code

To view the metrics, send a GET request to the `/metrics` endpoint:

```bash
curl http://localhost:8080/metrics
```

## Examples

Run Argus on a specific IP and port:

```bash
./argus --listen-addr 127.0.0.1 --port 3000
```

Set custom response headers and body:

```bash
./argus --response-headers "X-Custom-Header:Value1,Content-Type:application/json" --response-body '{"status": "ok"}'
```

Filter specific routes and methods:

```bash
./argus --filter-routes "/api,/public" --filter-methods "GET,POST"
```

Use a custom response status code:

```bash
./argus --response-status 201
```

Disable metrics collection:

```bash
./argus --disable-metrics
```

### Cooler examples

Use argus in a bash pipeline!

```bash
# pretty print incoming request logs
./argus | jq
```

## Demo

![argus_demo](https://cobi.dev/static/img/github/gif/argus-0.1.0.gif)
