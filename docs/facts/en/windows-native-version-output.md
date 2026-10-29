---
schema_version: 1
pair_id: windows-native-version-output
topic_slug: windows-native-version-output
language: en
counterpart: ../ko/windows-native-version-output.md
title: "Windows Native Version Output"
summary: "Decode the exact native version as UTF-8 and restore the caller encoding."
tags: [encoding, installation, windows]
aliases: []
sources:
  - "repo:scripts/install.ps1#sha256:25e0cad4ec3265eb838989036adddde01fffadbec498e0dc12302965c29d5951"
  - "repo:tests/results/runs/20261029T223136-9775bcc685b1.md#sha256:0026fabac51fcd69914724f52efcbe3dc2b84aa9370f5f0ca8c389b657914806"
  - "repo:tests/results/runs/20261029T223512-967275e97cca.md#sha256:edb5ca20375ade65d829c7df22a353456f0d4e7f1c5b517ef848ba42597d93a4"
links: [windows-direct-replacement, windows-powershell-module-isolation]
reviewed_revision: "git:a03c6a2e91d14f04e159aa2009a72ad8fa799f15"
status: active
---

# Windows Native Version Output

The Windows installer must decode a native --version reply as UTF-8 before matching its exact banner. A CP437 PowerShell 5.1 session reads the prerelease middle dot as two other characters and rejects valid public 0.14.0-test.2. Set Console.OutputEncoding only around the native call and restore it in finally. Keep exact version and nonzero-exit rejection. Actual public-binary probes and PowerShell 5.1/7 child-output tests cover decoding and restoration, not full installation. Origin: the 0.14.0 first-update completion request.
