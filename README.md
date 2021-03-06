# Prerequisites:

* Rust
* Make

To list project's commands, run `make` or `make help`.

# Roadmap:

> Legend:
> * π© = Implemented
> * π¨ = In progress / Partially implemented
> * π₯ = Not yet implemented, but will be
> * β = Idea for the future (may be hard to implement, hence the question mark)

* π© HTTP Server
  * π© Create an HTTP entrypoint for the webhook listener 
  * π₯ Create an HTTP entrypoint for the configuration update
  * π₯ Secure the config update entrypoint with an authentication system
  * β Allow multiple instances to be started
* π¨ Configuration
  * π© JSON config file 
  * π₯ Allow runtime updates of the configuration
  * β Think about other storages than a single file
* π¨ Webhook matching
  * π© Strategy based on list of matchers 
  * π© Matching by HTTP headers
  * π© Matching by JSON body
  * β Allow partial/strategy inside a single matcher, instead of having to rely on multiple matchers with the `one` strategy
  * β Allow filtering JSON body with string/regex matching (pretty hard though, since the entire JSON has to be traversed, but could be neat)
* π¨ Executor worker
  * π© Create a separate thread only to listen for actions to execute (the "queue" system) 
  * π© Allow executing processes in the worker thread
  * β Think about concurrency when a lot of actions have to be executed at the same time
* π₯ Analytics
  * π₯ Make most app structures more exhaustive (names, ids, logs...)
  * π₯ Store the logs (sqlite database, json-based log fileβ)
* π₯ App frontend
  * π₯ Create a separate command to spawn a frontend app
  * π₯ Create a dashboard to visualize logs
  * π₯ Create a dashboard to visualize configuration
  * π₯ Create a simple system to update app's configuration at runtime
  * β Allow to manage multiple running instances of the webhook listener 

# Reminder of the project's workflow graph

[![Application graph](./docs/Architecture.svg)](./docs/Architecture.svg)
