Pagoo
=====

> (Note: name to be subjet to change because this one might be too silly)

A single-executable to listen to HTTP calls and execute shell scripts based on the request content.

# Prerequisites:

* Rust
* Make (for code coverage)

To list project's commands, run `make` or `make help`.

> Note: If you do not use Make, the build will be different than when using it, because the `Makefile` sets environment vars to send to `rustc` to ease code coverage for testing. You can still use Cargo when you want to build for release.

# Installation

* Via Cargo:
  ```bash
  git clone https://github.com/Orbitale/pagoo pagoo-src
  cargo build --manifest-path=./pagoo-src/Cargo.toml --release
  cp ./pagoo-src/target/release/pagoo pagoo
  rm -rf ./pagoo-src/
  # Use the "pagoo" binary 👍
  ```

# Configuration

Run `pagoo init` to create a default configuration file `.pagoo.json` in the current directory.

The goal of the configuration file is to determine the list of webhooks that will be listened to by Pagoo, and you can configure each webhook with **matchers**.

Once an HTTP request is posted to the webhook, if a **matcher** corresponds to the request, it will execute the command specified in `actions-to-execute` webhook option.

Here is a sample configuration file with the options you can use:

```js
{
    "webhooks":
    // The array with the webhooks
    [ 
        {
            // A unique name for your webhook. Used for referencing and logging.
            "name": "webhook_1",

            // Can be "one" or "all".
            // Determines if one single matcher is enough to trigger the webhook,
            // or if all matchers have to be detected to trigger the webhook.
            "matchers-strategy": "one",  
          
            // Each matcher can contain one or multiple of these available matchers:
            // - match-json-body: will check if the JSON payload contains the specified JSON parts.
            //   This does not check if the two json strings are equal, only if this part is present in the whole payload.
            // - match-headers: will check if specified HTTP header contains the exact value specified.
            "matchers": [
                {
                    "match-json-body": {
                        "some-json-key": "some-json-value"
                    }
                },
                {
                    "match-headers": {
                        "x-some-http-header": "some-header-value",
                    }
                }
            ],
          
            // The shell command to execute when the webhook is triggered.
            // (⚠️ Warning: more options will be added, like the shell to execute the command, current working directory, 
            //   environment variables, etc., remember this is still a work in progress 😉) 
            "actions-to-execute": ["echo", "success!"]
        }
    ]
}
```

# Usage

To start the server, run `pagoo serve:webhook`. Add the `--help` option to see the different parameters (HTTP host, port, etc.).

To trigger the webhook, you must make a `POST` HTTP request to Pagoo's `/webhook` endpoint.

When you do so, Pagoo will compare the request with all your configured webhooks, and the first one matching the current request will trigger and execute a command. (Note: supporting multiple webhooks matching is not yet supported). 

# Roadmap:

> Legend:
> * 🟩 = Implemented
> * 🟨 = In progress / Partially implemented
> * 🟥 = Not yet implemented, but will be
> * ❓ = Idea for the future (may be hard to implement, hence the question mark)

* 🟩 HTTP Server
  * 🟩 Create an HTTP entrypoint for the webhook listener 
  * 🟥 Secure the config update entrypoint with an authentication system
  * ❓ Create an HTTP entrypoint for the configuration update
  * ❓ Allow multiple instances to be started
* 🟨 Configuration
  * 🟩 Create an `init` command to create a boilerplate of configuration file. 
  * 🟩 JSON config file 
  * ❓ Allow runtime updates of the configuration
  * ❓ Think about other storages than a single file
* 🟨 Webhook matching
  * 🟩 Strategy based on list of matchers 
  * 🟩 Matching by HTTP headers
  * 🟩 Matching by JSON body
  * ❓ Allow partial/strategy inside a single matcher, instead of having to rely on multiple matchers with the `one` strategy
  * ❓ Allow filtering JSON body with string/regex matching (pretty hard though, since the entire JSON has to be traversed, but could be neat)
* 🟨 Executor worker
  * 🟩 Create a separate thread only to listen for actions to execute (the "queue" system) 
  * 🟩 Allow executing processes in the worker thread
  * ❓ Think about concurrency when a lot of actions have to be executed at the same time
* 🟥 Analytics
  * 🟩 Store the logs (sqlite database, json-based log file❓)
  * 🟥 Create an HTTP entrypoint to get the logs
  * 🟥 Secure the HTTP entrypoint that delivers logs
* 🟥 App frontend
  * 🟥 Create a separate command to spawn a frontend app
  * 🟥 Create a dashboard to visualize logs
  * 🟥 Create a dashboard to visualize configuration
  * 🟥 Create a simple system to update app's configuration at runtime
  * ❓ Allow to manage multiple running instances of the webhook listener 

# Reminder of the project's workflow graph

[![Application graph](./docs/Architecture.svg)](./docs/Architecture.svg)
