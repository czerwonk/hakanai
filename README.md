# Hakanai (儚い)

A minimalist one-time secret sharing service built on zero-knowledge principles.

## Philosophy

Hakanai embodies the Japanese concept of transience - secrets that exist only for a moment before vanishing forever. No accounts, no tracking, no persistence. Just ephemeral data transfer with mathematical privacy guarantees.

## Core Principles

- **Zero-Knowledge**: The server never sees your data. All encryption happens client-side.
- **Single View**: Secrets self-destruct after one access. No second chances.
- **No Metadata**: We store only encrypted bytes and an ID. Nothing else.
- **Minimalist**: One function only - share secrets that disappear.

## How It Works

1. Your client (CLI or browser) encrypts the secret locally
2. Sends only the ciphertext to our server
3. You share the link with the decryption key
4. Recipient views once, then it's gone forever

## Security Opinion

We chose true client-side encryption over convenience. This means client (Browser with JS, CLI) is required to perform the crypto operations, but your secrets remain yours alone. The server is just a temporary dead drop that forgets everything.

Built for those who believe privacy isn't about having something to hide - it's about having something to protect.

## License
(c) Daniel Brendgen-Czerwonk, 2025. Licensed under [MIT](LICENSE) license.

