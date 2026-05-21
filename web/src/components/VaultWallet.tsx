import { useEffect, useMemo, useState } from "react";
import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

import { api, type ServerInfo } from "../lib/api";
import { config } from "../lib/config";
import { ErrorBanner, Loading } from "./Loading";
import { formatNumber, shortAddress } from "../lib/format";

// Public treasury display. Always renders so anyone visiting the site can see
// where $ZePIN rewards are paid out from. Reads the wallet pubkey from
// VITE_VAULT_WALLET; until set, shows a transparent "not yet announced" panel.
export function VaultWallet() {
  const vault = config.vaultWallet;
  const [info, setInfo] = useState<ServerInfo | null>(null);

  useEffect(() => {
    api.serverInfo().then(setInfo).catch(() => {});
  }, []);

  if (!vault) {
    return <VaultPlaceholder />;
  }

  // Fall back to the frontend's hardcoded $ZePIN mint if the backend hasn't been
  // redeployed with SPL_MINT yet — keeps balance lookup working immediately.
  const mint = info?.spl_mint ?? config.tokenMint ?? null;
  return <VaultContent vault={vault} mint={mint} />;
}

function VaultPlaceholder() {
  return (
    <section className="card flex flex-col gap-3">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h2 className="text-base font-semibold">{config.vaultLabel}</h2>
        <span className="inline-flex items-center rounded-full border border-zcash-warn/40 bg-zcash-warn/10 px-2 py-0.5 text-[10px] uppercase tracking-wider text-amber-200">
          Address to be announced
        </span>
      </div>
      <p className="text-sm text-zcash-subtle">
        Operator rewards are paid in $ZePIN from a public treasury wallet on Solana.
        The address will be posted here once it's funded so anyone can verify the
        on-chain balance. Refresh after launch.
      </p>
    </section>
  );
}

function VaultContent({ vault, mint }: { vault: string; mint: string | null }) {
  // Treasury wallet + $ZePIN mint always live on Solana mainnet, regardless of
  // what cluster the rest of the app is configured for. Uses a browser-friendly
  // RPC (publicnode by default) since api.mainnet-beta.solana.com 403s browsers.
  const connection = useMemo(
    () => new Connection(config.solanaRpcUrl, "confirmed"),
    [],
  );
  const [sol, setSol] = useState<number | null>(null);
  const [zepin, setZepin] = useState<number | null>(null);
  const [zepinAvailable, setZepinAvailable] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setSol(null);
    setZepin(null);
    setZepinAvailable(false);

    let pubkey: PublicKey;
    try {
      pubkey = new PublicKey(vault);
    } catch (e: unknown) {
      setError(`invalid vault address: ${e instanceof Error ? e.message : String(e)}`);
      setLoading(false);
      return;
    }

    async function load() {
      try {
        const lamports = await connection.getBalance(pubkey);
        if (!cancelled) setSol(lamports / LAMPORTS_PER_SOL);
      } catch (e: unknown) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }

      if (mint) {
        try {
          const mintKey = new PublicKey(mint);
          const accounts = await connection.getParsedTokenAccountsByOwner(pubkey, { mint: mintKey });
          let total = 0;
          for (const a of accounts.value) {
            const info = a.account.data as unknown as {
              parsed?: { info?: { tokenAmount?: { uiAmount?: number } } };
            };
            total += info.parsed?.info?.tokenAmount?.uiAmount ?? 0;
          }
          if (!cancelled) {
            setZepin(total);
            setZepinAvailable(true);
          }
        } catch (e: unknown) {
          // mint exists but the token program rejected — likely wrong network for the mint.
          if (!cancelled) {
            setZepin(0);
            setZepinAvailable(false);
            // don't surface as an error — just leave $ZePIN as "n/a"
          }
        }
      }

      if (!cancelled) setLoading(false);
    }
    load();
    return () => {
      cancelled = true;
    };
  }, [vault, mint, connection]);

  const explorerUrl = explorerLink(vault, config.solanaCluster);

  return (
    <section className="card flex flex-col gap-4">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <div className="flex flex-col gap-1">
          <h2 className="text-base font-semibold">{config.vaultLabel}</h2>
          <p className="text-xs text-zcash-subtle">
            Treasury wallet for $ZePIN distribution + protocol operations. Balances pulled
            live from Solana <code className="text-zcash-text">mainnet</code>.
          </p>
        </div>
        <a
          href={explorerUrl}
          target="_blank"
          rel="noreferrer"
          className="text-xs text-zcash-gold hover:underline"
        >
          View on Solscan ↗
        </a>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-zcash-border bg-zcash-dark px-3 py-2">
        <code className="break-all font-mono text-xs text-zcash-text">{vault}</code>
        <button
          type="button"
          className="text-[10px] uppercase tracking-wider text-zcash-subtle hover:text-zcash-text"
          onClick={() => navigator.clipboard.writeText(vault)}
        >
          copy
        </button>
      </div>

      {error && <ErrorBanner message={error} />}

      <div className="grid grid-cols-2 gap-3">
        <Balance
          label="SOL"
          value={sol}
          loading={loading}
          suffix=""
        />
        <Balance
          label="$ZePIN"
          value={zepin}
          loading={loading}
          suffix=""
          hint={
            mint
              ? zepinAvailable
                ? `mint ${shortAddress(mint, 4, 4)}`
                : "mint lookup failed"
              : "mint not configured"
          }
        />
      </div>
    </section>
  );
}

function Balance({
  label,
  value,
  loading,
  suffix,
  hint,
}: {
  label: string;
  value: number | null;
  loading: boolean;
  suffix: string;
  hint?: string;
}) {
  return (
    <div>
      <div className="stat-label">{label}</div>
      <div className="text-2xl font-semibold text-zcash-text">
        {loading ? (
          <Loading />
        ) : value == null ? (
          <span className="text-zcash-subtle">—</span>
        ) : (
          <>
            {formatNumber(roundTo(value, 4))}
            {suffix && <span className="ml-1 text-sm text-zcash-subtle">{suffix}</span>}
          </>
        )}
      </div>
      {hint && <div className="text-[10px] text-zcash-subtle">{hint}</div>}
    </div>
  );
}

function roundTo(n: number, digits: number): number {
  const f = Math.pow(10, digits);
  return Math.round(n * f) / f;
}

// Always link to mainnet — the treasury lives there regardless of the
// dev-cluster the rest of the app is talking to.
function explorerLink(address: string, _cluster: string): string {
  return `https://solscan.io/account/${address}`;
}
