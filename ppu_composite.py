#!/usr/bin/env python
# NES PPU composite video encoder
# Copyright (C) 2026 Persune

# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

import argparse
import sys
import numpy as np

VERSION = "0.3.0"

# signal LUTs
# voltage highs and lows
# from https://forums.nesdev.org/viewtopic.php?p=159266#p159266
# signal[6][2][2] $0x-$3x, $x0/$xD, no emphasis/emphasis
# 5th index is purely colorburst
signal_table_composite = np.array([
    [
        [ 0.616, 0.500 ],
        [ 0.228, 0.192 ]
    ],
    [
        [ 0.840, 0.676 ],
        [ 0.312, 0.256 ]
    ],
    [
        [ 1.100, 0.896 ],
        [ 0.552, 0.448 ]
    ],
    [
        [ 1.100, 0.896 ],
        [ 0.880, 0.712 ]
    ],
    # colorburst high, colorburst low
    [
        [ 0.524, 0.524 ],
        [ 0.148, 0.148 ]
    ],
    # sync level, blank level
    [
        [ 0.048, 0.048 ],
        [ 0.312, 0.312 ]
    ]
], np.float64)

# first 512 entries are exclusively for the 9-bit PPU pixel format: "eeellcccc".
# these additional entries are bitshifted to avoid collisions
SYNC_INDEX = 1 << 9
BLANK_INDEX = 2 << 9
COLORBURST_INDEX = 3 << 9

# constants for reference
# updated by normalize_table()
SYNC_LEVEL = signal_table_composite[5, 0, 0]
BLANK_LEVEL = signal_table_composite[5, 1, 0]
WHITE_LEVEL = signal_table_composite[3, 0, 0]
BLACK_LEVEL = signal_table_composite[1, 1, 0]

def parse_argv(argv):
    parser=argparse.ArgumentParser(
        description="NES PPU composite video generator",
        epilog="version " + VERSION)
    # output options
    parser.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="debug messages")

    return parser.parse_args(argv[1:])

def normalize_voltage(min, max):
    global signal_table_composite
    global SYNC_LEVEL
    global BLANK_LEVEL
    global BLACK_LEVEL
    global WHITE_LEVEL
    signal_table_composite -= min
    signal_table_composite /= abs(max - min)

    # update constants
    SYNC_LEVEL = signal_table_composite[5, 0, 0]
    BLANK_LEVEL = signal_table_composite[5, 1, 0]
    WHITE_LEVEL = signal_table_composite[3, 0, 0]
    BLACK_LEVEL = signal_table_composite[1, 1, 0]

# 2C07 phase alternation
ALTERNATE_PHASE = (
    0,

    4,
    3,
    2,
    1,
    12,
    11,
    10,
    9,
    8,
    7,
    6,
    5,

    13,
    14,
    15
)

def pal_phase(hue: int, alternate_line=False):
    """2C07 phase alternation"""
    if alternate_line:
        return ALTERNATE_PHASE[hue]
    else:
        return hue

def in_color_phase(hue: int, phase: int, alternate_line=False):
    """checks if current sample is high or low in a given color wave.

    input is assumed to be in the value range of 1-12.

    returns true if sample is high."""
    return ((pal_phase(hue, alternate_line) + phase) % 12) >= 6

def encode_composite_sample(
    pixel: int,
    wave_phase: int,
    sinusoidal_peak_generation: bool,
    cburst_phase: int,
    alternate_line=False):
    """encodes a composite sample from a given PPU pixel and a given phase"""

    if pixel >= SYNC_INDEX:
        if pixel == SYNC_INDEX:
            return SYNC_LEVEL
        elif pixel == BLANK_INDEX:
            return BLANK_LEVEL
        elif pixel == COLORBURST_INDEX:
            return signal_table_composite[
                4,
                int(not in_color_phase(cburst_phase, wave_phase, alternate_line)),
                0
            ]
        else:
            sys.exit(f"invalid PPU pixel. got {pixel:011b}")

    emphasis = (pixel & 0b111000000) >> 6
    luma =     (pixel & 0b000110000) >> 4
    hue =       pixel & 0b000001111
    # 1 = emphasis activate
    if emphasis != 0:
        attenuate = int(
            (((emphasis & 1) and not in_color_phase(
                0xC, wave_phase, alternate_line)) or
            ((emphasis & 2) and not in_color_phase(
                0x4, wave_phase, alternate_line)) or
            ((emphasis & 4) and not in_color_phase(
                0x8, wave_phase, alternate_line))) and
            (hue < 0xE)
        )
    else: attenuate = 0

    #columns $xE-$xF
    if (hue >= 0xE):
        luma = 0x1

    # generate sinusoidal waveforms with matching p-p amplitudes
    if (sinusoidal_peak_generation and hue > 0x00 and hue < 0x0D):
        wave_amp = (
            signal_table_composite[luma, 0, attenuate] -\
            signal_table_composite[luma, 1, attenuate]) / 2
        wave_dc = (
            signal_table_composite[luma, 0, attenuate] +\
            signal_table_composite[luma, 1, attenuate]) / 2

        return wave_dc + (
            np.sin(
                (2 * np.pi * (hue+0.5)/12) + (2 * np.pi / 12 * (wave_phase))
            ) * wave_amp
        )

    n_wave_level = 0

    # rows $x0 and $xD
    if (hue == 0x0):
        n_wave_level = 0
    elif (hue >= 0xD):
        n_wave_level = 1
    # 0 = waveform high; 1 = waveform low
    else:
        n_wave_level = int(
            in_color_phase(hue, wave_phase, alternate_line))

    return signal_table_composite[luma, n_wave_level, attenuate]

def main(argv=None):
    return

if __name__=='__main__':
    main(sys.argv)
