# OmniBox

An opinionated testing environment for chain abstracted applications. 

OmniBox contains a set of utilities and encapsulates multiple test environments into a unified framework that makes developing chain abstracted applications easier than ever.

[![Join our Telegram chat for support and community discussions.][telegram-badge]][telegram-url]

[telegram-badge]: https://img.shields.io/endpoint?color=neon&style=for-the-badge&url=https://tg.sumanjay.workers.dev/chain_abstraction
[telegram-url]: https://t.me/chain_abstraction

## Features

- Support for EVM Chains, Bitcoin and NEAR
- Pre configured accounts for easy development
- Account creation
- Easy compile and deploy to testnet
- Utilities for account derivation and signature construction
- Utilities to assert transaction propagations
- Utilities to interact easily with your deployed contracts

## The OmniBox Way

You are developing a smart contract that interacts with the MPC signer and require to propagate signed transactions to an EVM chain, Bitcoin network or NEAR.

The typical workflow would be as follows:

- Develop a smart contract that integrates the [MPC Signer], optionally using the [Omni-transaction-rs] library to create the payload
- Deploy it to the NEAR testnet 
- Interact with your contract by executing a transaction (this means you are calling the [MPC Signer])
- Get a signed transaction
- Propagate to the destination chain

Since this process is common among developers, the OmniBox provides an opinionated way to develop these type of applications.

OmniBox assumes your main goal as developer is to ensure that your integration with the [MPC Signer] and the propagation to the destination chain works seamlessly. This is only possible by following a test driven development approach and creating integration tests since the beginning.

## Pre Requisites

- [Bitcoin core]
- [Anvil]

Note: Although the OmniBox could have automatically download them, we believe it is more secure to avoid that and ensure you install these pre requisites from trusted sources.

## Installation

Add dependency

```toml
[dev_dependencies]
omni-box = "0.0.1"
```

or via cargo 

```bash
cargo add omni-box --dev
```

## Usage

To get started with the OmniBox, we simply create an OmniBox instance:

```rust
let omni_box = OmniBox::new().await;
```

The OmniBox provides high-level abstractions through `contexts`, which encapsulate interactions with specific blockchains. 

Each `context` includes utilities and features like pre-configured accounts, transaction propagation, and more.

### Contexts

#### Bitcoin Context

The Bitcoin context allows you to interact with your local Bitcoin node, where you can propagate signed transactions for testing.

```rust
// Access the btc context
let btc_context = omni_box.btc_context;

// Use pre-configured accounts
let alice_legacy = btc_context.alice_legacy;
let alice_segwit = btc_context.alice_segwit;
let bob_legacy = btc_context.bob_legacy;
let bob_segwit = btc_context.bob_segwit;

// Create custom accounts
let new_account_legacy = btc_context.create_account(AddressType.Legacy);
//or
let new account_segwit = btc_context.create_account(AddressType.Bech32);
```

#### EVM Context

The EVM context lets you interact with your local [Anvil] instance, where you can propagate signed transactions for testing.

```rust
// Access the evm context
let evm_context = omni_box.evm_context;

// Use pre-configured accounts
let alice = evm_context.alice;
let bob = evm_context.bob;

// Create custom accounts
let new_account = evm_context.create_account();
```

#### NEAR Context

The NEAR context helps you deploy and interact with a local [Near] instance, where you can propagate signed transactions for testing.

```rust
// Access the near context
let near_context = omni_box.near_context;

// Use pre-configured accounts
let alice = near_context.alice;
let bob = near_context.bob;

// Create custom accounts
let new_account = near_context.create_account();
```

## Configuration

Since OmniBox deploys your smart contract to the NEAR testnet, it requires a deployer account. This account must be configured in a `deployer.json` file located in the root of your project.

Example `deployer.json`:

```json
{
    "account_id": "your-account-id.testnet",
    "public_key": "your-public-key",
    "private_key": "your-private-key"
}
```

## Advanced Usage

In addition to the pre-configured contexts, OmniBox supports advanced configurations and workflows. Examples include custom RPC endpoints and dynamic account generation.

Stay tuned for more detailed examples and features in future updates.

<!-- References -->
[Near-Workspaces]: https://github.com/near/near-workspaces-rs
[MPC Signer]: https://github.com/near/mpc
[Bitcoin core]: https://bitcoin.org/en/download
[Anvil]: https://github.com/foundry-rs/foundry/tree/master/crates/anvil
[Omni-transaction-rs]: https://github.com/near/omni-transaction-rs