-- The min_real_height filter made (network, last_height) the most selective
-- combo for every public stats query. Add a composite index that the SQLite
-- planner can use directly instead of falling back to a scan.

CREATE INDEX IF NOT EXISTS idx_nodes_network_last_height
    ON nodes(network, last_height);

-- The leaderboard groups by wallet — give it an index that helps with the
-- GROUP BY when filtering by (network, last_proof_at, last_height).
CREATE INDEX IF NOT EXISTS idx_nodes_wallet_filter
    ON nodes(network, last_proof_at, last_height, wallet);
