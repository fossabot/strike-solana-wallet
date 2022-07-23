<p align="center">
  <a href="https://strikeprotocols.com">
    <img alt="Strike" src="https://strike-public-assets.s3.amazonaws.com/images/strike-logo-3d.png" width="250" />
  </a>
</p>

# Overview

Strike is the power wallet that power users have wanted for a long time. This is 
part of what we bring to the table:

- Powerful features. Multi-signer, multi-user, mutli-wallet operations. Rich permissioning
  schemes. Cryptographically enforced whitelists for transfer addresses and dApps.
  Authentication of dApp destinations to avoid malicious phishing.

- Safety & Security. Non-custodial and trustless wallets. State of the art key
  management systems that feature biometric authentication and cryptographically
  enforced approvals for every wallet action.

- Limitless Scalability. Start securely with the simplest wallet configurations and add
  users, wallets and policies over time, with complete control and with the click of a
  button.

- Great UX. Hassle-free activation, a mobile security app for key management, and a
  beautiful and intuitive user experience mean that being a power user has never been
  easier.

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
