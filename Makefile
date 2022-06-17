RELEASE ?= 0

##
##==================
## Project commands
##==================
##

ifeq ($(RELEASE),1)
	TARGET=--release
else
	TARGET=
endif

.DEFAULT_GOAL := help
help: ## Show this help message
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf " \033[32m%-25s\033[0m%s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'
.PHONY: help

start-webhook: ## Start the release version of the webhook API
	cargo run $(TARGET) -- --config-file samples/json_sample.json serve:webhook 2>&1
.PHONY: start-webhook

build: ## Build the project
	cargo build $(TARGET)
.PHONY: build

##
##=========
## Testing
##=========
##
build-test: ## Build the test modules	(alias: build-tests)
	cargo test --no-run $(TARGET)
.PHONY: build-tests

build-tests: build-test # Alias
.PHONY: build-tests

test: ## Run the tests			(alias: tests)
	cargo test --no-fail-fast $(TARGET) -- --show-output --nocapture
.PHONY: test

tests: test # Alias
.PHONY: tests

##
##----
##
##Note:
##All commands are executed in DEV mode.
##To run the commands in RELEASE mode, use the "RELEASE=1" env var.
##
##For example:
##$ make RELEASE=1 build
##
