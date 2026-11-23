---
title: Connect an overflow provider
description: Run Linux jobs in your own account at an overflow provider.
---

An overflow provider runs your jobs in sandboxes in your own account. The
provider bills you directly. You can connect one account for each provider
in your workspace. Each account serves all GitHub organizations in that
workspace.

## Connect an account

Only an Admin can connect an overflow provider. You can skip this step in
Setup and connect a provider later in **Settings**, under **Overflow**.

1. In Setup or in **Settings** under **Overflow**, choose an available
   provider. For Boat, follow [Connect Boat](/docs/guides/boat/).
2. Enter an API key from your account and a
   [monthly spend limit](/docs/guides/set-a-spend-limit/) for that provider.
3. Connect the account. ciproxy checks the key with the provider before it
   saves the connection. A rejected key shows an error in the key field.

After you save the connection, the dashboard shows the account name.
[Keys that stay with ciproxy][key-custody] explains how ciproxy stores and
uses the key.

## Choose where jobs run

The first time you connect a provider, ciproxy adds it to the end of the
order of each Linux label. You can move or remove it in **Settings**,
under **Labels**. [Labels and VM sizes](/docs/reference/labels/#custom-labels)
explains the label order.

In **Settings**, under **Overflow**, **Use overflow providers** controls
all providers in the workspace. Jobs skip every provider while this switch
is off. **Wait for a free slot on your machines** sets how long a job can
wait before it goes from your machines to the next place in its label order.

The **Machines** page shows each connected account, its spend, and the
overflow wait.

## Change a key or disconnect

Open **Settings**, then **Overflow**, and find the provider's panel.

- **Change key** opens a connection with the provider's saved spend limit.
  New runners on that provider use the new key after you save it.
- **Disconnect** stops new runners on that provider. Its entry stays in
  each label order, so jobs can use it again when you reconnect.

In both cases, running jobs finish. ciproxy deletes the old key after the
last sandbox that uses it ends.

When a provider rejects a saved key, jobs skip that provider. The provider's
panel shows the date of the rejection. Connect a new key to use the provider
again.

[key-custody]: /docs/security/agent-connection/#keys-that-stay-with-ciproxy
