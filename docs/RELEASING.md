# Releasing

The same process as [zellij-sessionizer](https://github.com/laperlej/zellij-sessionizer).
Keep the two in step: a change here belongs there too.

## Versioning

Tags are `vMAJOR.MINOR.PATCH` and are the source of truth.

Bump `version` in `Cargo.toml` to match the tag as part of the release. Nothing reads
it (the published asset is `zellij-choose-tree.wasm`, with no version in the name), so a
mismatch breaks nothing, but both repos drifted for years and the manifest stopped
meaning anything. v0.5.0 is where this one was brought back in line.

A new config key or user-visible behaviour is a minor bump. Bug fixes and build changes
are a patch bump.

## Cutting a release

Local `main` is held by the bare repo, so use a fresh worktree rather than switching
branches in an existing one.

```sh
git -C .bare worktree add ../release-X.Y.Z -b release-X.Y.Z origin/main
cd ../release-X.Y.Z

# bump the manifest; the build rewrites Cargo.lock's own entry to match
sed -i '' 's/^version = ".*"$/version = "X.Y.Z"/' Cargo.toml
cargo build --release

git add Cargo.toml Cargo.lock
git commit -m "version bump"
git push origin HEAD:main

gh release create vX.Y.Z --target main --title vX.Y.Z --generate-notes
```

Then confirm the workflow attached the asset:

```sh
gh run watch
gh release view vX.Y.Z --json assets -q '.assets[] | "\(.name) \(.size) \(.state)"'
```

Expect one asset, around 1.2 MB, state `uploaded`.

## Two ways to publish a release with no asset

`.github/workflows/release.yml` triggers on `release: published`, builds the plugin, and
uploads it with a step guarded by `startsWith(github.ref, 'refs/tags/')`. Both halves
matter:

- **Creating the release as a draft.** The `published` event never fires, so the
  workflow does not run at all and the release sits there with no asset.
- **Running the workflow via `workflow_dispatch`.** `github.ref` is then a branch, so
  the build succeeds, the upload step is skipped, and the job still reports success.

Neither failure announces itself. Check the asset list after every release.

## Release notes

Generated notes have been used since v0.4.1, so `--generate-notes` matches the existing
style. Preview the exact body without creating anything:

```sh
gh api -X POST /repos/laperlej/zellij-choose-tree/releases/generate-notes \
  -f tag_name=vX.Y.Z -f target_commitish=main -f previous_tag_name=vPREV
```

To hand-write instead, pass `--notes-file <path>`.
