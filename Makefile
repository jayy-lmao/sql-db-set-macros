.PHONY: test test-unit test-integration

test: test-unit test-integration

test-unit:
	@cargo nextest run 

setup-db-migrations:
 @cargo install sqlx-cli --no-default-features --features native-tls,postgres --version 0.8.3


test-integration:
	@echo "Starting temporary Postgres container for integration tests..."
	@docker run -d --name test-postgres-$$(date +%s) \
		-e POSTGRES_PASSWORD=testpass \
		-e POSTGRES_USER=testuser \
		-e POSTGRES_DB=testdb \
		-p 0:5432 \
		postgres:15 > /tmp/container_id
	@sleep 5
	@CONTAINER_ID=$$(cat /tmp/container_id) && \
	PORT=$$(docker port $$CONTAINER_ID 5432/tcp | cut -d: -f2) && \
	export DATABASE_URL="postgresql://testuser:testpass@localhost:$$PORT/testdb" && \
	echo "Running migrations..." && \
	cd tests && DATABASE_URL=$$DATABASE_URL sqlx migrate run && \
	echo "Running tests with DATABASE_URL=$$DATABASE_URL" && \
	DATABASE_URL=$$DATABASE_URL cargo nextest run; \
	TEST_RESULT=$$?; \
	echo "Cleaning up container $$CONTAINER_ID..."; \
	docker stop $$CONTAINER_ID > /dev/null 2>&1; \
	docker rm $$CONTAINER_ID > /dev/null 2>&1; \
	rm -f /tmp/container_id; \
	exit $$TEST_RESULT 

