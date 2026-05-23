import { Link } from "react-router-dom";

export function Register() {
  return (
    <div className="mx-auto max-w-2xl flex flex-col gap-6">
      <h1 className="text-2xl font-semibold">Register a node</h1>

      <div className="card flex flex-col gap-4">
        <p className="text-sm text-zcash-subtle">
          Node registration is done from the terminal using the relay CLI. This
          ensures only operators who are actually running a Zcash node can
          register — no browser wallet needed.
        </p>

        <div className="flex flex-col gap-3">
          <h2 className="text-base font-semibold">Quick start</h2>
          <pre className="overflow-x-auto rounded-md border border-zcash-border bg-zcash-dark p-4 font-mono text-xs leading-6 text-zcash-text">
{`# clone + build the relay (one-time)
git clone https://github.com/ZcashDePIN/DePINZcash
cd DePINZcash/prover
cargo build --release --bin depinzcash-relay
sudo cp ./target/release/depinzcash-relay /usr/local/bin/

# generate a Solana keypair
mkdir -p ~/.depinzcash
depinzcash-relay keygen --out ~/.depinzcash/solana-keypair.json

# register your node
depinzcash-relay register \\
  --api https://api.zcashdepin.com \\
  --keypair ~/.depinzcash/solana-keypair.json \\
  --kind zebra-full \\
  --label my-node

# start submitting proofs (runs every 5 min)
depinzcash-relay watch \\
  --state ~/.depinzcash/relay-state.json \\
  --node-rpc http://127.0.0.1:8232`}
          </pre>
        </div>

        <div className="flex flex-col gap-3">
          <h2 className="text-base font-semibold">Exposed RPC mode (no relay needed)</h2>
          <p className="text-sm text-zcash-subtle">
            Alternatively, expose your Zebra node's JSON-RPC publicly and the server
            polls it every 5 minutes. Add <code className="text-zcash-text">--rpc-endpoint</code> during
            registration:
          </p>
          <pre className="overflow-x-auto rounded-md border border-zcash-border bg-zcash-dark p-4 font-mono text-xs leading-6 text-zcash-text">
{`depinzcash-relay register \\
  --api https://api.zcashdepin.com \\
  --keypair ~/.depinzcash/solana-keypair.json \\
  --kind zebra-full \\
  --label my-node \\
  --rpc-endpoint https://zebra.yourdomain.com`}
          </pre>
          <a
            href="https://github.com/ZcashDePIN/DePINZcash/blob/main/docs/EXPOSED_RPC.md"
            target="_blank"
            rel="noreferrer"
            className="text-sm text-zcash-gold hover:underline"
          >
            Full Exposed RPC setup guide →
          </a>
        </div>

        <div className="flex flex-wrap gap-3 pt-2">
          <Link to="/run-node" className="btn-outline">Zebra setup guide</Link>
          <Link to="/run-lightwalletd" className="btn-outline">Lightwalletd guide</Link>
          <Link to="/dashboard" className="btn-outline">Check your dashboard</Link>
        </div>
      </div>
    </div>
  );
}
