SHELL := /bin/bash

# Configurable via environment or override on CLI: make qps BENCH_OPS=5000 BENCH_CONC=512
BENCH_URL ?= http://127.0.0.1:8080
BENCH_OPS ?= 10000
BENCH_CONC ?= 256
CARGO ?= cargo

.PHONY: migrate
migrate:
	@echo "Applying DB migrations"
	@$(CARGO) run --quiet --bin migrate

.PHONY: prepare
prepare:
	@echo "Preparing inventory (PREP_STOCK=$${PREP_STOCK:-500000} PREP_PROBABILITY=$${PREP_PROBABILITY:-100})"
	@$(CARGO) run --quiet --bin db_prepare

.PHONY: serve
serve:
	@echo "Starting server (Ctrl+C to stop)"
	@$(CARGO) run --bin fast-lottery-engine

.PHONY: qps
qps:
	@echo "Running QPS bench against $(BENCH_URL) ops=$(BENCH_OPS) conc=$(BENCH_CONC)"
	@BENCH_URL="$(BENCH_URL)" BENCH_OPS="$(BENCH_OPS)" BENCH_CONC="$(BENCH_CONC)" \
		$(CARGO) run --quiet --bin qps_bench

.PHONY: bench-local
bench-local:
	@set -euo pipefail; \
	 echo "Launching server in background..."; \
	 ($(CARGO) run --bin fast-lottery-engine >/tmp/fast-lottery-engine.log 2>&1 & echo $$! > .server.pid); \
	 echo "Waiting for server health..."; \
	 for i in {1..60}; do \
		 curl -sf "$(BENCH_URL)/healthz" >/dev/null 2>&1 && break || sleep 1; \
	 done; \
	 echo "Applying migrations..."; \
	 $(CARGO) run --quiet --bin migrate; \
	 echo "Preparing inventory..."; \
	 $(CARGO) run --quiet --bin db_prepare; \
	 echo "Running QPS bench..."; \
	 BENCH_FAST=1 BENCH_URL="$(BENCH_URL)" BENCH_OPS="$(BENCH_OPS)" BENCH_CONC="$(BENCH_CONC)" $(CARGO) run --quiet --bin qps_bench; \
	 echo "Stopping server..."; \
	 kill `cat .server.pid` >/dev/null 2>&1 || true; rm -f .server.pid;
