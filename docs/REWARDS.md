# DePINZcash Rewards System

**How you earn for running Zcash nodes**

## Overview

DePINZcash rewards node operators in two ways:
1. **One-time sync bonus** - for completing blockchain synchronization
2. **Ongoing uptime rewards** - for keeping your node running

All rewards are paid in **ZEC** (Zcash) or **SOL** (Solana), your choice.

## Reward Structure

### 1. Initial Sync Bonus

Get rewarded for syncing your Zebra node with the Zcash blockchain.

| Sync Completion | Reward (ZEC) |
|----------------|--------------|
| 100%           | 0.5 ZEC      |
| 90-99%         | 0.375 ZEC    |
| 75-89%         | 0.25 ZEC     |
| Below 75%      | 0 ZEC        |

**Note:** This is a one-time reward. You only get it once per node.

### 2. Uptime Rewards

Earn continuously for keeping your node online and synced.

**Base Rate:** 0.001 ZEC per hour

**Service Multiplier:**
- **1.5x** if your node is serving peers (actively helping the network)
- **1.0x** if your node is not serving peers

#### Calculation

```
uptime_reward = hours_online × 0.001 × multiplier
```

**Example:**
- 30 days online (720 hours)
- Serving 10+ peers
- Reward: 720 × 0.001 × 1.5 = **1.08 ZEC**

## Real Examples

### Scenario 1: New Node, Full Sync

**Setup:**
- Just synced Zebra to 100%
- Running for 7 days (168 hours)
- Not yet serving peers

**Rewards:**
- Sync bonus: 0.5 ZEC
- Uptime: 168 × 0.001 × 1.0 = 0.168 ZEC
- **Total: 0.668 ZEC**

### Scenario 2: Established Node, 30 Days

**Setup:**
- Already claimed sync bonus
- Running for 30 days (720 hours)
- Serving 23 peers actively

**Rewards:**
- Sync bonus: 0 (already claimed)
- Uptime: 720 × 0.001 × 1.5 = 1.08 ZEC
- **Total: 1.08 ZEC**

### Scenario 3: Part-Time Node, 14 Days

**Setup:**
- 95% synced (still catching up)
- Running for 14 days (336 hours)
- Serving 5 peers

**Rewards:**
- Sync bonus: 0.375 ZEC (90-99% tier)
- Uptime: 336 × 0.001 × 1.5 = 0.504 ZEC
- **Total: 0.879 ZEC**

## Proof Submission Frequency

You can submit proofs to claim rewards:

| Frequency | Recommended For | Notes |
|-----------|----------------|-------|
| Daily | Testing phase | Track your progress |
| Weekly | Active operators | Good balance |
| Monthly | Long-term runners | Less frequent claiming |

**Rate Limit:** Maximum 1 proof per 24 hours per wallet address.

## Payout Schedule

### Phase 1 (MVP - Current)
- Manual review and verification
- **Weekly batch payments**
- Distributed every Monday
- Minimum payout: 0.1 ZEC

### Phase 2 (Future)
- Automated smart contract
- Instant payouts after verification
- No minimum threshold

## Estimated Monthly Earnings

Assuming 24/7 uptime and serving peers (1.5x multiplier):

| Days Online | Hours | Base Reward | With Multiplier | + Sync Bonus |
|-------------|-------|-------------|-----------------|--------------|
| 7 days      | 168   | 0.168 ZEC   | 0.252 ZEC       | 0.752 ZEC    |
| 14 days     | 336   | 0.336 ZEC   | 0.504 ZEC       | 1.004 ZEC    |
| 30 days     | 720   | 0.720 ZEC   | 1.080 ZEC       | 1.580 ZEC    |
| 90 days     | 2160  | 2.160 ZEC   | 3.240 ZEC       | 3.740 ZEC    |
| 365 days    | 8760  | 8.760 ZEC   | 13.140 ZEC      | 13.640 ZEC   |

**At current ZEC prices (~$30):**
- 1 month: ~$47
- 3 months: ~$112
- 1 year: ~$409

*Prices vary. This is for illustration only.*

## Maximizing Your Rewards

### 1. Keep Your Node Synced
- Ensure Zebra stays updated
- Monitor sync status regularly
- Restart if sync stalls

### 2. Serve the Network
- Open port 8233 (mainnet) or 18233 (testnet)
- Allow incoming connections
- Maintain stable uptime
- **Bonus: 1.5x reward multiplier!**

### 3. Optimize Uptime
- Use reliable hardware
- Stable internet connection
- Consider UPS for power backup
- Set up monitoring/alerts

### 4. Submit Proofs Regularly
- Don't wait too long between submissions
- Weekly submissions recommended
- Track your earnings on dashboard

## How Rewards Are Calculated

When you submit a proof, our system:

1. **Verifies the ZK proof** - Ensures it's cryptographically valid
2. **Checks block height** - Confirms against Zcash network state
3. **Validates timestamp** - Prevents replay attacks
4. **Calculates sync bonus** - If you haven't claimed it yet
5. **Computes uptime reward** - Based on hours since last proof
6. **Applies multiplier** - 1.5x if serving peers
7. **Queues payout** - Adds to weekly distribution

## Reward Caps

To ensure sustainability:

| Limit Type | Value | Notes |
|------------|-------|-------|
| Max sync bonus | 0.5 ZEC | One-time per node |
| Max daily claim | 0.036 ZEC | 24 hours × 0.001 × 1.5 |
| Max monthly (est.) | 1.08 ZEC | 30 days uptime |
| Proof frequency | 1 per 24h | Rate limit |

## Payment Methods

### Option 1: Zcash (ZEC)

Sent directly to your Zcash address.

**Supported addresses:**
- Transparent (t-addr): `t1abc...`
- Shielded (z-addr): `zs1xyz...` (recommended for privacy)

**Transaction fees:** Paid by DePINZcash

### Option 2: Solana (SOL)

Rewards converted to SOL and sent to your Solana wallet.

**Conversion rate:** Market rate at time of distribution

**Why Solana?**
- Fast, cheap transactions
- Easy integration with DeFi
- Popular in DePIN ecosystem

## Future Enhancements

### Planned Reward Additions

1. **Lightwalletd Hosting** (Phase 2)
   - Additional 0.0005 ZEC/hour
   - Serves light clients
   - Lower barrier to entry

2. **Network Quality Bonus** (Phase 3)
   - Up to 2x multiplier
   - Based on peer reputation
   - Measured by served requests

3. **Staking Rewards** (Phase 4)
   - Lock ZEC for bonus rewards
   - 10 ZEC stake → 1.25x multiplier
   - 50 ZEC stake → 1.5x multiplier

## Tax Considerations

**Important:** Running a node and earning rewards may have tax implications in your jurisdiction.

**Consult a tax professional** regarding:
- Income from mining/staking activities
- Capital gains when selling rewards
- Record-keeping requirements

DePINZcash provides:
- Detailed earning history
- CSV export for tax reporting
- Timestamp of all payouts

## FAQ

### Q: When do I get my first reward?

After you submit your first proof and it's verified. Initial payouts are processed weekly.

### Q: Can I run multiple nodes?

Yes, but each must use a different wallet address. Same address = same node.

### Q: What if my node goes offline?

Uptime rewards pause when offline. No penalty, just no accrual. Resume when back online.

### Q: How often should I submit proofs?

**Recommended:** Weekly. Balances effort with reward tracking.

### Q: Is there a minimum payout?

Currently 0.1 ZEC. Rewards accumulate until you reach this threshold.

### Q: Can rewards change?

Yes. We may adjust rates based on:
- Network growth
- ZEC price volatility
- Community governance (future)

Changes announced 30 days in advance.

### Q: What if I miss the sync bonus?

No problem. Just submit a proof when fully synced. Bonus available anytime you reach 75%+.

## Getting Started

1. **Install Zebra**: [zebra.zfnd.org](https://zebra.zfnd.org/)
2. **Sync blockchain**: Let it run until 100%
3. **Install prover**: Clone this repo and run `./scripts/setup.sh`
4. **Generate proof**: Run `./scripts/generate_proof.sh`
5. **Submit proof**: Upload to [depinzcash.io/submit](https://depinzcash.io/submit)
6. **Get paid**: Rewards distributed weekly

## Support

Questions about rewards?

- **Discord:** [discord.gg/depinzcash](https://discord.gg/depinzcash)
- **Twitter:** [@DePINZcash](https://twitter.com/DePINZcash)
- **Email:** rewards@depinzcash.io

---

**Start earning today by strengthening Zcash privacy infrastructure!** 🦓⚡
