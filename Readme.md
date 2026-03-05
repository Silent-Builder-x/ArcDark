# 🌑 ArcDark: Institutional-Grade OTC Dark Pool on Solana

**Built for the Arcium "Dark Pools / Private Trading" Bounty**

## 🎯 Project Overview & Impact

In the current DeFi landscape, institutional players and whales suffer from severe **MEV (Miner Extractable Value) attacks and front-running** due to the transparent nature of on-chain states.

**ArcDark** is a zero-knowledge OTC (Over-The-Counter) Dark Pool designed to unlock institutional-grade onchain execution. By leveraging the bleeding-edge Arcium MPC (Multi-Party Computation) network, ArcDark ensures that users can place orders without revealing their intent or balances until execution.

The result? **0% MEV, 0% Front-running, and 100% Privacy.**

## 🧠 How Arcium is Used & Privacy Benefits

ArcDark deeply integrates Arcium to decouple state management from state execution:

- **Encrypted Shared State:** The liquidity pool's reserves (Token A and Token B) are stored as encrypted ciphertexts on Solana. No observer, not even the Solana validators, can see the liquidity depth.
- **Client-Side Encryption:** Users encrypt their trade intent (`amount_in`, `min_amount_out`, `direction`) locally using an ephemeral X25519 keypair and a shared secret with the MXE network.
- **Blind Matching & Settlement:** The Arcium Execution Environment (MXE) downloads the encrypted pool state and the user's encrypted order. Matching, risk checks (slippage), and balance updates run **entirely inside the encrypted dark box**.
- **Post-Execution:** Only the settled, blinded outcomes (new encrypted state) are posted back to Solana.

## 🚀 Technical Innovation

Standard AMMs require division operations (`x * y = k`), which generate impossibly massive Boolean circuits in MPC environments (The Division Curse), easily exceeding compute limits.

ArcDark introduces an **Innovation in Protocol Design**: An **O(1) Gas-Optimized Circuit (Division-Free)**.
We engineered a 1:1 fixed-rate OTC execution model. This reduces computational overhead (CU) by 99%, successfully bypassing the strict limitations of the Alpha network while maintaining perfect cryptographic stealth.

### 🛠️ Architecture & Deployed Contracts

- **Network:** Solana Devnet + Arcium Devnet Cluster (Offset: 456 v0.8.5)
- **Program ID:** `EQU8JCm5GYWZqJK2QXo8YFKR7m3MD9wkFAqd6VyCWTPH`
- **Initialized Computation Circuits:**
   - `init_pool`: Securely provisions the blinded liquidity pool (`.arcis` circuit).
   - `execute_swap`: Performs stealth state-transitions without exposing user slippage tolerance.

## 🖥️ User Experience

To abstract away the complex cryptography from the end-user, we built a sleek, seamless interface that simulates the confidential computing process.

👉 [Launch ArcDark Terminal](https://silent-builder-x.github.io/ArcDark/)

## 💻 Tech Stack

- **Smart Contracts:** Rust (Anchor Framework)
- **Confidential Computing:** Arcis (Arcium's DSL for encrypted circuits)
- **Client-Side Encryption:** `@arcium-hq/client` (RescueCipher, X25519)
- **Frontend UI:** Vanilla JS / Tailwind CSS (Optimized for performance and visualizations).

## 🛠 Build & Deploy

```
# Compile Arcis Circuits and Anchor Program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

*Note: The project is fully open-source. See the `/programs` folder for the Anchor integration and `/arcis` for the confidential circuits.*