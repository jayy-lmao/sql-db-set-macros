.PHONY: test test-unit test-integration test-postgres test-mysql

test: test-postgres test-mysql

test-postgres: test-unit-postgres test-integration-postgres

test-mysql: test-unit-mysql test-integration-mysql

test-unit-postgres:
	cargo nextest run --features postgres

test-unit-mysql:
	cargo nextest run --features mysql

test-integration-postgres:
	cd tests && cargo nextest run --features postgres

test-integration-mysql:
	cd tests && cargo nextest run --features mysql
