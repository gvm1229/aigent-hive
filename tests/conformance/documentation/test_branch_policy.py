"""Exercise source branch policy at real Git reference and push boundaries."""

import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/branch-policy.py"
SPEC = importlib.util.spec_from_file_location("branch_policy", SCRIPT)
POLICY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(POLICY)


class BranchPolicyTest(unittest.TestCase):
    def setUp(self):
        parent = ROOT / "tests/work/branch-policy"
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git("init", "--template=", "-b", "main")
        self.git("config", "user.name", "Branch policy fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("commit", "--allow-empty", "-m", "fixture")

    def git(self, *args, success=True):
        result = subprocess.run(["git", "-C", str(self.repo), *args],
                                capture_output=True, text=True, timeout=30)
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def install(self):
        POLICY.install_hooks(self.repo, True)

    def refs(self):
        return self.git("for-each-ref", "--format=%(refname):%(objectname)", "refs/heads").stdout

    def test_all_allowed_classes_and_nested_names(self):
        for name in (*POLICY.DEFAULTS, *(f"{kind}/topic" for kind in POLICY.CLASSES), "feature/a/b"):
            with self.subTest(name=name):
                POLICY.validate(name, self.repo)

    def test_prohibited_prefixes_and_invalid_git_syntax(self):
        for name in ("codex/refactor-x", "claude/fix-x", "person/task", "staging",
                     "main/topic", "refactor/", "fix/a..b", "fix/a b", "fix/a.lock", "@{-1}", "Fix/a", "fix/x\n"):
            with self.subTest(name=name), self.assertRaises(POLICY.PolicyError):
                POLICY.validate(name, self.repo)

    def test_creation_wrapper_rejects_before_mutation(self):
        before = self.refs()
        result = subprocess.run([sys.executable, str(SCRIPT), "--repo", str(self.repo),
                                 "create", "codex/no"], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(before, self.refs())
        self.assertEqual(POLICY.main(["--repo", str(self.repo), "create", "refactor/yes"]), 0)

    def test_reference_hook_rejects_direct_git_creation_paths(self):
        self.install()
        for command in (("branch", "codex/branch"), ("switch", "-c", "codex/switch"),
                        ("checkout", "-b", "codex/checkout"),
                        ("update-ref", "refs/heads/codex/update", "HEAD")):
            with self.subTest(command=command):
                before = self.refs()
                self.git(*command, success=False)
                self.assertEqual(before, self.refs())
        self.git("branch", "fix/allowed")

    def test_rename_helper_preserves_commit_and_rejects_invalid_destination(self):
        self.git("branch", "codex/legacy")
        self.install()
        sha = self.git("rev-parse", "codex/legacy").stdout
        self.git("branch", "-m", "codex/legacy", "refactor/legacy")
        self.assertEqual(sha, self.git("rev-parse", "refactor/legacy").stdout)
        before = self.refs()
        self.assertEqual(POLICY.main(["--repo", str(self.repo), "rename", "refactor/legacy", "codex/again"]), 1)
        self.assertEqual(before, self.refs())
        self.assertEqual(POLICY.main(["--repo", str(self.repo), "rename", "refactor/legacy", "fix/renamed"]), 0)
        self.assertEqual(sha, self.git("rev-parse", "fix/renamed").stdout)

    def test_native_rename_bypass_cannot_commit_or_push(self):
        self.git("switch", "-c", "fix/before")
        self.install()
        result = subprocess.run(["git", "-C", str(self.repo), "branch", "-m", "codex/bypass"],
                                capture_output=True, text=True)
        # Git 2.45 files-backend rename bypasses reference-transaction. Future
        # versions may reject it earlier; both paths must prevent publication.
        if result.returncode:
            self.assertEqual(POLICY.current(self.repo), "fix/before")
            return
        sha = self.git("rev-parse", "HEAD").stdout
        self.git("commit", "--allow-empty", "-m", "rejected", success=False)
        self.assertEqual(sha, self.git("rev-parse", "HEAD").stdout)
        remote = self.repo / "remote.git"
        self.git("init", "--template=", "--bare", str(remote))
        self.git("push", str(remote), "codex/bypass:refs/heads/fix/destination", success=False)

    def test_existing_invalid_branch_updates_rejected_and_deletion_allowed(self):
        self.git("switch", "-c", "codex/old")
        sha = self.git("rev-parse", "HEAD").stdout
        self.install()
        self.git("commit", "--allow-empty", "-m", "must reject", success=False)
        self.assertEqual(sha, self.git("rev-parse", "HEAD").stdout)
        self.git("switch", "main")
        self.git("branch", "-d", "codex/old")

    def test_tags_and_remote_tracking_refs_not_misclassified(self):
        self.install()
        self.git("tag", "codex/historical")
        self.git("update-ref", "refs/remotes/origin/codex/foreign", "HEAD")
        self.git("commit", "--allow-empty", "-m", "valid main update")

    def test_real_push_checks_destination_and_supports_valid_push(self):
        self.install()
        remote = self.repo / "remote.git"
        self.git("init", "--template=", "--bare", str(remote))
        self.git("push", str(remote), "HEAD:refs/heads/codex/rejected", success=False)
        result = subprocess.run(["git", "--git-dir", str(remote), "for-each-ref", "refs/heads"],
                                capture_output=True, text=True, check=True)
        self.assertEqual(result.stdout, "")
        self.git("push", str(remote), "HEAD:refs/heads/fix/accepted")

    def test_installer_preview_idempotence_and_foreign_hook_preservation(self):
        hooks = self.repo / ".git/hooks"
        hooks.mkdir(exist_ok=True)
        self.assertEqual(len(POLICY.install_hooks(self.repo, False)), 3)
        self.assertFalse((hooks / "reference-transaction").exists())
        (hooks / "pre-push").write_text("foreign hook\n")
        with self.assertRaises(POLICY.PolicyError):
            self.install()
        self.assertFalse((hooks / "reference-transaction").exists())
        self.assertEqual((hooks / "pre-push").read_text(), "foreign hook\n")
        (hooks / "pre-push").unlink()
        self.install()
        self.assertEqual(POLICY.install_hooks(self.repo, True), [])

    def test_installer_refuses_custom_hook_configuration(self):
        self.git("config", "core.hooksPath", "owned-by-user")
        with self.assertRaises(POLICY.PolicyError):
            self.install()
        self.assertFalse((self.repo / "owned-by-user").exists())

    def test_invalid_hook_input_and_ignored_completion_state(self):
        for function, args in ((POLICY.validate_push, ("bad", self.repo)),
                               (POLICY.validate_transaction, ("bad", "prepared", self.repo))):
            with self.assertRaises(POLICY.PolicyError):
                function(*args)
        POLICY.validate_transaction("bad", "committed", self.repo)
        POLICY.validate_push("(delete) " + "0" * 40 + " refs/heads/codex/old " + "a" * 40, self.repo)

    def test_main_pr_requires_same_repository_develop(self):
        POLICY.validate_pr("develop", "main", "a/b", "a/b", self.repo)
        POLICY.validate_pr("fix/x", "develop", "fork/b", "a/b", self.repo)
        for head, owner in (("fix/x", "a/b"), ("develop", "fork/b")):
            with self.assertRaises(POLICY.PolicyError):
                POLICY.validate_pr(head, "main", owner, "a/b", self.repo)
        with self.assertRaises(POLICY.PolicyError):
            POLICY.validate_pr("develop", "main", "", "", self.repo)

    def test_detached_head_needs_explicit_ref(self):
        self.git("checkout", "--detach")
        with self.assertRaises(POLICY.PolicyError):
            POLICY.current(self.repo)
        POLICY.validate("release/test", self.repo)

    def test_generated_server_policy_has_no_drift(self):
        self.assertEqual(POLICY.RULESET_PATH.read_text(encoding="utf-8"), POLICY.ruleset_text())
        self.assertEqual(POLICY.ruleset()["bypass_actors"], [])
        workflow = (ROOT / ".github/workflows/branch-policy.yml").read_text()
        self.assertIn("branches: ['**']", workflow)
        self.assertIn("github.head_ref", workflow)
        self.assertIn('pr --head "$HEAD_BRANCH"', workflow)
        self.assertNotIn("pull_request_target:", workflow)


if __name__ == "__main__":
    unittest.main()
