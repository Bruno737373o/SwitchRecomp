#!/usr/bin/env python3
"""Generate synthetic NRO/NSO inputs and provenance metadata."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import struct


def write_u32(buf: bytearray, offset: int, value: int) -> None:
    buf[offset : offset + 4] = struct.pack("<I", value)


def write_u64(buf: bytearray, offset: int, value: int) -> None:
    buf[offset : offset + 8] = struct.pack("<Q", value)


def align_up(value: int, align: int) -> int:
    return ((value + align - 1) // align) * align


def sha256_path(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(8192), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def push_dir_entry(
    buf: bytearray,
    parent: int,
    sibling: int,
    child_dir: int,
    child_file: int,
    next_hash: int,
    name: str,
) -> int:
    offset = len(buf)
    buf.extend(struct.pack("<IIIIII", parent, sibling, child_dir, child_file, next_hash, len(name)))
    buf.extend(name.encode("utf-8"))
    while len(buf) % 4 != 0:
        buf.append(0)
    return offset


def push_file_entry(
    buf: bytearray,
    parent: int,
    sibling: int,
    data_off: int,
    data_size: int,
    next_hash: int,
    name: str,
) -> int:
    offset = len(buf)
    buf.extend(struct.pack("<IIQQII", parent, sibling, data_off, data_size, next_hash, len(name)))
    buf.extend(name.encode("utf-8"))
    while len(buf) % 4 != 0:
        buf.append(0)
    return offset


def build_romfs_image() -> bytes:
    file_root = b"HELLO"
    file_nested = b"NESTED"
    nested_dir = "data"
    root_name = ""

    root_entry_size = align_up(0x18 + len(root_name), 4)
    nested_entry_off = root_entry_size
    nested_entry_size = align_up(0x18 + len(nested_dir), 4)
    dir_table_size = root_entry_size + nested_entry_size

    file_root_name = "hello.txt"
    file_nested_name = "nested.bin"
    file_root_entry_size = align_up(0x20 + len(file_root_name), 4)
    file_nested_off = file_root_entry_size
    file_nested_entry_size = align_up(0x20 + len(file_nested_name), 4)
    file_table_size = file_root_entry_size + file_nested_entry_size

    file_root_data_off = 0
    file_nested_data_off = align_up(len(file_root), 0x10)
    file_data = bytearray(file_root)
    file_data.extend(b"\x00" * (file_nested_data_off - len(file_data)))
    file_data.extend(file_nested)

    dir_table = bytearray()
    push_dir_entry(
        dir_table,
        0xFFFF_FFFF,
        0xFFFF_FFFF,
        nested_entry_off,
        0,
        0xFFFF_FFFF,
        root_name,
    )
    push_dir_entry(
        dir_table,
        0,
        0xFFFF_FFFF,
        0xFFFF_FFFF,
        file_nested_off,
        0xFFFF_FFFF,
        nested_dir,
    )

    file_table = bytearray()
    push_file_entry(
        file_table,
        0,
        0xFFFF_FFFF,
        file_root_data_off,
        len(file_root),
        0xFFFF_FFFF,
        file_root_name,
    )
    push_file_entry(
        file_table,
        nested_entry_off,
        0xFFFF_FFFF,
        file_nested_data_off,
        len(file_nested),
        0xFFFF_FFFF,
        file_nested_name,
    )

    header_size = 0x50
    dir_table_off = align_up(header_size, 0x10)
    file_table_off = align_up(dir_table_off + dir_table_size, 0x10)
    file_data_off = align_up(file_table_off + file_table_size, 0x10)
    total_size = file_data_off + len(file_data)
    image = bytearray(total_size)

    write_u64(image, 0x0, 0x50)
    write_u64(image, 0x8, dir_table_off)
    write_u64(image, 0x10, 0)
    write_u64(image, 0x18, dir_table_off)
    write_u64(image, 0x20, dir_table_size)
    write_u64(image, 0x28, file_table_off)
    write_u64(image, 0x30, 0)
    write_u64(image, 0x38, file_table_off)
    write_u64(image, 0x40, file_table_size)
    write_u64(image, 0x48, file_data_off)

    image[dir_table_off : dir_table_off + len(dir_table)] = dir_table
    image[file_table_off : file_table_off + len(file_table)] = file_table
    image[file_data_off : file_data_off + len(file_data)] = file_data
    return bytes(image)


def build_nro(path: Path, with_assets: bool) -> None:
    header_size = 0x80
    text = b"TEXT-SEGMENT"
    rodata = b"RODATA-SEGMENT"
    data = b"DATA-SEGMENT"

    text_off = header_size
    ro_off = text_off + len(text)
    data_off = ro_off + len(rodata)

    nro_size = header_size + len(text) + len(rodata) + len(data)
    buf = bytearray(nro_size)

    buf[0x10:0x14] = b"NRO0"
    write_u32(buf, 0x18, nro_size)
    write_u32(buf, 0x20, 0x0)
    write_u32(buf, 0x24, len(text))
    write_u32(buf, 0x28, 0x1000)
    write_u32(buf, 0x2C, len(rodata))
    write_u32(buf, 0x30, 0x2000)
    write_u32(buf, 0x34, len(data))
    write_u32(buf, 0x38, 0x20)

    build_id = b"SYNTHETIC-NRO-BUILD-ID".ljust(0x20, b"0")
    buf[0x40:0x60] = build_id

    buf[text_off : text_off + len(text)] = text
    buf[ro_off : ro_off + len(rodata)] = rodata
    buf[data_off : data_off + len(data)] = data

    if with_assets:
        asset_base = len(buf)
        asset_header_size = 0x38
        icon = b"SYNTH-ICON-DATA"
        nacp = bytearray(0x4000)
        nacp_metadata = b"SYNTHETIC NACP METADATA"
        nacp[: len(nacp_metadata)] = nacp_metadata
        romfs = build_romfs_image()

        icon_offset = asset_header_size
        nacp_offset = icon_offset + len(icon)
        romfs_offset = nacp_offset + len(nacp)
        total = asset_base + asset_header_size + len(icon) + len(nacp) + len(romfs)
        if total > len(buf):
            buf.extend(b"\x00" * (total - len(buf)))

        buf[asset_base : asset_base + 4] = b"ASET"
        write_u64(buf, asset_base + 0x8, icon_offset)
        write_u64(buf, asset_base + 0x10, len(icon))
        write_u64(buf, asset_base + 0x18, nacp_offset)
        write_u64(buf, asset_base + 0x20, len(nacp))
        write_u64(buf, asset_base + 0x28, romfs_offset)
        write_u64(buf, asset_base + 0x30, len(romfs))

        icon_start = asset_base + icon_offset
        buf[icon_start : icon_start + len(icon)] = icon
        nacp_start = asset_base + nacp_offset
        buf[nacp_start : nacp_start + len(nacp)] = nacp
        romfs_start = asset_base + romfs_offset
        buf[romfs_start : romfs_start + len(romfs)] = romfs

    path.write_bytes(buf)


def build_nso(path: Path) -> None:
    header_size = 0x100
    text = b"NSO-TEXT-SEGMENT"
    rodata = b"NSO-RODATA"
    data = b"NSO-DATA"

    text_off = header_size
    ro_off = text_off + len(text)
    data_off = ro_off + len(rodata)

    total = header_size + len(text) + len(rodata) + len(data)
    buf = bytearray(total)

    buf[0x0:0x4] = b"NSO0"
    write_u32(buf, 0x8, 0x0)
    write_u32(buf, 0x10, text_off)
    write_u32(buf, 0x14, 0x0)
    write_u32(buf, 0x18, len(text))
    write_u32(buf, 0x20, ro_off)
    write_u32(buf, 0x24, 0x1000)
    write_u32(buf, 0x28, len(rodata))
    write_u32(buf, 0x30, data_off)
    write_u32(buf, 0x34, 0x2000)
    write_u32(buf, 0x38, len(data))
    write_u32(buf, 0x3C, 0x40)

    module_id = b"SYNTHETIC-NSO-BUILD-ID".ljust(0x20, b"0")
    buf[0x40:0x60] = module_id
    write_u32(buf, 0x60, len(text))
    write_u32(buf, 0x64, len(rodata))
    write_u32(buf, 0x68, len(data))

    buf[text_off : text_off + len(text)] = text
    ro_start = ro_off
    buf[ro_start : ro_start + len(rodata)] = rodata
    data_start = data_off
    buf[data_start : data_start + len(data)] = data

    path.write_bytes(buf)


def build_provenance(path: Path, nro_path: Path, nso_path: Path) -> None:
    nro_sha = sha256_path(nro_path)
    nso_sha = sha256_path(nso_path)
    nro_size = nro_path.stat().st_size
    nso_size = nso_path.stat().st_size

    content = f"""schema_version = \"1\"\n\n[title]\nname = \"Homebrew Intake Sample\"\ntitle_id = \"0100000000000000\"\nversion = \"0.1.0\"\nregion = \"US\"\n\n[collection]\ndevice = \"demo\"\ncollected_at = \"2026-02-01\"\nnotes = \"Synthetic homebrew intake fixture with non-proprietary assets.\"\n\n[collection.tool]\nname = \"synthetic-generator\"\nversion = \"1.0\"\n\n[[inputs]]\npath = \"inputs/{nro_path.name}\"\nformat = \"nro0\"\nsha256 = \"{nro_sha}\"\nsize = {nro_size}\nrole = \"homebrew_module\"\n\n[[inputs]]\npath = \"inputs/{nso_path.name}\"\nformat = \"nso0\"\nsha256 = \"{nso_sha}\"\nsize = {nso_size}\nrole = \"auxiliary_module\"\n"""
    path.write_text(content)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate synthetic homebrew intake inputs")
    parser.add_argument("--no-assets", action="store_true", help="skip asset section")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent
    inputs_dir = root / "inputs"
    inputs_dir.mkdir(parents=True, exist_ok=True)

    nro_path = inputs_dir / "homebrew.nro"
    nso_path = inputs_dir / "overlay.nso"

    build_nro(nro_path, with_assets=not args.no_assets)
    build_nso(nso_path)
    build_provenance(root / "provenance.toml", nro_path, nso_path)

    print(f"Wrote {nro_path} ({nro_path.stat().st_size} bytes)")
    print(f"Wrote {nso_path} ({nso_path.stat().st_size} bytes)")
    print("Updated provenance.toml")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
