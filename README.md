<div align="center">
  <br>
  <img src="resources/icons/hicolor/scalable/apps/icon.svg" width="150" />
  <h1>Tasks</h1>

  <p>A simple task management application for the COSMIC™ desktop</p>

  ![window-light.png](resources/screenshots/window-light.png#gh-light-mode-only)
  ![window-dark.png](resources/screenshots/window-dark.png#gh-dark-mode-only)

  <a href='https://flathub.org/apps/dev.edfloreshz.Tasks'>
    <img width='200' alt='Get it on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

## Build and install

```sh
git clone https://github.com/cosmic-ext-orbit/cosmic-ext-tasks.git
cd cosmic-ext-tasks
just
sudo just install
```

`just` builds the release binary and generates localized desktop and AppStream
metadata under `target/xdgen`. Use `just run` to launch Tasks or `just check`
to run Clippy. The build reads metadata translations from `i18n/*/tasks.ftl`.

## Packaging

```sh
just vendor
just build-vendored
just rootdir=package prefix=/usr install
```

## Accounts

Open **Settings → Accounts** to sign in with Google or Microsoft. The embedded `accounts`
library owns the account records and encrypted credentials inside Tasks; no separate Accounts
installation, daemon, or consent app is required. Use **Reconnect** to refresh approval,
**Remove** to delete an account and its credentials, and the toggle to pause or resume sync.

Tasks ships with Google and Microsoft desktop OAuth registrations. To use a custom
registration, set `ACCOUNTS_GOOGLE_TASKS_CLIENT_ID` or `ACCOUNTS_MICROSOFT_TASKS_CLIENT_ID`
before launching Tasks. If the custom registration requires a secret, also set the matching
`_CLIENT_SECRET`.

Sign-in fails entirely if secure storage is unavailable or locked; credentials are never stored
in plaintext or kept only in memory.

## Dependencies
- [libcosmic](https://github.com/pop-os/libcosmic?tab=readme-ov-file#building)

# Copyright and licensing

Copyright 2024 © Eduardo Flores

Tasks is released under the terms of the [GPL-3.0](LICENSE)
