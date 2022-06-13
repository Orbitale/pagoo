
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
	cargo test --no-fail-fast
.PHONY: test

test-running: ## Run the tests while the binary is running
	@# TODO: check if we can run the tool in the background so we can run curl-based tests
	@curl --silent -i http://127.0.0.1:8000/webhook -X POST -d $$(jq -c -M '.webhooks[0].matchers[0]["match-json-body"]' samples/json_sample.json) > test_logs.txt 2>&1
	@OUT_LOG=$$(head -n 1 test_logs.txt) ;\
	if [ "$$OUT_LOG" != "HTTP/1.1 201 Created" ]; then \
		echo "Test failed: $$OUT_LOG" ;\
		exit 1 ;\
	else \
		echo "Test passed" ;\
	fi
	@rm test_logs.txt
.PHONY: test-running