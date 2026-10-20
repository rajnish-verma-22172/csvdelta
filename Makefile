.PHONY: test docker-test

test:
	cargo test

docker-test:
	docker build -t csvdelta:test .
