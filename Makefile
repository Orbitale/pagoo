
##
## Project commands
##

.DEFAULT_GOAL := help
help: ## Show this help message
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf " \033[32m%-25s\033[0m%s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'
.PHONY: help

build: ## Build the project
	cargo build --release
.PHONY: build

start-webhook: ## Start the release version of the webhook API
	cargo run --release -- --config-file samples/json_sample.json serve:webhook 2>&1
.PHONY: start-webhook

# TODO: check if we can run the tool in the background so we can run curl-based tests
#curl -i http://127.0.0.1:8000/webhook -X POST -d $$(jq -c -M '.webhooks[0].matchers[0][\"match-json-body\"]' samples/json_sample.json)
test: ## Run the tests
	cargo test --no-fail-fast
.PHONY: test
