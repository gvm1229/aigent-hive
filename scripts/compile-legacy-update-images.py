#!/usr/bin/env python3
"""Compile public predecessor PE fingerprints without executing or installing a package.

Inputs are already downloaded npm tarballs plus their public registry metadata. Runtime
verification reads only the fixed image headers and immutable sections described here.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import struct
import tarfile

VERSIONS = ("0.13.1", "0.13.2-test.1", "0.13.2-test.2")
MAX_IMAGE = 128 * 1024 * 1024


def digest(data):
    return hashlib.sha256(data).hexdigest()


def image_fingerprint(binary):
    def unpack(kind, offset):
        if offset < 0 or offset + struct.calcsize(kind) > len(binary):
            raise ValueError("PE field outside the public image")
        return struct.unpack_from(kind, binary, offset)

    if len(binary) > MAX_IMAGE or binary[:2] != b"MZ":
        raise ValueError("invalid public PE image")
    pe, = unpack("<I", 0x3C)
    if binary[pe:pe + 4] != b"PE\0\0":
        raise ValueError("missing PE signature")
    machine, count = unpack("<HH", pe + 4)
    optional_size, = unpack("<H", pe + 20)
    optional = pe + 24
    magic, = unpack("<H", optional)
    if machine != 0x8664 or magic != 0x20B or not 1 <= count <= 32 or optional_size < 240:
        raise ValueError("only known Windows x64 PE32+ layouts are supported")
    image_base, = unpack("<Q", optional + 24)
    image_size, header_size = unpack("<II", optional + 56)
    directories, = unpack("<I", optional + 108)
    if not 0 < header_size <= 16384 or not header_size <= image_size <= MAX_IMAGE or directories < 13:
        raise ValueError("invalid image bounds")
    section_start = optional + optional_size
    if section_start + count * 40 > header_size or header_size > len(binary):
        raise ValueError("section headers escaped image headers")
    sections = []
    for index in range(count):
        offset = section_start + index * 40
        name = binary[offset:offset + 8].rstrip(b"\0").decode("ascii")
        size, rva, raw_size, raw_offset = unpack("<IIII", offset + 8)
        flags, = unpack("<I", offset + 36)
        if any(prior["name"] == name for prior in sections):
            raise ValueError("duplicate public section name")
        if size <= 0 or rva < header_size or rva + size > image_size or raw_offset + raw_size > len(binary):
            raise ValueError("section escaped public image")
        if any(rva < prior["rva"] + prior["size"] and prior["rva"] < rva + size for prior in sections):
            raise ValueError("overlapping public sections")
        sections.append(dict(name=name, size=size, rva=rva, raw_size=raw_size, raw_offset=raw_offset, flags=flags))

    def raw_bytes(rva, size):
        for section in sections:
            relative = rva - section["rva"]
            if 0 <= relative and relative + size <= section["raw_size"]:
                offset = section["raw_offset"] + relative
                return binary[offset:offset + size]
        raise ValueError("directory escaped a public section")

    reloc_rva, reloc_size = unpack("<II", optional + 112 + 5 * 8)
    reloc_bytes = raw_bytes(reloc_rva, reloc_size) if reloc_size else b""
    relocations = []
    offset = 0
    while offset < len(reloc_bytes):
        if offset + 8 > len(reloc_bytes):
            raise ValueError("truncated relocations")
        page, size = struct.unpack_from("<II", reloc_bytes, offset)
        if size < 8 or size % 2 or offset + size > len(reloc_bytes):
            raise ValueError("invalid relocation block")
        for entry in range(offset + 8, offset + size, 2):
            value, = struct.unpack_from("<H", reloc_bytes, entry)
            kind, location = value >> 12, page + (value & 4095)
            if kind == 0:
                continue
            if kind != 10 or location + 8 > image_size:
                raise ValueError("unsupported relocation")
            relocations.append(location)
        offset += size
    if relocations != sorted(set(relocations)):
        raise ValueError("duplicate or unordered relocations")

    iat_rva, iat_size = unpack("<II", optional + 112 + 12 * 8)
    if iat_size:
        raw_bytes(iat_rva, iat_size)
    immutable = []
    selected_relocations = []
    for section in sections:
        flags = section["flags"]
        if not flags & 0x40000000 or flags & (0x80000000 | 0x02000000):
            continue
        rva, size = section["rva"], section["size"]
        initialized = min(section["raw_size"], size)
        data = bytearray(raw_bytes(rva, initialized) + bytes(size - initialized))
        mask = None
        if iat_size and rva <= iat_rva < rva + size:
            if iat_rva + iat_size > rva + size:
                raise ValueError("import address table crossed an immutable section")
            mask = dict(offset=iat_rva - rva, size=iat_size)
            data[mask["offset"]:mask["offset"] + iat_size] = bytes(iat_size)
        for relocation in relocations:
            if rva <= relocation < rva + size:
                if relocation + 8 > rva + size:
                    raise ValueError("relocation crossed an immutable section")
                selected_relocations.append(relocation)
        immutable.append(dict(name=section["name"], rva=rva, size=size, sha256=digest(data), import_address_mask=mask))
    if not any(section["flags"] & 0x20000000 and section["name"] in {s["name"] for s in immutable} for section in sections):
        raise ValueError("no immutable executable section")
    # Delta varints keep the compiled public relocation list small without a runtime codec.
    encoded = bytearray()
    previous = 0
    for address in sorted(selected_relocations):
        delta = address - previous
        previous = address
        while delta >= 128:
            encoded.append((delta & 127) | 128)
            delta >>= 7
        encoded.append(delta)
    return dict(binary_sha256=digest(binary), header_size=header_size, header_sha256=digest(binary[:header_size]), image_base_offset=optional + 24,
                image_base=image_base, image_size=image_size, sections=immutable,
                relocation_count=len(selected_relocations), relocations_delta_base64=base64.b64encode(encoded).decode())


def package_fingerprint(archive, metadata, version):
    if version not in VERSIONS or metadata["name"] != "@aigent-hive/win32-x64" or metadata["version"] != version:
        raise ValueError("predecessor package identity differs")
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(archive).digest()).decode()
    if metadata["dist"]["integrity"] != integrity:
        raise ValueError("public package integrity differs")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        member = package.getmember("package/bin/hive.exe")
        if not member.isfile() or not 0 < member.size <= MAX_IMAGE:
            raise ValueError("invalid public binary member")
        binary = package.extractfile(member).read(MAX_IMAGE + 1)
    return dict(version=version, package_integrity=integrity, **image_fingerprint(binary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    images = []
    for version in VERSIONS:
        archive = (args.input_dir / f"{version}.tgz").read_bytes()
        metadata = json.loads((args.input_dir / f"{version}.json").read_text(encoding="utf-8"))
        images.append(package_fingerprint(archive, metadata, version))
    value = dict(schema_version=1, platform="windows-x64", images=images)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"versions": VERSIONS, "sha256": digest(args.output.read_bytes()), "bytes":args.output.stat().st_size}))


if __name__ == "__main__":
    main()
