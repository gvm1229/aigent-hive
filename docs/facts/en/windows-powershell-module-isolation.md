---
schema_version: 1
pair_id: windows-powershell-module-isolation
topic_slug: windows-powershell-module-isolation
language: en
counterpart: ../ko/windows-powershell-module-isolation.md
title: "Windows PowerShell Module Isolation"
summary: "The Windows installer computes SHA-256 without module autoload so CMD-inherited PowerShell 7 module paths cannot break PowerShell 5.1."
tags: [installation, powershell, security, windows]
aliases: ["PSModulePath isolation"]
sources:
  - "repo:docs/plans/active/windows-shell-install.md#sha256:1a1f2b2e0657be2b2aa03bb5ea258cac5a8e9f8b81630b7105a7470d8b298654"
links: [test-distribution]
reviewed_revision: "git:cdde668bed5f3b35e08a35f64e7e25594ce9c3a2"
status: active
---

# Windows PowerShell Module Isolation

The CMD bootstrap launches Windows PowerShell 5.1. A machine may place PowerShell 7 module
directories before Windows PowerShell directories in inherited `PSModulePath`; module autoload
can then select an incompatible `Microsoft.PowerShell.Utility` and make `Get-FileHash`
unavailable. The installer avoids that search path by hashing regular files directly with
`System.Security.Cryptography.SHA256`. PowerShell 7 is not a consumer dependency. Acceptance:
PowerShell 5.1 first install, repeat install, pending-receipt recovery, and contaminated-CMD
execution preserve version and digest verification. Origin: the maintainer requested an exact
explanation and a clean-clone-safe Windows fix.
