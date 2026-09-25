# Source-only verification

For nonshipping source instructions, human prose, source-only directive/style
checkers, their data and related contract tests. Classify behavior, not just
file extensions. A checker change requires its own behavior/negative tests, not only a syntax check.

Do not use this tier for shipped Skills/directives affecting execution, consent, routing or closure;
compiled product, package/installer, signature/ownership enforcement, release-authority registries,
workflows, product schemas/fixtures, generated product files or binaries. Those retain 03's normal
product/release verification. This tier cannot waive a protected branch's required checks.

Before commit/push run every relevant local gate: Source Wiki index/lint for facts, human prose
style, Markdown links, plan state, directive routes/budgets, and affected source-tool/static tests.
Before pushing inspect the committed tree, not just a working tree containing unstaged hash repairs.

Passing source-only checks does not require a Rust rebuild, the full unrelated product suite,
cross-platform installation, runtime qualification or a new public test. CI for unrelated product
jobs may continue asynchronously; disclose that it was not awaited. A completed relevant failure
(documentation, directive, secret, link, packaging or affected static contract) remains blocking.
New evidence/changed inputs reopen affected checks only. Do not rerun unchanged broad evidence
for each prose/fact/receipt commit.

Source-only maintenance changes repository guidance, never the published product's version or
acceptance. No new numbered test, stable candidate, tag, package publication or installation.
For tests producing tests/work or target output, use
[the artifact lifecycle](verification.md#test-artifact-lifecycle); do not delete unreviewed evidence.
