SHELL=bash

RELEASE ?= 0

# Helper variables for display
_INFO := "\033[32m[%s]\033[0m %s\n" # Green text
_ERROR := "\033[31m[%s]\033[0m %s\n" # Red text

ifeq ($(RELEASE),1)
	TARGET=--release
else
	TARGET=
endif

## Necessary for coverage, doesn't impact compile-time too much (yet?).
export RUSTFLAGS := -Cinstrument-coverage
export LLVM_PROFILE_FILE := target/coverage/pagoo-%p-%m.profraw

##
##==================
## Project commands
##==================
##

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
	@printf $(_INFO) "INFO" "Building test modules..."
	@cargo test --no-run $(TARGET)
	@printf $(_INFO) "INFO" "✅ Done building test modules!"
.PHONY: build-test

build-tests: build-test # Alias
.PHONY: build-tests

test: build-test ## Run the tests			(alias: tests)
	@printf $(_INFO) "INFO" "Removing coverage artifacts..."
	@rm -rf target/coverage/*
	@cargo test --no-fail-fast $(TARGET) -- --show-output --nocapture
.PHONY: test

tests: test # Alias
.PHONY: tests

COVERAGE=html
COVERAGE_FLAGS=-t html -o ./target/coverage-html/
ifeq ($(COVERAGE),lcov)
	COVERAGE_FLAGS=-t lcov -o ./target/lcov.info
endif

coverage: ## Generate code coverage based on the test output. You can specify LCOV coverage with "COVERAGE=lcov"
	@if [[ -n "$(find ./target/coverage/ -maxdepth 1 -name '*.profraw' -print -quit)" ]]; then \
		printf $(_ERROR) "ERROR" "No coverage data available..." ;\
		exit 1 ;\
	fi

	@if ! command -v grcov &> /dev/null; then \
		printf $(_ERROR) "ERROR" "The grcov program does not seem to be installed on your machine." ;\
		printf $(_ERROR) "ERROR" "You can install it via \"cargo install grcov\" to generate code coverage." ;\
		exit 1 ;\
	fi

	@printf $(_INFO) "INFO" "Generating $(COVERAGE) coverage..."

	@grcov \
		target/coverage/ \
		--excl-line "#\[cfg\(test" \
		--excl-br-start "mod tests \{" --excl-start "mod tests \{" \
		--ignore src/logging.rs \
		--ignore src/commands/* \
		--ignore src/webhook/* \
		-s . \
		--binary-path ./target/debug/ \
		--branch \
		--ignore-not-existing \
		$(COVERAGE_FLAGS)

	@printf $(_INFO) "INFO" "✅ Done!"
.PHONY: coverage

##
##----
##
##Note:
##All commands are executed in "debug" mode.
##To run the commands in "release" mode, use the "RELEASE=1" env var.
##
##For example:
##$ make RELEASE=1 build
##
