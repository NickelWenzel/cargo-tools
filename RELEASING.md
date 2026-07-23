# Releasing Cargo Tools

Releases are built by GitHub Actions and published to the Visual Studio
Marketplace with a short-lived Microsoft Entra token. The workflow does not use
or store an Azure DevOps personal access token.

## One-time setup

These steps require access to the Microsoft Entra tenant, the GitHub repository
settings, and the `NickelWenzel` Visual Studio Marketplace publisher.

### 1. Create the workload identity

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

### 2. Configure GitHub

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

### 3. Authorize the identity in the Marketplace

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

## Validate the workflow without publishing

Run the **Release** workflow manually from the GitHub Actions page with the
`build` mode. It performs validation, tests, and packaging, then stops before
authentication and publication. Download the resulting VSIX artifact and test
it locally:

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
   git tag -a v0.5.0 -m "Release v0.5.0"
   git push origin v0.5.0
   ```

6. If configured, approve the `vscode-marketplace` environment deployment.
7. Verify the new version in both the VS Code Marketplace and GitHub Releases.

The workflow builds one VSIX, records its SHA-256 digest, publishes that exact
file to the Marketplace, and attaches the same file and digest to the GitHub
release. The GitHub release is created only after Marketplace publication
succeeds.

## Failure and retry behavior

- A build or validation failure publishes nothing.
- A Marketplace failure creates no GitHub release. Fix the identity or
  authorization problem and re-run the failed jobs.
- If Marketplace publication succeeded but GitHub release creation failed,
  re-run only the failed GitHub release job. Do not republish the same version.
- Published Marketplace versions are immutable. Prepare a new patch version to
  correct an already-published package.

Microsoft's current identity-based publishing instructions are documented at
<https://code.visualstudio.com/api/working-with-extensions/publishing-extension>.
