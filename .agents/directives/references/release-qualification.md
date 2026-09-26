# Release Qualification

## Release Qualification Ordering

- Treat a successful `Release candidate` workflow as a private artifact-generation result only.
  It is not a published public test. A public test exists only after the separate publication
  workflow succeeds with that exact successful candidate run ID, and independent registry and
  GitHub Release checks confirm the requested version.
- Keep the candidate run ID, publication run ID, exact source SHA, package version, registry tag,
  and GitHub Release tag together in the release evidence. Never infer any of them from a workflow
  name, requested dispatch input, or a successful build job.
- A release workflow that runs a script which reads a historical Git tag or commit must use a full
  checkout history (`fetch-depth: 0`). Test this requirement at the workflow level. A shallow
  checkout failure must stop before artifact upload or registry publication; repair the workflow
  and use the next permitted numbered public test when product, package, installer, metadata, or
  acceptance bytes changed.
- Before reporting a numbered public test as complete, independently query the exact npm package
  version and channel tag plus the exact GitHub prerelease tag. Verify that `latest` remains
  unchanged for a test-channel publication. Report a failed candidate as unpublished, even when a
  dispatch used a numbered test version.
- Never publish or install a stable version as exploratory, regression, acceptance, performance,
  or final release testing. Stable publication is a terminal distribution action, not a test lane.
- Before creating a stable candidate, reconcile every active plan item. Complete every item in the
  release scope except a future-version candidate that the active plan explicitly defers by ID.
- Publish a uniquely numbered public test version from the qualified `develop` commit before the
  stable candidate. A local dev build, candidate artifact, CI result, or prior stable installation
  does not replace the numbered public test.
- Install the exact public test artifact on every required acceptance host and run the active
  plan's clean-install, upgrade, rollback, recovery, fresh-session, data-preservation, and
  performance checks. Bind the evidence to the test version, source commit, artifact digest,
  operating system, and actual execution result.
- Any product, packaging, installer, metadata, or acceptance fix invalidates earlier test evidence.
  Publish the next numbered test version and repeat affected acceptance checks; never repair the
  candidate by silently reusing a version or by testing through the stable channel.
- Create the protected `main` stable candidate only after the latest numbered test is accepted and
  the active plan reports zero incomplete in-scope items. Stable publication cannot create missing
  qualification evidence or be described as testing.

Keep existing hostile and security tests. Until the first public release, do not add a new
hostile edge-case implementation or test unless it directly protects installation, canonical
data, credentials, external-path confinement, update rollback/recovery, or a regression found
in the changed behavior. Record other hardening candidates for post-release review instead of
expanding the active implementation.

After every commit:

```bash
git log -1 --format=%B
```

Verify that the message has the intended scope and contains no co-author trailer.
