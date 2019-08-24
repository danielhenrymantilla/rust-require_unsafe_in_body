.PHONY: default all test doc help

CARGO=cargo +nightly

default: help

all: test doc

test:
	$(CARGO) test --features unit-tests
	$(CARGO) test --features nightly

doc:
	$(CARGO) doc --features nightly --open

clean:
	cargo clean

help:
	@printf "Usage: make <targets>\n"
	@printf "\n"
	@printf "Targets:\n"
	@printf "\t%-8s  %s\n" "test" "Run the tests"
	@printf "\t%-8s  %s\n" "doc" "Generate the documentation"
	@printf "\t%-8s  %s\n" "all" "test + doc"
	@printf "\t%-8s  %s\n" "clean" "Remove the generated files"
	@printf "\t%-8s  %s\n" "help" "Display this help message"
