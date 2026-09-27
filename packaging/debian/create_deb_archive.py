#!/usr/bin/env python3
"""
Simple, portable Debian .deb (ar archive) generator without external dependencies.
Works identically on macOS, Linux, and BSD.
"""
import sys
import os
import time

def build_ar_archive(output_path, member_paths):
    with open(output_path, "wb") as out:
        out.write(b"!<arch>\n")
        
        for file_path in member_paths:
            name = os.path.basename(file_path)
            # In GNU ar (Debian standard), names are formatted up to 16 chars with a trailing slash or space
            # debian-binary, control.tar.gz, data.tar.gz
            name_field = (name).ljust(16).encode("ascii")
            
            stat = os.stat(file_path)
            mtime = str(int(stat.st_mtime)).ljust(12).encode("ascii")
            uid = "0".ljust(6).encode("ascii")
            gid = "0".ljust(6).encode("ascii")
            mode = "100644".ljust(8).encode("ascii")
            size = str(stat.st_size).ljust(10).encode("ascii")
            fmag = b"`\n"
            
            header = name_field + mtime + uid + gid + mode + size + fmag
            assert len(header) == 60, f"Header must be 60 bytes, got {len(header)}"
            out.write(header)
            
            with open(file_path, "rb") as f:
                content = f.read()
                out.write(content)
                if len(content) % 2 != 0:
                    out.write(b"\n") # 2-byte alignment padding

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: create_deb_archive.py <output.deb> <member1> <member2> ...")
        sys.exit(1)
        
    out_deb = sys.argv[1]
    members = sys.argv[2:]
    build_ar_archive(out_deb, members)
    print(f"Built {out_deb} successfully with {len(members)} members.")
