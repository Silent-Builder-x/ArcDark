# ArcDark: FHE-Powered Confidential Dark Pool Protocol

## 🌑 Overview

**ArcDark** is a high-performance, institutional-grade private trading protocol built on **Arcium** and **Solana**.

In traditional decentralized exchanges (DEXs), order visibility leads to toxic MEV (Maximum Extractable Value) and predatory front-running. **ArcDark** solves this by keeping the entire order book and matching process inside a "Black Box" of **Fully Homomorphic Encryption (FHE)**. Orders are matched in their encrypted state, ensuring that neither the intent nor the volume is revealed until execution.

## 🚀 Live Deployment Status (Devnet)

The protocol is fully operational and verified on the Arcium Devnet.

- **MXE Address:** `6MtqpRvV3Uyk5TNmbiBFRhzuAjABA27K37JSM29K8qis`
- **MXE Program ID:** `DEMPEao4sqvMLqGa4M2s7tBkkGGYUdascA2fUE1o9WSi`
- **Computation Definition:** `ELRcK4WhYKU89TxLDRi6ncrJKULZWPcEXff83qasuchX`
- **Status:** `Active`

## 🧠 Core Innovation: The "Invisible Hand"

ArcDark utilizes Arcis FHE circuits to implement a **Confidential Matching Engine**:

- **Encrypted Order Placement:** Price and volume are submitted as ciphertexts.
- **Mux-Based Match Logic:** Uses optimized homomorphic multiplexers to compare `Bid >= Ask` and calculate the midpoint execution price without decryption.
- **Privacy-First Settlement:** Only successful trades are committed to the Solana ledger, preventing information leakage for unfulfilled orders.

## 🛠 Build & Deploy

```
# Compile Arcis Circuits and Anchor Program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Engine:** `match_orders` (Arcis-FHE)
- **Settlement:** Verified MXE Callback via `MatchOrdersCallback`
- **Security:** Recovery Set Size 4 on Arcium Cluster 456