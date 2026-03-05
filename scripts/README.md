# Scripts

Automation scripts used by the Makefile live here.

## Layout

- `bootstrap/`
  - `setup.sh`: dependency/bootstrap workflow used by `make setup`
  - `verify-setup.sh`: setup verification used by `make verify`
- `deploy/`
  - `validate_gateway_contract.py`: validates gateway env contract + `spin.toml` outbound-host alignment (invoked by `make deploy-env-validate`)
  - `validate_gateway_route_collisions.py`: validates discovered origin surface catalog against Shuma/Spin reserved routes and writes a deterministic preflight report (invoked by `make deploy-env-validate`)
  - `probe_gateway_origin_bypass.py`: optional active probe that compares gateway vs direct-origin reachability and classifies origin-bypass posture (invoked by `make test-gateway-origin-bypass-probe`)
- `tests/`
  - `integration.sh`: HTTP integration scenarios used by `make test` and `make test-integration`
  - `gateway_tls_wasm_harness.py`: wasm32 TLS failure matrix harness (expired/self-signed/hostname-mismatch cert paths) used by `make test-gateway-wasm-tls-harness`
- `config_seed.sh`: seeds KV tunables from `config/defaults.env`
- `set_crate_type.sh`: switches crate type between native-test and WASM build modes
- `deploy_linode_one_shot.sh`: provisions a Linode VM and deploys Shuma runtime (invoked by `make deploy-linode-one-shot`)

Use `make help` for the supported entrypoints.
