<p align="center">
  <a href="https://strikeprotocols.com">
    <img alt="Strike" src="https://strike-public-assets.s3.amazonaws.com/images/strike-logo.png" width="250" />
  </a>
</p>

# Overview
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FStrikeProtocols%2Fstrike-solana-wallet.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FStrikeProtocols%2Fstrike-solana-wallet?ref=badge_shield)


The Strike Wallet is a Solana multi-approver program-based wallet suitable for
use by institutions and anyone else looking for effortless institutional-grade
access to the Solana ecosystem. It supports SOL and SPL tokens, staking and
dApps. The multi-approver protocol applies to transfers and dApp transactions,
policy changes, and recovery, with a different approver policy for each of these.

# Building

## **1. Install rustc, cargo and rustfmt.**

```bash
$ curl https://sh.rustup.rs -sSf | sh
$ source $HOME/.cargo/env
$ rustup component add rustfmt
```

## **2. Download the source code.**

```bash
$ git clone https://github.com/StrikeProtocols/strike-wallet.git
$ cd strike-wallet
```

## **3. Build**

```bash
$ make build
```

# Testing

## **1. In a terminal, run the unit test suite**

```bash
$ make test
```

# Vulnerability Analysis

## **1. Install [Soteria](https://www.soteria.dev/post/soteria-a-vulnerability-scanner-for-solana-smart-contracts)**

## **2. In a terminal, run the analyze target**

```bash
$ make analyze
```


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FStrikeProtocols%2Fstrike-solana-wallet.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FStrikeProtocols%2Fstrike-solana-wallet?ref=badge_large)