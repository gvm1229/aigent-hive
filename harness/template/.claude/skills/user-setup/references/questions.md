# One canonical question view

The installed CLI's question catalog owns order, conditions, defaults, validation and translation.
Do not maintain another question list or load the full schema for normal setup. Quick setup does
not authorize resetting existing choices. New blocking and live interruption controls are off.

For an explicitly requested explanation or a pending question ID, use:

```text
hive setup --scope user --user-root <user-root> --describe --question <question-id> --output json
```

The response contains one localized question, its current value and source, within 8KiB. A value
that exceeds the budget is explicitly omitted with its digest; never treat omission as an empty
choice. Use the explicit `--describe --section <section>` view if the user needs that full detail.
Neither describe nor a summary grants mutation, external delivery or monitor consent.

For normal completion, let the visible CLI terminal ask and validate inputs. Do not mirror those
questions in chat, reconstruct the full answer document, or change saved language from the language
of an unrelated user message.
