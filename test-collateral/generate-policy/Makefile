# Makefile for the Veracruz JSON policy generation utility.
#
# AUTHORS
#
# The Veracruz Development Team.
#
# COPYRIGHT AND LICENSING
#
# See the `LICENSING.markdown` file in the Veracruz root directory for
# licensing and copyright information.

.PHONY: all doc clean fmt

all:
	cargo build --release

doc:
	cargo doc

fmt:
	cargo fmt

clean:
	cargo clean
	rm -f Cargo.lock

