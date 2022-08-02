# PBA Final: Multi-Token DEX

---

## Introduction
I selected the multi-token DEX option as I know little about defi and it therefore seemed a good opportunity to
learn how liquidity pools work. My implementation is rather rudimentary and the scope has been limited to
Uniswap V1, with the addition of multi-asset pools/swaps.

Liquidity providers can provide liquidity to liquidity pools in exchange for liquidity pool (LP) tokens, which are a
claim on pool rewards. Each swap takes a 0.3% commission/fee, which is added to the liquidity pool. A liquidity
provider can then redeem their LP tokens at any point to withdraw their liquidity, along with their portion of the
rewards.

The implementation is described below, which has been integrated into a working substrate node. 
I had also hoped to create a simple swap UI using [Yew](https://yew.rs/), and explore creating a WebAssembly wrapper such as
[eth-wasm](https://github.com/evilrobotindustries/eth-wasm), but sadly ran out of time.

## Implementation
The [Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template) has been forked 
and the following pallets have been added. The pallets have been integrated into a working now and the 
runtime configuration can be found [here](runtime/src/lib.rs).

### Pallets
The pallets used to implement the solution are as follows, shown in a somewhat layered order.

#### [Assets](https://github.com/paritytech/substrate/tree/master/frame/assets) 
  - An existing FRAME pallet provided by Substrate, used to add multi-asset support
  - Asset `0` is created at genesis as a proxy of the native currency. Wrapper functions for balance transfers and 
    therefore process the

#### **[DEX](pallets/dex)**
  - A custom pallet implementing a simple decentralised exchange
  - Uses the [assets](https://github.com/paritytech/substrate/tree/master/frame/assets) pallet
  - A new asset is created for each liquidity pool. The asset identifiers start at the end of the `u32` range and
        are decremented each time via the `LiquidityPoolTokenIdGenerator` storage item. This should ideally be using 
    hashes.
  - Liquidity rewards are generated via fees on each call to the `LiquidityPool.swap()` method (via 
    `LiquidityPool::calculate()`.
  - The pallets provide two traits, which exposes pricing and swap functionality to other pallets via loose coupling:
    - `Price`: `fn price(amount: Balance, asset: AssetId, other: AssetId) -> Result<Balance, DispatchError>`
    - `Swap`: `fn swap(amount: Balance, asset: AssetId, other: AssetId, buyer: AccountId) -> DispatchResult;`
  - A runtime API has also been implemented to provide price oracle functionality. The final portion of implementing a 
    RPC client could not be completed due to trait compiler issues, but I feel like I was on the right track. See 
    [here](node/src/rpc.rs) and [here](pallets/dex/rpc) and the end of [here](runtime/src/lib.rs).

#### [Uniques](https://github.com/paritytech/substrate/tree/master/frame/uniques)
  - Existing FRAME pallet provided by Substrate, used to add non-fungible token support

#### **[Marketplace](pallets/marketplace)**
  - A custom pallet for implementing a simple NFT marketplace
  - Uses the [uniques](https://github.com/paritytech/substrate/tree/master/frame/uniques) pallet
  - Uses the [DEX](pallets/dex) pallet to auto-swap assets to facilitate buying/selling using any asset/token. It 
    also adds the `Price` trait bound fir future use.

### Genesis Config
The [genesis config](node/src/chain_spec.rs) of the chain contains the below:

#### Assets
Alice has been issued amounts of EVIL, WETH and WBTC.

| ID  | Symbol  | Name            | Decimals |
|-----|---------|-----------------|----------|
| 0   | UNIT    | Native Token    | 18       |
| 1   | EVIL 🤖 | EVIL 🤖 Coin    | 18       |
| 2   | WETH    | Wrapped Ether   | 18       |
| 3   | WBTC    | Wrapped Bitcoin | 18       |

#### Liquidity Pools
The following liquidity pools are created at genesis, funded exclusively by Alice.

| Pool ID       | Pair        | Liquidity                      |
|---------------|-------------|--------------------------------|
| 4,294,967,295 | UNIT / EVIL | 100,000 UNIT / 10,000,000 EVIL |
| 4,294,967,294 | UNIT / WETH | 100,000 UNIT / 1,000,000 WETH  |
| 4,294,967,293 | UNIT / WBTC | 100,000 UNIT / 500,000 WBTC    |
