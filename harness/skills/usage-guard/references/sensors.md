# Sensor fallback

Read only after a qualified native-sensor failure.

4. If native sensing is unavailable or unsupported and CodexBar is missing, report the returned provider-specific notification and exact `next_action`. Do not substitute a different provider:

   ```text
   hive usage fallback-install --host codex|claude --dry-run --output json
   ```

   Antigravity has no CodexBar fallback: Google's Antigravity terms prohibit third-party tools from accessing Antigravity. Only Antigravity's own statusLine usage callback (`hive usage capture --host antigravity`) counts.

   Decline executes nothing, preserves core Hive use, and leaves automatic dispatch `hive.usage-unknown`. Package-manager unavailability remains a sanitized unsupported result.
5. On explicit install acceptance, run the provider-specific dry run first and show its fixed package-manager command. Apply only after fresh current-action confirmation:

   ```text
   hive usage fallback-install --host codex|claude --apply --confirm-install --output json
   ```

   Never infer consent, install silently, request credentials, reinstall a provider CLI, or suggest CodexBar API-key or manual-cookie setup.
