
# ⚡ Zero-Capital Arbitrage Web App

**Zero-Capital Arbitrage** is a next-generation DeFi arbitrage platform that lets users execute **flash-loan arbitrage**, **triangular arbitrage**, **cross-chain arbitrage**, **mint arbitrage**, and **JIT liquidity** — all with **zero upfront capital** and **zero upfront gas fees**.

A real-time radar scans **55+ DEXes across 6 chains** (Ethereum, Arbitrum, Optimism, Polygon, BSC, Avalanche) to surface every price difference, then executes the optimal trade via **Velora (ex-ParaSwap) V5 split-routing**, funded by flash loans from **Aave V3 / Radiant V2 / Spark Protocol**, and submitted through **Flashbots / MEV-Share** private bundles or **ERC-4337 paymasters** (Pimlico / ZeroDev).

---

## 📖 Table of Contents / جدول المحتويات

- [English](#-english)
  - [Features](#-features)
  - [Architecture](#-architecture)
  - [Tech Stack](#-tech-stack)
  - [How It Works](#-how-it-works)
  - [Quick Start](#-quick-start)
  - [Prerequisites](#-prerequisites)
  - [Configuration](#-configuration)
  - [Running the Project](#-running-the-project)
  - [API Reference](#-api-reference)
  - [Smart Contract](#-smart-contract)
- [العربية](#-العربية)
  - [الميزات](#-الميزات)
  - [الهندسة المعمارية](#-الهندسة-المعمارية)
  - [تقنيات التطوير](#-تقنيات-التطوير)
  - [طريقة العمل](#-طريقة-العمل)
  - [البدء السريع](#-البدء-السريع)
  - [المتطلبات الأساسية](#-المتطلبات-الأساسية)
  - [الإعداد والتشغيل](#-الإعداد-والتشغيل)
  - [مرجع API](#-مرجع-api)
  - [العقد الذكي](#-العقد-الذكي)

---

## 🇬🇧 English

### ✨ Features

| Feature | Description |
|---------|-------------|
| **Zero-Capital Arbitrage** | Execute trades funded by flash loans. Borrow from Aave V3 / Radiant V2 / Spark Protocol — repay from profit. |
| **Zero-Upfront Gas** | Two gas strategies: Flashbots/MEV-Share (miner paid from profit, $0 if revert) or ERC-4337 paymasters (Pimlico/ZeroDev, gas paid in profit tokens). |
| **55+ DEX Scanner** | Real-time price scanner across 55 DEXes on 6 chains (Ethereum, Arbitrum, Optimism, Polygon, BSC, Avalanche). |
| **Velora (ex-ParaSwap) V5 Split Routing** | Visual pie chart showing how Velora (ex-ParaSwap) splits the trade across multiple DEXes for the best price. |
| **All Prices Matrix** | View every single DEX price in a sortable, filterable table with chain summaries. |
| **6 Arbitrage Strategies** | Flash Loan Arbitrage, Direct Swap, Mint Arbitrage, Triangular Arbitrage, Cross-Chain Arbitrage, JIT Liquidity / MEV. |
| **Visual Execution Flow** | Graphical route display: FlashLoan → Buy DEX → Velora (ex-ParaSwap) Split → Sell DEX → Profit → Repayment. |
| **RTL / Arabic Support** | Full Arabic interface with RTL layout toggle. |
| **Dark Glassmorphism UI** | Premium dark mode UI with glassmorphism cards, neo-glow effects, and real-time radar animations. |

### 🏗 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Next.js 14 Frontend                      │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Radar     │  │ All Prices   │  │ Advanced Execution   │   │
│  │ Dashboard │  │ Matrix       │  │ Engine (6 strategies)│   │
│  ├──────────┤  ├──────────────┤  ├──────────────────────┤   │
│  │ Wallet   │  │ Route Vis.   │  │ Transaction Log      │   │
│  │ Connect  │  │ (Recharts)   │  │ (Persistent History) │   │
│  └──────────┘  └──────────────┘  └──────────────────────┘   │
│                      wagmi + viem                           │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTP / WebSocket
┌──────────────────────────▼──────────────────────────────────┐
│                    Rust Backend (Axum)                       │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐   │
│  │ Radar Scanner  │  │ Velora (ex-ParaSwap) V5   │  │ WebSocket    │   │
│  │ (6 chains      │  │ Client (split │  │ Hub (real-   │   │
│  │  × 55 DEXes)   │  │  routing API) │  │ time scans)  │   │
│  └────────────────┘  └────────────────┘  └──────────────┘   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         Multi-Strategy Execution Engine              │    │
│  │  Flash Loan │ Direct Swap │ Mint │ Triangular │ JIT │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │ Blockchain RPC (alloy-rs)
┌──────────────────────────▼──────────────────────────────────┐
│                  Smart Contracts (Solidity)                   │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  ZeroRiskArb.sol                                      │    │
│  │  ├── Aave V3 flashLoanSimple()                       │    │
│  │  ├── Radiant V2 flashLoan()                          │    │
│  │  ├── Spark Protocol flashLoanSimple()                │    │
│  │  ├── Velora (ex-ParaSwap) V8 Augustus call()                     │    │
│  │  ├── Profit validation + safe revert                 │    │
│  │  └── Flashbots block.coinbase.transfer               │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

### 🛠 Tech Stack

| Layer | Technology |
|-------|-----------|
| **Frontend** | Next.js 14, React 18, TypeScript, TailwindCSS, Recharts |
| **Web3** | wagmi 2.x, viem 2.x, MetaMask, WalletConnect |
| **Backend** | Rust, Axum, alloy-rs, tokio-tungstenite, serde |
| **Smart Contracts** | Solidity 0.8.20, Foundry (forge, cast, anvil) |
| **Flash Loans** | Aave V3, Radiant V2, Spark Protocol |
| **DEX Aggregation** | Velora (ex-ParaSwap) V5 API (split routing) |
| **Gas Strategies** | Flashbots MEV-Share, Pimlico (ERC-4337), ZeroDev (ERC-4337) |

### 🔄 How It Works

#### Arbitrage Flow

```
1. SCAN      → User enters token symbol
               Rust backend queries 55+ DEXes across 6 chains in parallel
               Returns: every price + best buy/sell opportunities

2. ANALYZE   → All 6 strategies are evaluated:
               • Simple Arb: cheapest buy → most expensive sell
               • Triangular: 3-token cycle profit
               • Cross-Chain: bridge price differences
               • Mint: mint vs market price
               • JIT: mempool large-swap monitoring
               • Velora (ex-ParaSwap): optimal split-route pricing

3. EXECUTE   → User selects strategy + flash loan source + gas strategy:
               • Spark Protocol (0% fee on DAI) ← Priority
               • Aave V3 (0.05% fee)
               • Radiant V2 (0.03% fee)
               
4. SETTLE    → Smart contract flow:
               FlashLoan → Velora (ex-ParaSwap).call(data) → Calculate Profit
               → Repay Loan + Fee → Send Profit to User
               → Pay Miner Tip (Flashbots) or Burn (ERC-4337)
               
5. VERIFY    → If balance < loan + fee + minProfit → entire tx REVERTS
               User loses absolutely $0.00
```

### 🚀 Quick Start

#### Prerequisites

- **Rust** 1.77+ (install via [rustup](https://rustup.rs))
- **Node.js** 18+ and **pnpm** / **npm**
- **Foundry** (install via `curl -L https://foundry.paradigm.xyz | bash`)
- **MetaMask** browser extension
- **Git**

#### Environment Variables

```bash
# Copy the example env files
cp .env.example .env
cp frontend/.env.local.example frontend/.env.local
```

`backend/.env`:
```env
ETH_RPC_URL=https://eth.merkle.io
ARB_RPC_URL=https://arb1.arbitrum.io/rpc
OP_RPC_URL=https://mainnet.optimism.io
POLY_RPC_URL=https://polygon-rpc.com
BSC_RPC_URL=https://bsc-dataseed.binance.org
AVAX_RPC_URL=https://api.avax.network/ext/bc/C/rpc
PARASWAP_API_KEY=
PORT=3001
FLASHBOOTS_RELAY=https://relay.flashbots.net
PIMLICO_API_KEY=
ZERODEV_PROJECT_ID=
```

`frontend/.env.local`:
```env
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=
```

#### Installation

```bash
# 1. Clone the repository
git clone https://github.com/YOUR_USERNAME/zero-cap-arb.git
cd zero-cap-arb

# 2. Install Rust backend dependencies
cd backend
cargo build --release
cd ..

# 3. Install Foundry dependencies (smart contracts)
cd contracts
forge install
forge build
cd ..

# 4. Install frontend dependencies
cd frontend
npm install
# or: pnpm install
cd ..
```

#### Running the Project

```bash
# Terminal 1: Start the Rust backend
cd backend
cargo run --release

# Terminal 2: Start the Next.js frontend (separate terminal)
cd frontend
npm run dev

# Terminal 3: (Optional) Start Anvil local testnet for contract testing
cd contracts
anvil

# Run Foundry tests
cd contracts
forge test
```

Open **http://localhost:3000** in your browser.

### 📡 API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Health check + connected chains |
| `POST` | `/api/scan` | Scan simple arbitrage opportunities |
| `POST` | `/api/all-prices` | Get all DEX prices (full matrix) |
| `POST` | `/api/all-opportunities` | Scan all 6 arbitrage strategies |
| `POST` | `/api/paraswap/price` | Get Velora (ex-ParaSwap) V5 price + split routes |
| `POST` | `/api/paraswap/build-tx` | Build Velora (ex-ParaSwap) transaction calldata |
| `POST` | `/api/execute` | Execute basic arbitrage |
| `POST` | `/api/execute/advanced` | Execute any strategy (flash loan / swap / mint / triangular / cross-chain / JIT) |
| `GET` | `/api/chains` | List all supported chains |
| `GET` | `/api/dexes/{chainId}` | List DEXes for a specific chain |
| `WS` | `/ws` | Real-time WebSocket radar updates |

### 📜 Smart Contract

**`ZeroRiskArb.sol`** is the core execution contract:

- **Flash Loan Integration**: Implements Aave V3 `IFlashLoanSimpleReceiver`, Radiant V2 multi-asset flash loans, and Spark Protocol flash loans (0% fee on DAI).
- **Velora (ex-ParaSwap) Execution**: Calls the Velora (ex-ParaSwap) Augustus contract with exact calldata from the Velora (ex-ParaSwap) V5 API `buildTransaction()`.
- **Profit Safety**: `if (balance < loan + fee + minProfit) revert` — user loses $0 if trade fails.
- **Flashbots Tips**: Swaps 10% of profit to WETH and sends via `block.coinbase.transfer`.
- **Gas Optimized**: 1M optimizer runs, `via-ir` compilation, `uint8` packed enums.
- **Foundry Tests**: 8 test cases including fuzz testing and gas benchmarks.

```bash
# Deploy (example)
forge script script/Deploy.s.sol --rpc-url eth --broadcast

# Test
forge test --match-path src/ZeroRiskArb.t.sol -vvv
```

---

## 🇸🇦 العربية

### ✨ الميزات

| الميزة | الوصف |
|--------|-------|
| **مراجحة بدون رأس مال** | تنفيذ الصفقات عبر القروض الفورية. اقترض من Aave V3 / Radiant V2 / Spark Protocol — وسدد من الأرباح. |
| **غاز بدون دفعة مقدمة** | استراتيجيتان للغاز: Flashbots (يدفع المعدن من الربح، 0$ إذا فشلت) أو ERC-4337 (يدفع الغاز من أرباح الصفقة). |
| **ماسح 55+ بورصة لامركزية** | مسح فوري لـ 55 بورصة لامركزية عبر 6 سلاسل (إيثريوم، أربيتروم، أوبتيمزم، بوليجون، بي إن بي، أفالانش). |
| **تقسيم المسار عبر Velora (ex-ParaSwap) V5** | رسم بياني دائري يوضح كيفية توزيع Velora (ex-ParaSwap) الصفقة عبر عدة بورصات للحصول على أفضل سعر. |
| **مصفوفة الأسعار الكاملة** | عرض كل سعر من كل بورصة في جدول قابل للفرز والتصفية مع ملخص كل سلسلة. |
| **6 استراتيجيات مراجحة** | المراجحة بالقروض الفورية، المبادلة المباشرة، المراجحة بالسك (الصك)، المراجحة المثلثية، المراجحة عبر السلاسل، السياقة الفورية / MEV. |
| **واجهة عربية كاملة** | دعم كامل للغة العربية مع تخطيط RTL. |
| **واجهة داكنة زجاجية** | واجهة عصرية داكنة بتأثيرات زجاجية وإضاءة متوهجة. |

### 🏗 الهندسة المعمارية

```
┌─────────────────────────────────────────────────────────────────┐
│                     واجهة Next.js 14                            │
│  ┌──────────────┐  ┌──────────────────┐  ┌────────────────┐     │
│  │ لوحة الرادار  │  │ مصفوفة الأسعار   │  │ محرك التنفيذ   │     │
│  │              │  │ الكاملة          │  │ المتقدم        │     │
│  ├──────────────┤  ├──────────────────┤  ├────────────────┤     │
│  │ ربط المحفظة  │  │ عرض المسار البياني│  │ سجل المعاملات  │     │
│  └──────────────┘  └──────────────────┘  └────────────────┘     │
│                        wagmi + viem                             │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP / WebSocket
┌────────────────────────────▼────────────────────────────────────┐
│                   الخادم الخلفي (Rust + Axum)                    │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐   │
│  │ ماسح الرادار     │  │ عميل Velora (ex-ParaSwap) V5 │  │ خادم WebSocket│   │
│  │ (6 سلاسل × 55   │  │ (تقسيم المسار)   │  │ (تحديث فوري)  │   │
│  │ بورصة)          │  │                  │  │              │   │
│  └──────────────────┘  └──────────────────┘  └──────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              محرك التنفيذ متعدد الاستراتيجيات             │    │
│  │  قرض فوري │ مبادلة مباشرة │ صك │ مثلثي │ عبر السلاسل │   │    │
│  └─────────────────────────────────────────────────────────┘    │
└────────────────────────────┬────────────────────────────────────┘
                             │ RPC (alloy-rs)
┌────────────────────────────▼────────────────────────────────────┐
│                      العقود الذكية (Solidity)                     │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  ZeroRiskArb.sol                                          │    │
│  │  ├── Aave V3 flashLoanSimple()                           │    │
│  │  ├── Radiant V2 flashLoan()                              │    │
│  │  ├── Spark Protocol flashLoanSimple()                     │    │
│  │  ├── تنفيذ Velora (ex-ParaSwap) Augustus                              │    │
│  │  ├── التحقق من الربح + الإلغاء الآمن                      │    │
│  │  └── دفع عمولة المعدن عبر Flashbots                       │    │
│  └──────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

### 🛠 تقنيات التطوير

| الطبقة | التقنية |
|--------|---------|
| **الواجهة الأمامية** | Next.js 14, React 18, TypeScript, TailwindCSS, Recharts |
| **الويب 3** | wagmi 2.x, viem 2.x, MetaMask, WalletConnect |
| **الخادم الخلفي** | Rust, Axum, alloy-rs, tokio-tungstenite, serde |
| **العقود الذكية** | Solidity 0.8.20, Foundry (forge, cast, anvil) |
| **القروض الفورية** | Aave V3, Radiant V2, Spark Protocol |
| **تجميع البورصات** | Velora (ex-ParaSwap) V5 API (تقسيم المسار) |
| **استراتيجيات الغاز** | Flashbots MEV-Share, Pimlico (ERC-4337), ZeroDev (ERC-4337) |

### 🔄 طريقة العمل

#### تدفق المراجحة

```
1. المسح ← يدخل المستخدم رمز العملة
            خادم Rust يمسح 55+ بورصة عبر 6 سلاسل بالتوازي
            يعرض: كل الأسعار + أفضل فرص الشراء/البيع

2. التحليل ← تقييم جميع الاستراتيجيات الست:
            • بسيط: أرخص شراء → أغلى بيع
            • مثلثي: دورة 3 عملات
            • عبر السلاسل: فرق السعر عبر الجسور
            • الصك: سعر الصك مقابل السوق
            • السيولة الفورية: مراقبة الصفقات الكبيرة

3. التنفيذ ← يختار المستخدم الاستراتيجية + مصدر القرض + استراتيجية الغاز

4. التسوية ← تدفق العقد الذكي:
            قرض فوري → تنفيذ Velora (ex-ParaSwap) → حساب الربح
            → سداد القرض + الرسوم → إرسال الربح للمستخدم
            → دفع عمولة المعدن (Flashbots)

5. التحقق ← إذا كان الرصيد < القرض + الرسوم + الحد الأدنى للربح
            → يتم إلغاء الصفقة بالكامل
            → المستخدم لا يخسر شيئًا
```

### 🚀 البدء السريع

#### المتطلبات الأساسية

- **Rust** 1.77+ (تثبيت عبر [rustup](https://rustup.rs))
- **Node.js** 18+ و **npm** / **pnpm**
- **Foundry** (تثبيت عبر `curl -L https://foundry.paradigm.xyz | bash`)
- **MetaMask** إضافة المتصفح
- **Git**

#### متغيرات البيئة

```bash
# نسخ ملفات البيئة النموذجية
cp .env.example .env
cp frontend/.env.local.example frontend/.env.local
```

`backend/.env`:
```env
ETH_RPC_URL=https://eth.merkle.io
ARB_RPC_URL=https://arb1.arbitrum.io/rpc
OP_RPC_URL=https://mainnet.optimism.io
POLY_RPC_URL=https://polygon-rpc.com
BSC_RPC_URL=https://bsc-dataseed.binance.org
AVAX_RPC_URL=https://api.avax.network/ext/bc/C/rpc
PARASWAP_API_KEY=
PORT=3001
FLASHBOOTS_RELAY=https://relay.flashbots.net
PIMLICO_API_KEY=
ZERODEV_PROJECT_ID=
```

`frontend/.env.local`:
```env
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=
```

#### التنصيب

```bash
# 1. استنساخ المستودع
git clone https://github.com/YOUR_USERNAME/zero-cap-arb.git
cd zero-cap-arb

# 2. تنصيب اعتماديات Rust
cd backend
cargo build --release
cd ..

# 3. تنصيب اعتماديات العقود الذكية
cd contracts
forge install
forge build
cd ..

# 4. تنصيب اعتماديات الواجهة الأمامية
cd frontend
npm install
cd ..
```

#### تشغيل المشروع

```bash
# الطرفية 1: تشغيل الخادم الخلفي
cd backend
cargo run --release

# الطرفية 2: تشغيل الواجهة الأمامية
cd frontend
npm run dev

# الطرفية 3: (اختياري) تشغيل شبكة اختبار محلية
cd contracts
anvil

# اختبار العقود الذكية
cd contracts
forge test
```

افتح **http://localhost:3000** في متصفحك.

### 📡 مرجع API

| الطريقة | المسار | الوصف |
|---------|--------|-------|
| `GET` | `/api/health` | فحص الصحة + السلاسل المتصلة |
| `POST` | `/api/scan` | مسح فرص المراجحة البسيطة |
| `POST` | `/api/all-prices` | الحصول على كل أسعار البورصات (المصفوفة الكاملة) |
| `POST` | `/api/all-opportunities` | مسح جميع استراتيجيات المراجحة الست |
| `POST` | `/api/paraswap/price` | الحصول على سعر Velora (ex-ParaSwap) V5 + المسارات المقسمة |
| `POST` | `/api/paraswap/build-tx` | بناء بيانات معاملة Velora (ex-ParaSwap) |
| `POST` | `/api/execute` | تنفيذ مراجحة أساسية |
| `POST` | `/api/execute/advanced` | تنفيذ أي استراتيجية (قرض فوري / مبادلة / صك / مثلثي / عبر السلاسل / سيولة فورية) |
| `GET` | `/api/chains` | عرض جميع السلاسل المدعومة |
| `GET` | `/api/dexes/{chainId}` | عرض البورصات لسلسلة محددة |
| `WS` | `/ws` | تحديثات فورية عبر WebSocket |

### 📜 العقد الذكي

**`ZeroRiskArb.sol`** هو عقد التنفيذ الأساسي:

- **القروض الفورية**: يدعم Aave V3 (`IFlashLoanSimpleReceiver`)، Radiant V2 (قروض متعددة الأصول)، وSpark Protocol (0% رسوم على DAI).
- **تنفيذ Velora (ex-ParaSwap)**: يستدعي عقد Velora (ex-ParaSwap) Augustus مع بيانات المعاملة من API Velora (ex-ParaSwap) V5.
- **سلامة الربح**: `if (balance < loan + fee + minProfit) revert` — المستخدم لا يخسر شيئًا إذا فشلت الصفقة.
- **عمولة المعدن**: يحول 10% من الربح إلى WETH ويرسل عبر `block.coinbase.transfer`.
- **محسن الغاز**: 1,000,000 دورة تحسين، تجميع `via-ir`.

```bash
# نشر العقد (مثال)
forge script script/Deploy.s.sol --rpc-url eth --broadcast

# اختبار
forge test --match-path src/ZeroRiskArb.t.sol -vvv
```

---

## 📄 License / الترخيص

MIT License — feel free to use, modify, and distribute.

## 🙏 Acknowledgments / الشكر

- [Velora (ex-ParaSwap)](https://paraswap.io) — DEX aggregation API
- [Aave](https://aave.com) — Flash loan infrastructure
- [Radiant](https://radiant.capital) — Cross-chain flash loans
- [Spark Protocol](https://sparkprotocol.io) — 0% fee DAI flash loans
- [Flashbots](https://flashbots.net) — MEV-Share private mempool
- [Pimlico](https://pimlico.io) — ERC-4337 paymaster infrastructure
- [ZeroDev](https://zerodev.app) — ERC-4337 wallet SDK

---

<p align="center">
  <b>Zero-Capital Arbitrage</b> — <i>Trade like you own the mempool. Pay nothing until you profit.</i>
</p>
<p align="center" dir="rtl">
  <b>مراجحة بدون رأس مال</b> — <i>تداول وكأنك تملك المجمع. لا تدفع شيئًا حتى تربح.</i>
</p>
