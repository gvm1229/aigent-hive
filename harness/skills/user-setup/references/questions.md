# One canonical question view

The installed CLI's question catalog owns order, conditions, defaults, validation and translation.
Do not maintain another question list or load the full schema for normal setup. Quick setup does
not authorize resetting existing choices. New blocking and live interruption controls are off.

For an explicitly requested explanation, a pending question ID, or an unknown input key, use:

```text
hive setup --scope user --user-root <user-root> --describe --question <question-id> --output json
```

The response contains one localized question, its current value and source, within 8KiB. A value
that exceeds the budget is explicitly omitted with its digest; never treat omission as an empty
choice. Use the explicit `--describe --section <section>` view if the user needs that full detail.
Neither describe nor a summary grants mutation, external delivery or monitor consent.
For already supplied custom choices, use only the relevant section to obtain the exact
`question.key`, input type and options. Copy those keys without renaming them; do not guess from labels.

For normal agent-driven completion, ask one pending question at a time in chat from this view and
let the CLI validate the patch. Do not reconstruct the full answer document, or change saved
language from the language of an unrelated user message.

Per-host usage questions repeat once for each selected app, in the stored app order, only when
usage protection is on. Ask them in this order for one app before the next app: (1) whether the
subscription has a 5-hour limit; (2) only after yes, which limits to guard: 5-hour only, weekly
only, or both; without a 5-hour limit, guard weekly only and skip this question; (3) one
remaining-percent threshold from 0 to 99 for a single guarded limit, or two separate thresholds
for both. Antigravity reports only provider quota pools: ask it only one remaining-percent
threshold, stored as `five_hour_limit: false`, `guard: weekly` and
`weekly_stop_remaining_percent`, and applied to every pool. Name the app in each question. The keys are
`/usage_guard/host_windows/<host>/{five_hour_limit,guard,five_hour_stop_remaining_percent,weekly_stop_remaining_percent}`
and the concrete question IDs end with `-<host>`. Never answer for another app or reuse one app's
answer for another.

The preview-browser question (`user-browser`) stores `/browser/target`. Before asking it, read the
operating system's default browser once with one read-only query and patch the raw answer into
`/browser/detected_default`: Windows `reg query HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice /v ProgId`;
macOS the `https` handler in `defaults read com.apple.LaunchServices/com.apple.launchservices.secure LSHandlers`;
Linux `xdg-settings get default-web-browser`. If the query fails, leave the key unset. Hive uses
that browser only when it is Chromium based and falls back to Chrome. Never change browser,
extension or host settings from this step.
