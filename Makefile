
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

test: ## Run the tests
	cargo test --no-fail-fast --release -- --show-output
.PHONY: test

tests: test # Mostly here for people confusing plural and singular
.PHONY: tests
