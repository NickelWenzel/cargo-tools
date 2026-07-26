# Releasing Cargo Tools

Releases are built by GitHub Actions and published to both the Visual Studio
Marketplace and Open VSX. Marketplace publishing uses a short-lived Microsoft
Entra token, so the workflow does not use or store an Azure DevOps personal
access token. Open VSX publishing uses a dedicated access token stored as a
GitHub environment secret.

## One-time setup

These steps require access to the Microsoft Entra tenant, the GitHub repository
settings, the `NickelWenzel` Visual Studio Marketplace publisher, and the
`NickelWenzel` Open VSX namespace.

### Visual Studio Marketplace

#### 1. Create the workload identity

1. In Microsoft Entra ID, create an app registration named something like
   `cargo-tools-marketplace-publisher`.
2. Record its **Application (client) ID** and **Directory (tenant) ID**.
3. Do not create a client secret.
4. Under **Certificates & secrets > Federated credentials**, add a GitHub
   Actions credential with these values:
   - Organization: `NickelWenzel`
   - Repository: `cargo-tools`
   - Entity type: `Environment`
   - Environment: `vscode-marketplace`

The resulting subject must be:

```text
repo:NickelWenzel/cargo-tools:environment:vscode-marketplace
```

#### 2. Configure GitHub

In **Settings > Environments**, create an environment named
`vscode-marketplace`.

Add these environment variables (they are identifiers, not secrets):

- `AZURE_CLIENT_ID`: the Entra application's client ID
- `AZURE_TENANT_ID`: the Entra directory's tenant ID

Restrict the environment to tags matching `v*.*.*` and the protected `master`
branch. The branch allowance is needed only for the manual identity check; the
workflow's publish job still accepts tags exclusively. A required reviewer is
recommended so every use of the Marketplace identity needs explicit approval.

No `VSCE_PAT` secret should be configured.

#### 3. Authorize the identity in the Marketplace

Open the **Release** workflow in GitHub Actions and run it manually with the
`marketplace-identity` mode. This performs an OIDC login without publishing and
prints the identity's Azure DevOps profile ID. It is equivalent to running:

```bash
az rest \
  --url https://app.vssps.visualstudio.com/_apis/profile/profiles/me \
  --resource 499b84ac-1321-427f-aa17-267ca6975798
```

Copy the ID from the workflow output. In the Visual Studio Marketplace publisher
management page, open the `NickelWenzel` publisher and add that identity ID as a
member with the **Contributor** role.

### Open VSX

#### 1. Link an Eclipse account and accept the Publisher Agreement

1. Create an [Eclipse account](https://accounts.eclipse.org/user/register) and
   set its GitHub username to the same account used to sign in to Open VSX.
2. Sign in to [Open VSX](https://open-vsx.org/) with that GitHub account.
3. In the Open VSX profile settings, choose **Log in with Eclipse** and link the
   Eclipse account.
4. Open **Show Publisher Agreement**, read the agreement, and accept it.

The Eclipse Contributor Agreement is separate and is not the agreement required
for publishing extensions.

#### 2. Create a CI access token

In **Open VSX > Settings > Access Tokens**, generate a token dedicated to this
repository's GitHub Actions workflow. Copy it immediately because it is shown
only once.

#### 3. Create and claim the namespace

The extension's `publisher` field is `NickelWenzel`, so its Open VSX namespace
must be exactly `NickelWenzel` and its extension ID will be
`NickelWenzel.cargo-tools`.

Create the namespace if it does not exist:

```bash
read -rsp "Open VSX token: " OVSX_PAT
export OVSX_PAT
npx ovsx create-namespace NickelWenzel
unset OVSX_PAT
```

Creating a namespace does not make it exclusive. Follow the Open VSX namespace
ownership process to claim `NickelWenzel`; wait for the claim to be approved if
approval is required before the first automated release.

#### 4. Configure GitHub

In **Settings > Environments**, create an environment named `open-vsx` and add
the access token as an environment secret named `OVSX_PAT`.

Restrict the environment to tags matching `v*.*.*` and the protected `master`
branch. The branch allowance is used only by the manual token check; the publish
job still accepts tags exclusively. A required reviewer is recommended so each
use of the publishing token needs explicit approval.

#### 5. Verify the token

Open the **Release** workflow in GitHub Actions and run it manually with the
`openvsx-token` mode. It runs `ovsx verify-pat` against `https://open-vsx.org`
to confirm that the token can publish to `NickelWenzel`, without packaging or
publishing the extension.

## Validate the workflow without publishing

Run the **Release** workflow manually from the GitHub Actions page with the
`build` mode. It performs validation, tests, and packaging, then stops before
authentication and publication. The `marketplace-identity` and `openvsx-token`
modes independently validate the configured publishing identities without
publishing. Download the VSIX artifact from a `build` run and test it locally:

```bash
code --install-extension cargo-tools.vsix
```

## Publish a release

1. Update `CHANGELOG.md`: replace `Unreleased` with the release date in
   `YYYY-MM-DD` format.
2. Set the same version in `package.json` and `package-lock.json`. A convenient
   command for future versions is:

   ```bash
   npm version patch --no-git-tag-version
   ```

3. Run the local checks:

   ```bash
   npm ci
   npm run lint
   cargo lint-cargo
   cargo xt-test
   npm run package
   ```

4. Commit and merge the release preparation.
5. Create and push an annotated tag matching the package version:

   ```bash
   git tag -a v0.5.1 -m "Release v0.5.1"
   git push origin v0.5.1
   ```

6. If configured, approve the `vscode-marketplace` and `open-vsx` environment
   deployments.
7. Verify the new version in the
   [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=NickelWenzel.cargo-tools),
   [Open VSX](https://open-vsx.org/extension/NickelWenzel/cargo-tools), and
   GitHub Releases.

The workflow builds one VSIX, records its SHA-256 digest, publishes that exact
file to both registries in parallel, and attaches the same file and digest to
the GitHub release. The GitHub release is created only after both publications
succeed.

## Failure and retry behavior

- A build or validation failure publishes nothing.
- The registry publish jobs run in parallel, so one registry can succeed while
  the other fails. A failure in either registry creates no GitHub release. Fix
  the credential, authorization, namespace, or registry problem and re-run only
  the failed publish job; do not re-run a job that already published the version.
- If both registry publications succeeded but GitHub release creation failed,
  re-run only the failed GitHub release job.
- Published registry versions are immutable. Prepare a new patch version to
  correct an already-published package. An unexpected duplicate-version error
  must be investigated rather than skipped.

## Rotate the Open VSX token

Generate a replacement token in Open VSX, replace the `OVSX_PAT` secret in the
`open-vsx` GitHub environment, and run the manual `openvsx-token` check. After
the check succeeds, delete the old token from Open VSX. Revoke a token
immediately if it may have been exposed.

Microsoft's current identity-based publishing instructions are documented at
<https://code.visualstudio.com/api/working-with-extensions/publishing-extension>.
Open VSX publishing and namespace setup are documented at
<https://github.com/EclipseFdn/open-vsx.org/wiki/Publishing-Extensions>.
