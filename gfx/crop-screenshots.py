#! /usr/bin/env python3
""""
Run with -h for usage information.
"""
import argparse
import os

# Parse command line arguments
parser = argparse.ArgumentParser(
    description="Crops down full screen, zoomed in `valve-puzzle` terminal screenshots down to just the valve model diagram.")
parser.add_argument("fnames", metavar="FILE", nargs="+",
                    type=str, help="Screenshot image to crop.")
args = parser.parse_args()

for fname in args.fnames:
    (base, ext) = os.path.splitext(os.path.basename(fname))

    (base, num, minus, plus) = base.split("-")
    (minus, plus) = (int(minus), int(plus))

    out_fname = f"{base}-{num}{ext}"

    print(f"Cropping `{fname}` to `{out_fname}`...")

    start_y = 159 - minus*37
    height = (7 + minus + plus) * 37

    os.system(
        f"convert \"{fname}\" -crop 378x{height}+1+{start_y} \"{out_fname}\"")
