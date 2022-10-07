# Prerequisites:

* Rust
* Make

To list project's commands, run `make` or `make help`.

> Note: If you do not use Make, the build will be different than when using it, because the `Makefile` sets environment vars to send to `rustc` to ease code coverage for testing. You can still use Cargo when you want to build for release.

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
