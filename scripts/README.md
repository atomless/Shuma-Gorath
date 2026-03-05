# Scripts

Automation scripts used by the Makefile live here.

## Layout

- `bootstrap/`
  - `setup.sh`: dependency/bootstrap workflow used by `make setup`
  - `verify-setup.sh`: setup verification used by `make verify`
- `deploy/`
  - `validate_gateway_contract.py`: validates gateway env contract + `spin.toml` outbound-host alignment (invoked by `make deploy-env-validate`)
- `tests/`
  - `integration.sh`: HTTP integration scenarios used by `make test` and `make test-integration`
- `config_seed.sh`: seeds KV tunables from `config/defaults.env`
- `set_crate_type.sh`: switches crate type between native-test and WASM build modes
- `deploy_linode_one_shot.sh`: provisions a Linode VM and deploys Shuma runtime (invoked by `make deploy-linode-one-shot`)

Use `make help` for the supported entrypoints.
