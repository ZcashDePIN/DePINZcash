import { useEffect, useState } from "react";
import { Link } from "react-router-dom";

import {
  api,
  type ProofRecord,
  type PublicNode,
  zcashExplorerUrl,
} from "../lib/api";
import { ErrorBanner, Loading } from "../components/Loading";
import { formatNumber, formatRelative, formatUptime, shortAddress } from "../lib/format";

type VerdictFilter = "all" | "accepted" | "rejected" | "pending";

export function Explorer() {
  const [nodes, setNodes] = useState<PublicNode[] | null>(null);
  const [proofs, setProofs] = useState<ProofRecord[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [verdict, setVerdict] = useState<VerdictFilter>("all");
  const [walletInput, setWalletInput] = useState("");
  const [walletFilter, setWalletFilter] = useState("");

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [n, p] = await Promise.all([
          api.activeNodes(200),
          api.recentProofs({ limit: 100, verdict, wallet: walletFilter || undefined }),
        ]);
        if (cancelled) return;
        setNodes(n);
        setProofs(p);
        setError(null);
      } catch (e: unknown) {
        if (cancelled) return;
        setError(e instanceof Error ? e.message : String(e));
      }
    }
    load();
    const id = setInterval(load, 30_000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [verdict, walletFilter]);

  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-col gap-2">
        <h1 className="text-2xl font-semibold">Explorer</h1>
        <p className="text-sm text-zcash-subtle">
          Every registered node and every proof they submit. Each block hash links to
          Blockchair so you can verify it's a real Zcash mainnet block. Auto-refreshes
          every 30s.
        </p>
      </header>

      {error && <ErrorBanner message={error} />}

      <section className="flex flex-col gap-3">
        <div className="flex items-baseline justify-between">
          <h2 className="text-lg font-semibold">Active nodes</h2>
          <span className="text-xs text-zcash-subtle">
            {nodes ? `${formatNumber(nodes.length)} with accepted proofs` : "…"}
          </span>
        </div>
        {!nodes && !error && <Loading />}
        {nodes && nodes.length === 0 && (
          <div className="card text-sm text-zcash-subtle">
            No nodes have submitted accepted proofs yet.
          </div>
        )}
        {nodes && nodes.length > 0 && (
          <div className="card overflow-x-auto p-0">
            <table className="w-full text-sm">
              <thead className="border-b border-zcash-border text-left text-xs uppercase tracking-wider text-zcash-subtle">
                <tr>
                  <th className="px-4 py-3">Label</th>
                  <th className="px-4 py-3">Kind</th>
                  <th className="px-4 py-3">Wallet</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="px-4 py-3">Last height</th>
                  <th className="px-4 py-3">Last block</th>
                  <th className="px-4 py-3">Last seen</th>
                  <th className="px-4 py-3 text-right">Points</th>
                </tr>
              </thead>
              <tbody>
                {nodes.map((n) => (
                  <tr key={n.id} className="border-b border-zcash-border/60 last:border-b-0">
                    <td className="px-4 py-2">
                      <Link to={`/node/${encodeURIComponent(n.id)}`} className="hover:text-zcash-gold">
                        {n.label || <span className="text-zcash-subtle">unlabeled</span>}
                      </Link>
                    </td>
                    <td className="px-4 py-2">
                      <span className="pill">{n.kind}</span>
                    </td>
                    <td className="px-4 py-2">
                      <Link
                        to={`/dashboard/${encodeURIComponent(n.wallet)}`}
                        className="font-mono text-xs text-zcash-subtle hover:text-zcash-text"
                      >
                        {shortAddress(n.wallet, 4, 4)}
                      </Link>
                    </td>
                    <td className="px-4 py-2">
                      <StatusBadge status={n.status} />
                    </td>
                    <td className="px-4 py-2">{n.last_height != null ? formatNumber(n.last_height) : "—"}</td>
                    <td className="px-4 py-2 font-mono text-xs">
                      {n.last_block_hash ? (
                        <a
                          href={zcashExplorerUrl(n.last_block_hash)}
                          target="_blank"
                          rel="noreferrer"
                          className="text-zcash-gold hover:underline"
                          title="verify on Blockchair"
                        >
                          {shortAddress(n.last_block_hash, 8, 6)}
                        </a>
                      ) : (
                        "—"
                      )}
                    </td>
                    <td className="px-4 py-2 whitespace-nowrap">{formatRelative(n.last_proof_at)}</td>
                    <td className="px-4 py-2 text-right font-semibold">{formatNumber(n.points)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="flex flex-col gap-3">
        <div className="flex flex-wrap items-baseline justify-between gap-3">
          <h2 className="text-lg font-semibold">Recent proofs</h2>
          <span className="text-xs text-zcash-subtle">
            {proofs ? `${formatNumber(proofs.length)} latest` : "…"} · click block hash to verify on-chain
          </span>
        </div>

        <div className="card flex flex-wrap items-end gap-3">
          <div className="flex flex-col gap-1">
            <label className="stat-label" htmlFor="filter-verdict">Verdict</label>
            <select
              id="filter-verdict"
              className="input"
              value={verdict}
              onChange={(e) => setVerdict(e.target.value as VerdictFilter)}
            >
              <option value="all">all</option>
              <option value="accepted">accepted</option>
              <option value="rejected">rejected</option>
              <option value="pending">pending</option>
            </select>
          </div>
          <form
            className="flex flex-1 flex-col gap-1"
            onSubmit={(e) => {
              e.preventDefault();
              setWalletFilter(walletInput.trim());
            }}
          >
            <label className="stat-label" htmlFor="filter-wallet">Wallet</label>
            <div className="flex gap-2">
              <input
                id="filter-wallet"
                className="input flex-1"
                value={walletInput}
                onChange={(e) => setWalletInput(e.target.value)}
                placeholder="paste a Solana wallet to filter…"
              />
              <button type="submit" className="btn-outline">apply</button>
              {walletFilter && (
                <button
                  type="button"
                  className="btn-outline"
                  onClick={() => {
                    setWalletInput("");
                    setWalletFilter("");
                  }}
                >
                  clear
                </button>
              )}
            </div>
            {walletFilter && (
              <p className="mt-1 text-[10px] text-zcash-subtle">
                showing proofs for <code className="font-mono">{walletFilter}</code>
              </p>
            )}
          </form>
        </div>
        {!proofs && !error && <Loading />}
        {proofs && proofs.length === 0 && (
          <div className="card text-sm text-zcash-subtle">No proofs submitted yet.</div>
        )}
        {proofs && proofs.length > 0 && (
          <div className="card overflow-x-auto p-0">
            <table className="w-full text-sm">
              <thead className="border-b border-zcash-border text-left text-xs uppercase tracking-wider text-zcash-subtle">
                <tr>
                  <th className="px-4 py-3">When</th>
                  <th className="px-4 py-3">Wallet</th>
                  <th className="px-4 py-3">Height</th>
                  <th className="px-4 py-3">Block hash</th>
                  <th className="px-4 py-3">Verdict</th>
                  <th className="px-4 py-3">Peers</th>
                  <th className="px-4 py-3">Uptime</th>
                  <th className="px-4 py-3 text-right">Points</th>
                </tr>
              </thead>
              <tbody>
                {proofs.map((p) => (
                  <tr key={p.id} className="border-b border-zcash-border/60 last:border-b-0">
                    <td className="px-4 py-2 whitespace-nowrap">{formatRelative(p.received_at)}</td>
                    <td className="px-4 py-2">
                      <Link
                        to={`/dashboard/${encodeURIComponent(p.wallet)}`}
                        className="font-mono text-xs text-zcash-subtle hover:text-zcash-text"
                      >
                        {shortAddress(p.wallet, 4, 4)}
                      </Link>
                    </td>
                    <td className="px-4 py-2">{formatNumber(p.claimed_height)}</td>
                    <td className="px-4 py-2 font-mono text-xs">
                      <a
                        href={zcashExplorerUrl(p.claimed_block_hash)}
                        target="_blank"
                        rel="noreferrer"
                        className="text-zcash-gold hover:underline"
                        title="verify on Blockchair"
                      >
                        {shortAddress(p.claimed_block_hash, 8, 6)} ↗
                      </a>
                    </td>
                    <td className="px-4 py-2">
                      <VerdictBadge verdict={p.verdict} reason={p.reject_reason} />
                    </td>
                    <td className="px-4 py-2">{p.peers != null ? formatNumber(p.peers) : "—"}</td>
                    <td className="px-4 py-2 whitespace-nowrap">
                      {p.uptime_seconds != null ? formatUptime(p.uptime_seconds) : "—"}
                    </td>
                    <td className="px-4 py-2 text-right font-semibold">{formatNumber(p.points_awarded)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const colour =
    status === "active"
      ? "border-zcash-success/40 bg-zcash-success/10 text-emerald-300"
      : status === "stale"
        ? "border-zcash-warn/40 bg-zcash-warn/10 text-amber-200"
        : status === "suspended"
          ? "border-zcash-danger/40 bg-zcash-danger/10 text-red-200"
          : "border-zcash-border bg-zcash-surface text-zcash-subtle";
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wider ${colour}`}>
      {status}
    </span>
  );
}

function VerdictBadge({ verdict, reason }: { verdict: string; reason: string | null }) {
  const colour =
    verdict === "accepted"
      ? "border-zcash-success/40 bg-zcash-success/10 text-emerald-300"
      : verdict === "rejected"
        ? "border-zcash-danger/40 bg-zcash-danger/10 text-red-200"
        : "border-zcash-warn/40 bg-zcash-warn/10 text-amber-200";
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wider ${colour}`}
      title={reason ?? undefined}
    >
      {verdict}
    </span>
  );
}
