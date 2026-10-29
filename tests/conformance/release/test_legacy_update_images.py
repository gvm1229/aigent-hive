"""Fixed public executable fingerprints cannot accept ambiguous PE or package inputs."""
import base64
import ast
import hashlib
import importlib.util
import io
import os
import struct
import sys
import tarfile
import unittest
from unittest.mock import patch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("legacy_update_images", ROOT / "scripts/compile-legacy-update-images.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def pe_image():
    image = bytearray(0xA00)
    image[:2] = b"MZ"
    struct.pack_into("<I", image, 0x3C, 0x80)
    image[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", image, 0x84, 0x8664, 3)
    struct.pack_into("<H", image, 0x94, 240)
    optional = 0x98
    struct.pack_into("<H", image, optional, 0x20B)
    struct.pack_into("<Q", image, optional + 24, 0x140000000)
    struct.pack_into("<II", image, optional + 56, 0x4000, 0x400)
    struct.pack_into("<I", image, optional + 108, 16)
    struct.pack_into("<II", image, optional + 112 + 40, 0x3000, 12)
    struct.pack_into("<II", image, optional + 112 + 96, 0x2010, 8)
    for index, (name, size, rva, raw, flags) in enumerate([
        (b".text", 16, 0x1000, 0x400, 0x60000020),
        (b".rdata", 32, 0x2000, 0x600, 0x40000040),
        (b".reloc", 12, 0x3000, 0x800, 0x42000040),
    ]):
        offset = optional + 240 + 40 * index
        image[offset:offset + len(name)] = name
        struct.pack_into("<IIII", image, offset + 8, size, rva, 0x200, raw)
        struct.pack_into("<I", image, offset + 36, flags)
    image[0x400:0x410] = bytes(range(16))
    struct.pack_into("<Q", image, 0x600, 0x140002008)
    struct.pack_into("<Q", image, 0x610, 0x140001001)
    struct.pack_into("<IIHH", image, 0x800, 0x2000, 12, 0xA000, 0)
    return image


class LegacyUpdateImageTests(unittest.TestCase):
    def test_posix_installer_fixture_binds_the_requested_test_version(self):
        tree = ast.parse((ROOT / "tests/fixtures/interactive-cli/linux-legacy-update.py").read_text(encoding="utf-8"))
        installer = next(ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign)
                         and any(isinstance(target, ast.Name) and target.id == "installer" for target in node.targets))
        validation, rest = installer.split("count=root/'install-count'", 1)
        self.assertTrue(rest)
        for version in ["0.13.2-test.2", "0.14.0-test.1", "0.14.0-test.2", "0.14.0", "0.14.0-test.0", "0.15.0-test.1"]:
            for direct in [False, True]:
                with self.subTest(version=version, direct=direct):
                    product = version.split("-test.", 1)[0]
                    environment = {"AIGENT_HIVE_VERSION": product, "AIGENT_HIVE_PACKAGE_VERSION": version,
                                   "AIGENT_HIVE_PREFIX": str(Path("/tmp/hive-first-update/package"))}
                    argv = ["sh", "install.sh"] if direct else ["npm", "install", "-g", "aigent-hive@" + version]
                    with patch.dict(os.environ, environment, clear=True), patch.object(sys, "argv", argv):
                        if version in {"0.13.2-test.2", "0.14.0-test.1", "0.14.0-test.2"}:
                            namespace = {}
                            exec(validation, namespace)
                            self.assertEqual(namespace["requested"], version)
                            self.assertEqual(namespace["product"], product)
                        else:
                            with self.assertRaises(AssertionError):
                                exec(validation, {})
        environment = {"AIGENT_HIVE_VERSION": "0.14.0", "AIGENT_HIVE_PACKAGE_VERSION": "0.14.0-test.2",
                       "AIGENT_HIVE_PREFIX": str(Path("/tmp/hive-first-update/package"))}
        for changed in [{"AIGENT_HIVE_VERSION": "0.13.2"}, {"AIGENT_HIVE_PREFIX": "/outside"}]:
            with patch.dict(os.environ, {**environment, **changed}, clear=True), patch.object(sys, "argv", ["sh", "install.sh"]):
                with self.assertRaises(AssertionError):
                    exec(validation, {})
        with patch.object(sys, "argv", ["npm", "install", "-g", "foreign@0.14.0-test.2"]):
            with self.assertRaises(AssertionError):
                exec(validation, {})

    def test_image_hashes_include_immutable_code_and_data_with_only_loader_addresses_excluded(self):
        image = pe_image()
        result = MODULE.image_fingerprint(image)
        self.assertEqual(result["image_base_offset"], 0xB0)
        self.assertEqual([section["name"] for section in result["sections"]], [".text", ".rdata"])
        self.assertEqual(result["sections"][0]["sha256"], hashlib.sha256(bytes(range(16))).hexdigest())
        expected = image[0x600:0x620]
        expected[16:24] = bytes(8)
        self.assertEqual(result["sections"][1]["sha256"], hashlib.sha256(expected).hexdigest())
        self.assertEqual(result["sections"][1]["import_address_mask"], {"offset":16,"size":8})
        self.assertEqual(result["relocation_count"], 1)
        self.assertEqual(base64.b64decode(result["relocations_delta_base64"]), b"\x80\x40")
        changed = image.copy()
        changed[0x400] ^= 1
        self.assertNotEqual(MODULE.image_fingerprint(changed)["sections"][0]["sha256"], result["sections"][0]["sha256"])

    def test_malformed_headers_sections_and_relocations_are_rejected(self):
        mutations = [
            (0x3C, "<I", (0xFFFFFF,)),
            (0x84, "<H", (0xAA64,)),
            (0x98 + 56, "<I", (0x1000,)),
            (0x98 + 240 + 16, "<I", (0xFFFF,)),
            (0x98 + 240 + 40 + 12, "<I", (0x1000,)),
            (0x804, "<I", (7,)),
            (0x808, "<H", (0x5000,)),
            (0x80A, "<H", (0xA000,)),
        ]
        for offset, kind, values in mutations:
            with self.subTest(offset=offset):
                changed = pe_image()
                struct.pack_into(kind, changed, offset, *values)
                with self.assertRaises(ValueError):
                    MODULE.image_fingerprint(changed)

    def test_only_integrity_bound_supported_public_packages_are_compiled(self):
        data = io.BytesIO()
        with tarfile.open(fileobj=data, mode="w:gz") as archive:
            member = tarfile.TarInfo("package/bin/hive.exe")
            member.size = len(pe_image())
            archive.addfile(member, io.BytesIO(pe_image()))
        archive = data.getvalue()
        metadata = {"name":"@aigent-hive/win32-x64","version":"0.13.1",
                    "dist":{"integrity":"sha512-" + base64.b64encode(hashlib.sha512(archive).digest()).decode()}}
        self.assertEqual(MODULE.package_fingerprint(archive, metadata, "0.13.1")["version"], "0.13.1")
        for version in ["0.13.0", "0.14.0"]:
            with self.assertRaises(ValueError):
                MODULE.package_fingerprint(archive, metadata, version)
        with self.assertRaises(ValueError):
            MODULE.package_fingerprint(archive + b"tampered", metadata, "0.13.1")


if __name__ == "__main__":
    unittest.main()
