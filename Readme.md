# ArcDark: MPC-Powered Confidential Dark Pool Protocol

## 🌑 Overview

**ArcDark** is a high-performance, institutional-grade private trading protocol built on **Arcium** and **Solana**.

In traditional decentralized exchanges (DEXs), order visibility leads to toxic MEV (Maximum Extractable Value) and predatory front-running. **ArcDark** solves this by keeping the entire order book and matching process inside a "Black Box" of **Secure Multi-Party Computation (MPC)**. Orders are split into secret shares and processed by the network without ever reconstructing the raw data until execution.

## 🚀 Live Deployment Status (Devnet v0.8.3)

The protocol is fully operational and verified on the Arcium Devnet.

## 🧠 Core Innovation: The "Invisible Hand"

ArcDark utilizes Arcis MPC circuits to implement a **Confidential Matching Engine**:

- **Secret-Shared Order Placement:** Price and volume are split into secret shares at the client side before submission. No single node ever sees the original order.
- **Oblivious Match Logic:** Uses optimized MPC multiplexers to compare `Bid >= Ask` and calculate the midpoint execution price without revealing the inputs.
- **Privacy-First Settlement:** Only successful trades are reconstructed and committed to the Solana ledger, preventing information leakage for unfulfilled orders.

## 🛠 Build & Deploy

```
# Compile Arcis Circuits and Anchor Program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Engine:** `match_orders` (Arcis-MPC)
- **Encryption Scheme:** Linear Secret Sharing (LSS) / Shamir
- **Settlement:** Verified MXE Callback via `MatchOrdersCallback`
- **Security:** Threshold Security (Recovery Set Size 4) on Arcium Cluster 456