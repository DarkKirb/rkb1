#!/usr/bin/env python3
# coding=utf8
#
# Simple script to scale a KiCad footprint
# Usage:
# python kicad-resize-footprint.py <input.kicad_mod> <output.kicad_mod> <scale>
#
# Where scale is how much to scale (1 = 100%)
#
# Copyright (C) 2020, Uri Shaked.

import sys
import re

scale = float(sys.argv[3])

def scaler(prefix):
    def do_scale(val):
        x = float(val.group(1)) * scale
        y = float(val.group(2)) * scale
        return '({} {} {})'.format(prefix, x, y)
    return do_scale

with open(sys.argv[1], 'r') as in_file, open(sys.argv[2], 'w', newline='') as out_file:
    for line in in_file:
        line = re.sub(r'\(xy ([0-9-.]+) ([0-9-.]+)\)', scaler("xy"), line)
        line = re.sub(r'\(start ([0-9-.]+) ([0-9-.]+)\)', scaler("start"), line)
        line = re.sub(r'\(end ([0-9-.]+) ([0-9-.]+)\)', scaler("end"), line)
        out_file.write(line)
