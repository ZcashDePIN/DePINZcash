# Exposed RPC mode — operator setup guide

Run a Zcash node, expose its JSON-RPC on a public URL, and let DePINZcash poll
it. No relay binary, no CLI on your machine. The server cross-checks your tip
against a trusted RPC quorum every few minutes and credits you in $ZePIN when
the answers match.

This guide walks through a Zebra setup; the same pattern works for `zcashd`.

## What the server actually polls

Every `EXPOSED_RPC_POLL_INTERVAL` (default 5 min on Fly) the server calls your
URL with two standard JSON-RPC methods:

| method | params | server uses it for |
|---|---|---|
| `getblockcount` | `[]` | learns your current tip height |
| `getblockhash` | `[<height>]` | gets your hash at that height |

If your tip is within `max_height_drift` of the trusted quorum (default 8
blocks) AND your `getblockhash` answer matches the quorum's answer at the same
height, you get an accepted proof. Mismatch → rejected, no points, no penalty
beyond that.

That's it. Two calls, no auth, no special endpoint paths.

## Prerequisites

- A Linux box with at least 4 GB RAM and ~100 GB free disk (Zcash chain is ~80 GB).
- Public IPv4 (or IPv6) reachable from the internet on the port you'll expose.
- A domain or subdomain you can point at the box (recommended for TLS).

If you only have a residential connection, port-forward 8232 (Zebra default
JSON-RPC port) on your router, or use a tunnel like `cloudflared` or Tailscale
Funnel. Localhost-only / private-network URLs are explicitly **rejected at
registration** — they're unreachable from Fly.

## 1. Install and run Zebra

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y build-essential git protobuf-compiler clang
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone https://github.com/ZcashFoundation/zebra
cd zebra
cargo install --locked --path zebrad
```

Create `~/.config/zebrad.toml`:

```toml
[network]
network = "Mainnet"

[rpc]
# Bind to all interfaces; we'll restrict access at the reverse-proxy layer.
listen_addr = "0.0.0.0:8232"
parallel_cpu_threads = 4

[state]
cache_dir = "/var/lib/zebra"
```

Then:

```bash
sudo mkdir -p /var/lib/zebra
sudo chown $USER /var/lib/zebra
zebrad start
```

Initial sync takes 12–48 hours depending on your hardware. The relay can
register before sync finishes — proofs from a syncing node just get rejected
until the height catches up to the trusted tip.

## 2. Front it with TLS (Caddy, 30 seconds)

The DePINZcash server can talk to plain HTTP, but you don't want random
internet traffic hitting `:8232` directly. Caddy gives you HTTPS with one
config line and handles cert renewal automatically.

```bash
# Install Caddy (Ubuntu)
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install -y caddy
```

Edit `/etc/caddy/Caddyfile`:

```
zebra.yourdomain.com {
    reverse_proxy 127.0.0.1:8232

    # Only allow Zcash's two read-only RPC methods. Anything else returns 403.
    @rpc {
        method POST
        header Content-Type application/json
    }
    handle @rpc {
        reverse_proxy 127.0.0.1:8232
    }
    handle {
        respond "method not allowed" 405
    }
}
```

```bash
sudo systemctl reload caddy
```

Point your DNS A record `zebra.yourdomain.com` → your box's public IP. Caddy
provisions a Let's Encrypt cert on first request.

## 3. Verify it from the outside

From any other machine (your laptop is fine):

```bash
curl -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' \
    https://zebra.yourdomain.com
```

Expected output (your height will vary):

```json
{"jsonrpc":"2.0","id":1,"result":3351000}
```

If this returns a real number, the DePINZcash server can too.

## 4. Register your node

```bash
# Option A — register from your terminal with the relay binary
git clone https://github.com/ZcashDePIN/DePINZcash
cd DePINZcash/prover
cargo build --release --bin depinzcash-relay
sudo cp target/release/depinzcash-relay /usr/local/bin/

depinzcash-relay keygen --out ~/.depinzcash/solana-keypair.json
depinzcash-relay register \
    --api https://api.zcashdepin.com \
    --keypair ~/.depinzcash/solana-keypair.json \
    --kind zebra-full \
    --label primary \
    --rpc-endpoint https://zebra.yourdomain.com
```

Or use the web UI at <https://zcashdepin.com/register> — connect Phantom,
fill in the **Public RPC URL** field with `https://zebra.yourdomain.com`,
sign.

## 5. Confirm the server is polling you

Within ~5 minutes of registration, the server should have written its first
exposed-rpc proof. Check it:

```bash
curl https://api.zcashdepin.com/api/wallet/<your-wallet>/proofs?limit=5 | jq
```

Look for entries with `binary_hash: "exposed-rpc-poll"`:

```json
{
  "claimed_height": 3351023,
  "claimed_block_hash": "0000000000…",
  "verdict": "accepted",
  "binary_hash": "exposed-rpc-poll",
  "points_awarded": 60
}
```

Or visit <https://zcashdepin.com/explorer> — your node shows up in the active
list with its last height and a clickable Blockchair link to verify the hash
on-chain.

## Troubleshooting

**Proofs show `verdict: rejected` with `exposed-rpc: hash mismatch`** — your
Zebra is on the wrong network (Testnet vs Mainnet) or still syncing far behind
the trusted tip. Let it finish syncing; the next poll will accept.

**Proofs say `pending` with `trusted-quorum-failed-to-agree`** — the server's
own trusted RPC endpoints are flaky right now. Not on you. Earnings resume
when the server's quorum comes back.

**No proofs at all** — the server can't reach you. Check:

1. `curl -X POST https://zebra.yourdomain.com -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'` from a third machine.
2. DNS is propagated: `dig zebra.yourdomain.com`.
3. Firewall allows 443 inbound: `sudo ufw status`.
4. Caddy logs: `sudo journalctl -u caddy -f`.

**Worried about DoS / scraping** — only the two methods above are needed.
Tighten Caddy's `@rpc` matcher to require an exact body shape, or add a basic
auth gate (Zebra supports `rpc.cookie_dir`, or wrap with Caddy's
`basic_auth`). DePINZcash can include credentials in the URL
(`https://user:pass@host`) if you want auth.

## Security notes

- Exposing JSON-RPC means anyone can call read-only methods on your node. Zebra
  exposes no wallet/signing methods, so the blast radius is limited to "people
  can read the public Zcash chain from your bandwidth." Still — restrict the
  endpoint set with Caddy as shown.
- Never expose `zcashd`'s RPC the same way without auth: it has wallet
  methods. Use `rpcuser`/`rpcpassword` and Caddy basic_auth, then put the
  credentials in the URL when you register.
- Rotate your domain or move the node any time. Just re-register with the new
  URL (same wallet, unique label).

## Reference

- Trusted quorum methods: [`server/src/rpc.rs`](../server/src/rpc.rs)
- Poll loop: [`server/src/scheduler.rs`](../server/src/scheduler.rs) (`exposed_rpc_loop`)
- Tests covering accept / reject / dedupe / drift / no-endpoint: [`server/tests/exposed_rpc.rs`](../server/tests/exposed_rpc.rs)
