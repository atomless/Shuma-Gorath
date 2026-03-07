# Scripts

Automation scripts used by the Makefile live here.

## Layout

- `bootstrap/`
  - `setup.sh`: dependency/bootstrap workflow used by `make setup`
  - `verify-setup.sh`: setup verification used by `make verify`
- `deploy/`
  - `local_env.py`: shared helpers for gitignored operator env files such as `.env.local`
  - `gateway_surface_catalog.py`: shared catalog parsing, reserved-route matching, and forward-probe path selection helpers
  - `render_gateway_spin_manifest.py`: renders a deployment-specific Spin manifest with the exact gateway upstream allowlist for shared-host runtime/preflight use
  - `remote_target.py`: normalized `ssh_systemd` remote receipt helpers and generic day-2 remote command dispatch
  - `select_gateway_smoke_path.py`: selects a deterministic non-reserved public path from a surface catalog for post-deploy smoke forwarding checks
  - `spin_manifest.py`: shared manifest normalization/render helpers used by deploy validation and gateway harnesses
  - `validate_gateway_contract.py`: validates gateway env contract + effective Spin manifest outbound-host alignment (invoked by `make deploy-env-validate`)
  - `validate_gateway_route_collisions.py`: validates discovered origin surface catalog against Shuma/Spin reserved routes and writes a deterministic preflight report (invoked by `make deploy-env-validate`)
  - `probe_gateway_origin_bypass.py`: optional active probe that compares gateway vs direct-origin reachability and classifies origin-bypass posture (invoked by `make test-gateway-origin-bypass-probe`)
- `build_site_surface_catalog.py`: compiles a deterministic site-surface catalog from a local docroot so setup flows do not require a human-authored sitemap
- `manage_remote_target.py`: CLI entrypoint for normalized `ssh_systemd` day-2 remote operations (`make remote-*`)
- `prepare_linode_shared_host.py`: agent-oriented shared-host setup entrypoint that captures/persists local Linode handoff state and writes a setup receipt
- `site_surface_catalog.py`: shared local docroot and sitemap inventory helpers used by the generic site-surface catalog builder
- `tests/`
  - `integration.sh`: HTTP integration scenarios used by `make test` and `make test-integration`
  - `gateway_tls_wasm_harness.py`: wasm32 TLS failure matrix harness (expired/self-signed/hostname-mismatch cert paths) used by `make test-gateway-wasm-tls-harness`
- `config_seed.sh`: seeds KV tunables from `config/defaults.env`
- `set_crate_type.sh`: switches crate type between native-test and WASM build modes
- `deploy_linode_one_shot.sh`: provisions a new Linode VM or attaches to a prepared Linode instance and deploys Shuma runtime (invoked by `make deploy-linode-one-shot`)

Use `make help` for the supported entrypoints.
