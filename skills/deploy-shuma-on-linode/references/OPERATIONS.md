# Linode Deploy Operations

## Preflight Checklist

Run this before provisioning:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--profile medium --region gb-lon --preflight-only"
```

The preflight verifies:

- Linode token can query API.
- Region slug exists.
- Instance type exists.
- Image lookup (best-effort first-page validation).

## Common Issues

### Linode API auth fails

Symptoms:

- script exits with HTTP `401` or `403` during preflight/create.

Checks:

```bash
echo "$LINODE_TOKEN" | wc -c
```

- confirm token has Linodes read/write scope.
- confirm token has not been revoked.

### Region or type validation fails

Symptoms:

- preflight fails before any instance is created.

Fix:

- choose a valid region/type for your account.
- rerun with `--region` and/or `--type`.

### SSH never becomes ready

Symptoms:

- instance created but script times out waiting for SSH.

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip>
```

- confirm local private key matches uploaded public key.
- verify Linode networking/firewall allows SSH.

### Service fails after deploy

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip> 'sudo systemctl status shuma-gorath --no-pager'
ssh -i <private-key> shuma@<instance-ip> 'sudo journalctl -u shuma-gorath -n 200 --no-pager'
```

Potential causes:

- insufficient instance resources for build/start.
- `.env.local` values need adjustment for your environment.

### TLS/Caddy not serving

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip> 'sudo systemctl status caddy --no-pager'
```

- verify DNS A/AAAA points to the Linode public IP.
- restart Caddy after DNS propagation.

## Cleanup

Use one of these cleanup paths:

1. During failures, run with `--destroy-on-failure` so failed creates are auto-removed.
2. Manual remove:

```bash
curl -X DELETE \
  -H "Authorization: Bearer <LINODE_TOKEN>" \
  "https://api.linode.com/v4/linode/instances/<INSTANCE_ID>"
```
