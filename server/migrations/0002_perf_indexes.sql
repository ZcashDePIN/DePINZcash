-- Indexes for the public /api/stats/network and /api/stats/leaderboard paths.
-- Both filter on (network, last_proof_at IS NOT NULL); without these, every
-- aggregate is a full table scan over the nodes table.

CREATE INDEX IF NOT EXISTS idx_nodes_network_last_proof
    ON nodes(network, last_proof_at);

CREATE INDEX IF NOT EXISTS idx_nodes_network_status_last_proof
    ON nodes(network, status, last_proof_at);

-- The total_proofs / accepted_proofs counters join proofs onto nodes and
-- filter by (network, last_proof_at IS NOT NULL, verdict). Index proofs by
-- verdict so the join doesn't have to scan every row.
CREATE INDEX IF NOT EXISTS idx_proofs_verdict ON proofs(verdict);
