#!/usr/bin/env python3

import sys


def read_u32(data, offset):
    return int.from_bytes(data[offset:offset + 4], "little")


def write_u32(data, offset, value):
    data[offset:offset + 4] = value.to_bytes(4, "little")


def patch_vxp(data: bytearray, imsi: str):
    # Browser patcher prepends a '9'
    imsi_bytes = ("9" + imsi).encode("ascii")

    tag_table_offset = read_u32(data, len(data) - 12)

    pos = tag_table_offset

    while pos < len(data):
        field_id = read_u32(data, pos)
        if field_id == 0:
            break

        pos += 4

        field_len = read_u32(data, pos)
        len_pos = pos
        pos += 4

        if field_len == 0:
            continue

        # Field ID 2 -> overwrite App ID with 0xFFFFFFFF
        if field_id == 2:
            data[pos:pos + 4] = b"\xff\xff\xff\xff"

        # Field ID 0x12 -> replace IMSI
        elif field_id == 0x12:
            write_u32(data, len_pos, len(imsi_bytes))
            data[pos:pos + field_len] = imsi_bytes
            field_len = len(imsi_bytes)

        pos += field_len

    return data


def main():
    if len(sys.argv) != 3:
        print(
            f"Usage: {sys.argv[0]} <input.vxp> <output.vxp>",
            file=sys.stderr,
        )
        sys.exit(1)

    infile = sys.argv[1]
    outfile = sys.argv[2]

    with open("imsi.txt", "r") as f:
        imsi = f.read().strip()

    with open(infile, "rb") as f:
        data = bytearray(f.read())

    patched = patch_vxp(data, imsi)

    with open(outfile, "wb") as f:
        f.write(patched)

    print(f"Patched: {outfile}")


if __name__ == "__main__":
    main()