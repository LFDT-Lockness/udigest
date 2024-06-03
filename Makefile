.PHONY: docs docs-open

docs:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features

docs-open:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features --open

docs-private:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features --document-private-items

readme:
	cargo readme -i src/lib.rs -r udigest/ -t ../docs/readme.tpl \
		| perl -ne 's/\[(.+?)\]\((?!https).+?\)/\1/g; print;' \
		| perl -ne 's/(?<!#)\[(.+?)\](?!\()/\1/g; print;' \
		> README.md
	cargo readme -i src/lib.rs -r udigest-derive/ -t ../docs/readme.tpl \
		> udigest-derive/README.md
