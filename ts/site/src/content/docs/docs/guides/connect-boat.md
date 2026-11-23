---
title: Connect Boat
description: Use a Boat API key and choose a wallet for your overflow jobs.
slug: docs/guides/boat
---

Boat is the first overflow provider in ciproxy. Your Boat wallet pays for
the Boat jobs of every GitHub organization in your workspace. You can use
your personal wallet or a wallet of a Boat organization.

## Connect your Boat account

Only an Admin can connect Boat.

1. Create an API key in your Boat account.
2. In Setup or in **Settings** under **Overflow**, choose **Boat**.
3. Paste the API key and enter a monthly spend limit in US dollars.
4. Select **Check key**. After Boat accepts the key, ciproxy lists the
   wallets that the key can use and selects the likely wallet.
5. Choose the wallet that pays for the sandboxes. An organization key
   offers the wallet of its own Boat organization.
6. Select **Connect Boat** to save the key, wallet, and spend limit.

## Sandbox lifetime and spend

Boat ends a sandbox at its lifetime of 6 hours. On a Boat free trial, the
lifetime is 2 hours.
[Boat start fees](/docs/guides/set-a-spend-limit/#boat-start-fees) explains
how ciproxy counts those fees.

[Connect an overflow provider](/docs/guides/connect-an-overflow-provider/)
shows how to change where jobs run, change a key, or disconnect an account.
