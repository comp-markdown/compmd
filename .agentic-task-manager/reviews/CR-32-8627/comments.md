## RELEASING.md:129-130 · round 1 · t1
> cargo make approve-packs --dry-run   # what is staged, and what it would approve: nothing done
> cargo make approve-packs

- [x] **Hakan Serce** · 2026-11-20T21:55:45Z · c1
  When trying these, I get the following error:

  ```
  [cargo-make] INFO - Execute Command: "node" "tools/packs/approve-packs.ts" "--dry-run"
  npm notice npm tokens that bypass 2FA are being restricted for account changes and direct publishing. Learn how to prepare: https://gh.io/npm-gat-bypass2fa-deprecation
  npm error code E401
  npm error Unable to authenticate, your authentication token seems to be invalid.
  npm error To correct this please try logging in again with:
  npm error   npm login
  npm error A complete log of this run can be found in: /Users/hakanserce/.npm/_logs/2026-11-20T21_54_28_975Z-debug-0.log
  npx --yes npm@11.21.1 stage list --json failed: Unable to authenticate, your authentication token seems to be invalid. Log in from outside the repository: (cd ~ && npm login)
  Error while executing command, exit code: 1
  ```

  I tried npm login, and it forced me to pnpm login, is it somethign similar here as well?
  - **Hakan Serce** · 2026-11-20T21:58:11Z · c2
    nevermind, works when I login ourside the workspace
  - **claude:79612601** · 2026-11-20T21:58:21Z · c3
    Yes, the same thing is happening here. In this repository `package.json`'s `devEngines` names pnpm, so `npm login` run in the repository is refused and points you to pnpm. The command itself is fine: the token npm has for the registry (in `~/.npmrc`) is no longer accepted. The notice above the error suggests why: npm is restricting tokens that bypass 2FA, and `npm stage` is account-level, so it needs a login of yours, not an old token.

    From outside the repository:

    ```
    cd ~
    npm login
    npm whoami
    ```

    `npm login` opens the browser sign-in and writes a new token to `~/.npmrc`. `npm whoami` should print your user. Then `cargo make approve-packs --dry-run` works from the repository: `approve-packs` runs npm 11.21.1 from a temporary folder, and npm reads the same `~/.npmrc` there. `pnpm login` writes to `~/.npmrc` too, so if it worked for you it may already be enough. `npm whoami` from `~` tells you.

    I won't run any of this myself, since it uses your login.
