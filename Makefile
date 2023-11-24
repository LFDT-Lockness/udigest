readme:
	cargo readme -i src/lib.rs -r udigest/ -t ../docs/readme.tpl \
		| perl -ne 's/\[(.+?)\]\(.+?\)/\1/g; print;' \
		| perl -ne 's/(?<!#)\[(.+?)\]/\1/g; print;' \
		> README.md
	cargo readme -i src/lib.rs -r udigest-derive/ -t ../docs/readme.tpl \
		> udigest-derive/README.md
