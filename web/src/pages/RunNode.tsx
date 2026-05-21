import { useState } from "react";
import { Link } from "react-router-dom";

type InstallMethod = "binary" | "docker" | "source";

export function RunNode() {
  const [method, setMethod] = useState<InstallMethod>("binary");

  return (
    <div className="flex flex-col gap-10">
      <section className="grid gap-6 md:grid-cols-2 md:items-center">
        <div className="flex flex-col gap-4">
          <span className="pill w-fit">Operator guide · ~30 min setup</span>
          <h1 className="text-3xl font-semibold leading-tight md:text-4xl">
            Run a <span className="text-zcash-gold">Zebra</span> full node.
          </h1>
          <p className="max-w-prose text-sm text-zcash-subtle">
            Zebra is the official Zcash full node client built by the Zcash Foundation in Rust.
            Running one keeps the network decentralised, validates every shielded transaction,
            and earns you DePINZcash rewards while it's online.
          </p>
          <div className="flex flex-wrap gap-2">
            <a
              href="https://zfnd.org/zebra/"
              target="_blank"
              rel="noreferrer"
              className="btn-outline"
            >
              Official Zebra docs ↗
            </a>
            <Link to="/register" className="btn-primary">After install → Register node</Link>
          </div>
        </div>
        <NodeIllustration />
      </section>

      <section className="grid gap-4 md:grid-cols-2">
        <article className="card flex flex-col gap-3">
          <h2 className="text-lg font-semibold">What is a Zcash full node?</h2>
          <p className="text-sm text-zcash-subtle">
            A full node downloads every block ever mined on Zcash, verifies it against the
            consensus rules, and serves that data to wallets and other peers. Unlike a light
            client, it does not trust a third party — it sees the chain directly.
          </p>
          <p className="text-sm text-zcash-subtle">
            Zebra is one of two implementations of the Zcash protocol (the other being
            <code className="mx-1 text-zcash-text">zcashd</code>). It is written in Rust,
            independently audited, and the future of Zcash node software.
          </p>
        </article>

        <article className="card flex flex-col gap-3">
          <h2 className="text-lg font-semibold">Why running one matters</h2>
          <ul className="flex flex-col gap-2 text-sm text-zcash-subtle">
            <li>
              <span className="text-zcash-text font-medium">Decentralisation.</span>{" "}
              Every additional independent node makes Zcash harder to censor or coerce.
            </li>
            <li>
              <span className="text-zcash-text font-medium">Privacy verification.</span>{" "}
              Shielded transactions are validated by your machine — strengthening the
              guarantees the protocol provides to everyone.
            </li>
            <li>
              <span className="text-zcash-text font-medium">Network capacity.</span>{" "}
              Lightwalletd and wallet RPCs depend on full nodes serving data quickly.
            </li>
            <li>
              <span className="text-zcash-text font-medium">DePIN rewards.</span>{" "}
              You get paid in $ZePIN on Solana for keeping a verified Zebra node synced.
            </li>
          </ul>
        </article>
      </section>

      <section className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold">System requirements</h2>
        <div className="card overflow-x-auto p-0">
          <table className="w-full text-sm">
            <thead className="border-b border-zcash-border text-left text-xs uppercase tracking-wider text-zcash-subtle">
              <tr>
                <th className="px-4 py-3">Resource</th>
                <th className="px-4 py-3">Minimum</th>
                <th className="px-4 py-3">Recommended</th>
              </tr>
            </thead>
            <tbody className="text-sm">
              <ReqRow what="CPU" min="2 cores, 64-bit x86_64 / arm64" rec="4+ cores" />
              <ReqRow what="RAM" min="4 GB" rec="8 GB" />
              <ReqRow what="Disk" min="120 GB SSD (mainnet, growing)" rec="200 GB NVMe" />
              <ReqRow what="Network" min="10 Mbps, ~50 GB/mo egress" rec="100 Mbps, unmetered" />
              <ReqRow what="OS" min="Linux / macOS / Windows" rec="Recent Linux (Ubuntu 22.04+)" />
              <ReqRow what="Uptime" min="A few hours per day" rec="24/7 — better rewards" />
            </tbody>
          </table>
        </div>
      </section>

      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold">Install Zebra</h2>
        <div className="flex flex-wrap gap-2">
          <Tab active={method === "binary"} onClick={() => setMethod("binary")} label="Pre-built binary" />
          <Tab active={method === "docker"} onClick={() => setMethod("docker")} label="Docker (recommended)" />
          <Tab active={method === "source"} onClick={() => setMethod("source")} label="Build from source" />
        </div>

        {method === "binary" && (
          <Card>
            <p className="text-sm text-zcash-subtle">
              The Zcash Foundation publishes signed release binaries for Linux and macOS on
              GitHub. Pick the latest stable release.
            </p>
            <Code lang="bash">{`# Linux x86_64 — replace v3.x.x with the current release tag from
# https://github.com/ZcashFoundation/zebra/releases
ZEBRA_VERSION=v3.0.0
curl -L -o zebrad.tar.gz \\
  https://github.com/ZcashFoundation/zebra/releases/download/$ZEBRA_VERSION/zebrad-$ZEBRA_VERSION-x86_64-unknown-linux-gnu.tar.gz
tar -xzf zebrad.tar.gz
sudo mv zebrad /usr/local/bin/
zebrad --version`}</Code>
            <p className="text-xs text-zcash-subtle">
              On macOS the asset is named <code className="text-zcash-text">…-apple-darwin.tar.gz</code>.
              Windows users can grab the <code className="text-zcash-text">.msi</code> from the same
              release page or use the Docker route below.
            </p>
          </Card>
        )}

        {method === "docker" && (
          <Card>
            <p className="text-sm text-zcash-subtle">
              Easiest option if you already have Docker. Persists chain data in a named volume
              and restarts automatically.
            </p>
            <Code lang="bash">{`docker run -d \\
  --name zebrad \\
  --restart unless-stopped \\
  -p 8233:8233 \\
  -p 8232:8232 \\
  -v zebra-state:/home/zebra/.cache/zebra \\
  zfnd/zebra:latest`}</Code>
            <p className="text-xs text-zcash-subtle">
              Port <code className="text-zcash-text">8233</code> is the peer-to-peer port — open it
              in your firewall for inbound peers (optional but boosts your rewards tier).
              Port <code className="text-zcash-text">8232</code> is the JSON-RPC for the relay CLI.
            </p>
          </Card>
        )}

        {method === "source" && (
          <Card>
            <p className="text-sm text-zcash-subtle">
              Build a fresh Zebra from the official source. Takes 10–20 minutes on a modern laptop.
            </p>
            <Code lang="bash">{`# Toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Build deps
sudo apt-get update && sudo apt-get install -y build-essential clang libclang-dev pkg-config

# Build Zebra itself
git clone --depth 1 --branch v3.0.0 https://github.com/ZcashFoundation/zebra.git
cd zebra
cargo build --release --bin zebrad
sudo install ./target/release/zebrad /usr/local/bin/
zebrad --version`}</Code>
          </Card>
        )}
      </section>

      <section className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold">Run your first sync</h2>
        <Card>
          <ol className="flex list-decimal flex-col gap-3 pl-5 text-sm text-zcash-subtle marker:text-zcash-gold">
            <li>
              <span className="text-zcash-text font-medium">Generate a config.</span>{" "}
              <Code lang="bash">{`zebrad generate -o ~/.config/zebrad.toml`}</Code>
            </li>
            <li>
              <span className="text-zcash-text font-medium">Enable the JSON-RPC</span>{" "}
              so the DePINZcash relay can read your node state. Open the config and add:
              <Code lang="toml">{`[rpc]
listen_addr = "127.0.0.1:8232"`}</Code>
            </li>
            <li>
              <span className="text-zcash-text font-medium">Start syncing.</span>{" "}
              First sync downloads ~60 GB and takes 4–24 hours depending on hardware and
              connection. Leave it running.
              <Code lang="bash">{`zebrad start`}</Code>
            </li>
            <li>
              <span className="text-zcash-text font-medium">Watch progress</span>{" "}
              in the logs — look for <code className="text-zcash-text">100% synced</code>
              {" "}or query the tip:
              <Code lang="bash">{`curl -s -H 'Content-Type: application/json' \\
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' \\
  http://127.0.0.1:8232 | jq`}</Code>
            </li>
          </ol>
        </Card>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        <NextStep
          step="1"
          title="Register your node"
          body="Connect Phantom and sign the registration message. You'll get a node ID + auth token."
          to="/register"
          cta="Open register →"
        />
        <NextStep
          step="2"
          title="Download depinzcash-relay"
          body="Tiny binary that signs and submits your node's state on a 5-minute loop."
          href="https://github.com/ZcashDePIN/DePINZcash/releases"
          cta="Releases ↗"
        />
        <NextStep
          step="3"
          title="Watch your points"
          body="Connect your wallet on the Dashboard to see uptime, points, and your next snapshot claim."
          to="/dashboard"
          cta="Open dashboard →"
        />
      </section>
    </div>
  );
}

function Tab({ active, onClick, label }: { active: boolean; onClick: () => void; label: string }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-md border px-3 py-1.5 text-sm transition-colors ${
        active
          ? "border-zcash-gold bg-zcash-gold/10 text-zcash-gold"
          : "border-zcash-border bg-transparent text-zcash-subtle hover:text-zcash-text"
      }`}
    >
      {label}
    </button>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return <div className="card flex flex-col gap-3">{children}</div>;
}

function Code({ children, lang }: { children: string; lang?: string }) {
  return (
    <pre className="overflow-x-auto rounded-md border border-zcash-border bg-zcash-dark p-3 font-mono text-xs leading-5 text-zcash-text">
      {lang && <div className="mb-1 text-[10px] uppercase tracking-wider text-zcash-subtle">{lang}</div>}
      <code>{children}</code>
    </pre>
  );
}

function ReqRow({ what, min, rec }: { what: string; min: string; rec: string }) {
  return (
    <tr className="border-b border-zcash-border/60 last:border-b-0">
      <td className="px-4 py-2 font-medium text-zcash-text">{what}</td>
      <td className="px-4 py-2 text-zcash-subtle">{min}</td>
      <td className="px-4 py-2">{rec}</td>
    </tr>
  );
}

function NextStep({
  step,
  title,
  body,
  to,
  href,
  cta,
}: {
  step: string;
  title: string;
  body: string;
  to?: string;
  href?: string;
  cta: string;
}) {
  return (
    <div className="card flex flex-col gap-2">
      <span className="text-xs text-zcash-gold">Step {step}</span>
      <h3 className="text-base font-semibold">{title}</h3>
      <p className="text-sm text-zcash-subtle">{body}</p>
      <div className="mt-auto pt-2">
        {to ? (
          <Link to={to} className="text-sm text-zcash-gold hover:underline">{cta}</Link>
        ) : (
          <a href={href} target="_blank" rel="noreferrer" className="text-sm text-zcash-gold hover:underline">
            {cta}
          </a>
        )}
      </div>
    </div>
  );
}

function NodeIllustration() {
  return (
    <div className="card flex h-full flex-col gap-3">
      <div className="rounded-md border border-zcash-border bg-zcash-dark p-4">
        <div className="grid grid-cols-2 gap-3 text-xs text-zcash-subtle">
          <Pill label="Zebra full node" sub="zebrad" />
          <Pill label="JSON-RPC" sub=":8232" />
          <Pill label="P2P" sub=":8233" />
          <Pill label="Verified blocks" sub="every shielded tx" />
        </div>
      </div>
      <div className="rounded-md border border-zcash-border bg-zcash-dark p-4 text-xs text-zcash-subtle">
        <div className="mb-2 text-zcash-text">depinzcash-relay submits every 5 minutes</div>
        <div className="font-mono leading-5">
          <span className="text-zcash-gold">→</span> POST /api/proofs/submit
          <br />
          <span className="text-zcash-gold">←</span> verdict: accepted · points: +75
        </div>
      </div>
    </div>
  );
}

function Pill({ label, sub }: { label: string; sub: string }) {
  return (
    <div className="rounded-md border border-zcash-border bg-zcash-surface px-3 py-2">
      <div className="text-zcash-text">{label}</div>
      <div className="text-[10px] text-zcash-subtle">{sub}</div>
    </div>
  );
}
