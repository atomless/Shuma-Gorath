.PHONY: local prod clean test test-unit test-integration test-dashboard

local:
	pkill -f spin || true
	sleep 1
	cargo clean
	cargo build --target wasm32-wasip1 --release
	spin up

prod:
	pkill -f spin || true
	sleep 1
	cargo clean
	cargo build --target wasm32-wasip1 --release
	spin up --release

clean:
	cargo clean

# Test targets
test: test-unit test-integration
	@echo "✅ All backend tests complete!"
	@echo "💡 To test dashboard: make test-dashboard"

test-unit:
	@echo "🧪 Running Rust unit tests..."
	@cargo test

test-integration:
	@echo "🧪 Running Spin integration tests..."
	@./test_spin_colored.sh

test-dashboard:
	@echo "🧪 Dashboard testing (manual):"
	@echo "1. Ensure Spin is running: make local"
	@echo "2. Open: http://127.0.0.1:3000/dashboard/index.html"
	@echo "3. Follow checklist in TESTING.md"
