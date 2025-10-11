#!/usr/bin/env python3
# Pally - palette generator for the NES
# Copyright (C) 2025 Persune
# inspired by PalGen, Copyright (C) 2018 DragWx <https://github.com/DragWx>
# also a testbed for video decoding concepts in
# https://www.nesdev.org/wiki/NTSC_video#Composite_decoding

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

import os, sys
import argparse
import numpy as np
import ppu_composite as ppu

VERSION = "0.23.0"

def parse_argv(argv):
    parser=argparse.ArgumentParser(
        description="yet another NES palette generator",
        epilog="version " + VERSION)
    # output options
    parser.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="debug messages")
    parser.add_argument(
        "--skip-plot",
        action="store_true",
        help="skips showing the palette plot")
    parser.add_argument(
        "-o",
        "--output",
        type=str,
        help="file output, extension determined with \"--file-format\". if rendering with \"--render-img\", interpreted as folder output. if rendering with \"--test-image\", interpreted as image output.")
    parser.add_argument(
        "-f",
        "--file-format",
        choices=[
            ".pal uint8",
            ".pal double",
            ".pal Jasc",
            ".gpl",
            ".png",
            ".txt HTML hex",
            ".txt MediaWiki",
            ".h uint8_t",
        ],
        default=".pal uint8",
        help="file output format. default = \".pal uint8\"")
    parser.add_argument(
        "-e",
        "--emphasis",
        action="store_true",
        help="include emphasis entries")
    parser.add_argument(
        "-t",
        "--test-image",
        type=str,
        help="256x240 uint16 raw binary PPU frame buffer input to be applied with a palette")

    # visualization options
    parser.add_argument(
        "-r",
        "--render-img",
        type=str,
        help="render views and diagrams as images in output folder with the provided file extension.")
    parser.add_argument(
        "-w",
        "--waveforms",
        action="store_true",
        help="view composite waveforms")
    parser.add_argument(
        "-p",
        "--phase-QAM",
        action="store_true",
        help="view QAM demodulation")

    # generation options
    parser.add_argument(
        "-n",
        "--normalize",
        type=str,
        help="normalize all colors within gamut by scaling values",
        choices=[
            "scale",
            "scale clip negative"
        ])
    parser.add_argument(
        "-ppu",
        "--ppu",
        type=str,
        help="PPU chip used for generating colors. default = 2C02",
        choices=[
            "2C02",
            "2C03",
            "2C04-0000",
            "2C04-0001",
            "2C04-0002",
            "2C04-0003",
            "2C04-0004",
            "2C05-99",
            "2C07",
        ],
        default = "2C02")
    parser.add_argument(
        "-c",
        "--clip",
        type=str,
        help="clips out-of-gamut RGB colors",
        choices=[
            "darken",
            "desaturate"
        ])

    # color decoding options
    parser.add_argument(
        "-bri",
        "--brightness",
        type = np.float64,
        help = "luma brightness delta in IRE units, default = 0.0",
        default = 0.0)
    parser.add_argument(
        "-con",
        "--contrast",
        type = np.float64,
        help = "luma contrast factor, default = 1.0",
        default = 1.0)
    parser.add_argument(
        "-hue",
        "--hue",
        type = np.float64,
        help = "chroma hue angle delta, in degrees, default = 0.0",
        default = 0)
    parser.add_argument(
        "-sat",
        "--saturation",
        type = np.float64,
        help ="chroma saturation factor, default = 1.0",
        default = 1)
    parser.add_argument(
        "-blp",
        "--black-point",
        type = np.float64,
        help = "black point, in IRE units, default = 0 IRE")
    parser.add_argument(
        "-whp",
        "--white-point",
        type = np.float64,
        help = "white point, in IRE units, default = level $20")
    parser.add_argument(
        "-gai",
        "--gain",
        type = np.float64,
        help = "gain adjustment to signal before decoding, in IRE units, default = 0.0",
        default = 0.0)
    parser.add_argument(
        "-gam",
        "--gamma",
        type = np.float64,
        help = "if defined, will apply a simple OETF gamma transfer function instead, where the EOTF function is assumed to be gamma 2.2.")
    parser.add_argument(
        "--delay-line-filter",
        action = "store_true",
        help = "use 1D delay line comb filter decoding instead of single-line decoding")
    parser.add_argument(
        "-axs",
        "--axis-shift",
        type = str,
        help = "axis adjustment for R-Y and G-Y like Sony CXA2025AS, default = None",
        choices=[
            "None",
            "CXA2025AS_JP",
            "CXA2025AS_US",
            "bisqwit_NTSC_1953",
        ],
        default = "None")
    parser.add_argument(
        "-bse",
        "--burst-saturation-enable",
        action = "store_true",
        help = "enable using colorburst amplitude as saturation reference")
    parser.add_argument(
        "-spg",
        "--sinusoidal-peak-generation",
        action = "store_true",
        help = "generate sine waves in composite encoding instead of square waves")

    # analog effects options
    parser.add_argument(
        "-phs",
        "--phase-skew",
        type = np.float64,
        help = "hue shift per luma for composite PPUs, in degrees, default = 0.0",
        default = 0)
    # analog effects options
    parser.add_argument(
        "-phd",
        "--phase-distortion",
        type = np.float64,
        help = "amount of voltage-dependent impedance for RC lowpass, where RC = \"amount * (level/composite_white) * 1e-8\". this will also desaturate and hue shift the resulting colors nonlinearly. a value of 4 very roughly corresponds to a -5 degree delta per luma row. default = 0.0",
        default = 0.0)
    parser.add_argument(
        "-aps",
        "--antiemphasis-phase-skew",
        type = np.float64,
        help = "additonal hue shift on colors $x2/$x6/$xA for composite PPUs, in degrees, default = 0.0",
        default = 0)
    parser.add_argument(
        "-ela",
        "--emphasis-luma-attenuation",
        type = np.float64,
        help = "additonal luma brightness on colors $x4/$x8/$xC for composite PPUs, in voltage units, default = 0.0",
        default = 0)

    # colorimetry options
    parser.add_argument(
        "-rfc",
        "--reference-colorspace",
        type = str,
        help = "use colour.RGB_COLOURSPACES reference colorspace, default = \"SMPTE C\"",
        default = 'SMPTE C')
    parser.add_argument(
        "-dsc",
        "--display-colorspace",
        type = str,
        help = "use colour.RGB_COLOURSPACES display colorspace, default = \"sRGB\"",
        default = 'sRGB')
    parser.add_argument(
        "-cat",
        "--chromatic-adaptation-transform",
        type = str,
        help = "chromatic adaptation transform method, default = None",
        default = None)
    parser.add_argument(
        "-ict",
        "--inverse-chromatic-transform",
        action="store_true",
        help = "invert direction of chromatic adaptation transform method (from display to reference colorspace)")
    parser.add_argument(
        "-oetf",
        "--opto-electronic",
        type=str,
        help="applies \"colour.models\" color component transform function to use as opto-electronic transform function override",
        )
    parser.add_argument(
        "-eotf",
        "--electro-optic",
        type=str,
        help="applies \"colour.models\" color component transform function to use as electro-optic transform function override",
        )
    parser.add_argument(
        "--opto-electronic-disable",
        action = "store_true",
        help = "disable converting linear light to linear signal")
    parser.add_argument(
        "--electro-optic-disable",
        action = "store_true",
        help = "disable converting linear signal to linear light")
    parser.add_argument(
        "-cld",
        "--colorimetry-disable",
        action = "store_true",
        help = "disable all colorimetry functions")

    # colorimetry reference RGB and whitepoint primaries
    parser.add_argument(
        "-rpr",
        "--reference-primaries-r",
        type = np.float64,
        nargs=2,
        help = "set custom reference color primary R, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-rpg",
        "--reference-primaries-g",
        type = np.float64,
        nargs=2,
        help = "set custom reference color primary G, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-rpb",
        "--reference-primaries-b",
        type = np.float64,
        nargs=2,
        help = "set custom reference color primary B, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-rpw",
        "--reference-primaries-w",
        type = np.float64,
        nargs=2,
        help = "set custom reference whitepoint, in CIE xy chromaticity coordinates")

    # colorimetry display RGB and whitepoint primaries
    parser.add_argument(
        "-dpr",
        "--display-primaries-r",
        type = np.float64,
        nargs=2,
        help = "set custom display color primary R, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-dpg",
        "--display-primaries-g",
        type = np.float64,
        nargs=2,
        help = "set custom display color primary G, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-dpb",
        "--display-primaries-b",
        type = np.float64,
        nargs=2,
        help = "set custom display color primary B, in CIE xy chromaticity coordinates")
    parser.add_argument(
        "-dpw",
        "--display-primaries-w",
        type = np.float64,
        nargs=2,
        help = "set custom display whitepoint, in CIE xy chromaticity coordinates")

    return parser.parse_args(argv[1:])


# matrix function to convert YUV to YIQ and vice-versa
IQ_tilt = np.radians(33)
YUV_YIQ = np.array([
    [1, 0, 0],
    [0, np.cos(IQ_tilt), np.sin(IQ_tilt)],
    [0, -np.sin(IQ_tilt), np.cos(IQ_tilt)]
], np.float64)

# B-Y and R-Y reduction factors
# S170m-2004.pdf: Composite Analog Video Signal NTSC for Studio Applications. Page 16.
BY_rf = 0.492111
RY_rf = 0.877283

# derived from the NTSC base matrix of luminance and color-difference
# S170m-2004.pdf: Composite Analog Video Signal NTSC for Studio Applications. Page 4.
RGB_to_YUV = np.array([
    [ 0.299,        0.587,        0.114],
    [-0.299*BY_rf, -0.587*BY_rf,  0.886*BY_rf],
    [ 0.701*RY_rf, -0.587*RY_rf, -0.114*RY_rf]
], np.float64)

# derived from https://forums.nesdev.org/viewtopic.php?p=172817#p172817
# coefficients shifted up by 6 decimals
YIQ_to_RGB_bisqwit = np.array([
    [1,  1.994681  ,  0.9915742],
    [1,  0.09151351, -0.6334805],
    [1, -1.012984  ,  1.667217 ]
], np.float64)

RGB_to_YIQ_bisqwit = np.linalg.inv(YIQ_to_RGB_bisqwit)

RGB_to_YUV_bisqwit = np.array([
    RGB_to_YIQ_bisqwit[0,:],
    ((RGB_to_YIQ_bisqwit[2,:]*np.sin(IQ_tilt)) - (RGB_to_YIQ_bisqwit[1,:]*np.cos(IQ_tilt))),
    ((RGB_to_YIQ_bisqwit[1,:]*np.sin(IQ_tilt)) + (RGB_to_YIQ_bisqwit[2,:]*np.cos(IQ_tilt)))
], np.float64)

YUV_to_RGB_bisqwit = np.linalg.inv(RGB_to_YUV_bisqwit)

# thanks, NewRisingSun!
# Sony CXA2025AS axis offsets from the datasheet
YUV_to_RGB = np.linalg.inv(RGB_to_YUV)

CXA_JP_RY_angle = 95
CXA_JP_RY_gain = 0.78 / BY_rf
CXA_JP_GY_angle = 240
CXA_JP_GY_gain = 0.30 / BY_rf

CXA_US_RY_angle = 112
CXA_US_RY_gain = 0.83 / BY_rf
CXA_US_GY_angle = 252
CXA_US_GY_gain = 0.30 / BY_rf

YUV_to_RGB_CXA_JP = np.array([
    [YUV_to_RGB[0,0], np.sin(CXA_JP_RY_angle)*CXA_JP_RY_gain*YUV_to_RGB[0,1], np.cos(CXA_JP_RY_angle)*CXA_JP_RY_gain*YUV_to_RGB[0,2]],
    [YUV_to_RGB[1,0], np.sin(CXA_JP_GY_angle)*CXA_JP_GY_gain*YUV_to_RGB[1,1], np.cos(CXA_JP_GY_angle)*CXA_JP_GY_gain*YUV_to_RGB[1,2]],
    YUV_to_RGB[2,:]
], np.float64)

YUV_to_RGB_CXA_US = np.array([
    [YUV_to_RGB[0,0], np.sin(CXA_US_RY_angle)*CXA_US_RY_gain*YUV_to_RGB[0,1], np.cos(CXA_US_RY_angle)*CXA_US_RY_gain*YUV_to_RGB[0,2]],
    [YUV_to_RGB[1,0], np.sin(CXA_US_GY_angle)*CXA_US_GY_gain*YUV_to_RGB[1,1], np.cos(CXA_US_GY_angle)*CXA_US_GY_gain*YUV_to_RGB[1,2]],
    YUV_to_RGB[2,:]
], np.float64)

composite_black = ppu.signal_table_composite[1, 1, 0]
composite_white = ppu.signal_table_composite[3, 0, 0]
composite_CbL = ppu.signal_table_composite[4, 1, 0]
composite_CbH = ppu.signal_table_composite[4, 0, 0]

# figure plotting for palette preview
# TODO: interactivity
def palette_plot(RGB_buffer,
    RGB_uncorrected,
    emphasis,
    all_emphasis = False,
    export_diagrams = False,
    export_img = False,
    args=None,
    s_colorspace = None,
    t_colorspace = None):
    if args.skip_plot and not (export_diagrams or export_img):
        return

    import matplotlib.pyplot as plt
    import matplotlib.gridspec as gridspec
    import colour.plotting

    # if emphasis enabled, reshape to single emphasis permutation
    if (args.emphasis):
        RGB_sub = RGB_buffer.reshape(8,4,16,3)[emphasis]
        RGB_sub_raw = RGB_uncorrected.reshape(8,4,16,3)[emphasis]
    else:
        RGB_sub = RGB_buffer
        RGB_sub_raw = RGB_uncorrected

    fig = plt.figure(tight_layout=True, dpi=96)
    gs = gridspec.GridSpec(2, 2)

    ax0 = fig.add_subplot(gs[1, 1])
    ax1 = fig.add_subplot(gs[0, 1], projection='polar')
    # preview on indexed image (if provided)
    if (args.test_image is not None):
        if all_emphasis or not args.emphasis:
            ax2 = fig.add_subplot(gs[0, 0])
            ax3 = fig.add_subplot(gs[1, 0])
            with open(args.test_image, mode="rb") as index_file:
                index_image = np.reshape(np.frombuffer(index_file.read(), dtype=np.uint16), (240, 256))
                preview_image = np.empty([240,256,3], np.float64)
                for y in range(index_image.shape[0]):
                    for x in range(index_image.shape[1]):
                        preview_image[y,x] = RGB_sub[(index_image[y,x] >> 4), (index_image[y,x] & 0x0F)]
                ax3.set_title("Palette preview")
                ax3.imshow(preview_image)
        else:
            ax2 = fig.add_subplot(gs[:, 0])
            print("test image with emphasis mode enabled, test image preview will not be rendered.")
    else:
        ax2 = fig.add_subplot(gs[:, 0])

    if (export_diagrams):
        fig.suptitle('NES palette (emphasis = {0:03b})'.format(emphasis))
    else:
        fig.suptitle('NES palette')

    # colors
    ax0.set_title("Color swatches")
    ax0.imshow(RGB_sub)

    # polar plot
    YUV_calc = np.einsum('ij,klj->kli', RGB_to_YUV, RGB_sub, dtype=np.float64)
    color_theta = np.arctan2(YUV_calc[:, :, 2], YUV_calc[:, :, 1])
    color_r = YUV_calc[:, :, 0]

    ax1.set_title("RGB color phase")
    ax1.set_yticklabels([])
    ax1.axis([0, 2*np.pi, 0, 1])
    ax1.scatter(color_theta, color_r, c=np.reshape(RGB_sub,(RGB_sub.shape[0]*RGB_sub.shape[1], 3)), marker=None, s=color_r*500, zorder=3)

    # CIE graph
    colour.plotting.plot_RGB_chromaticities_in_chromaticity_diagram_CIE1931(
        RGB_sub_raw,
        colourspace=s_colorspace,
        show_whitepoints=False,
        scatter_kwargs=dict(c=np.reshape(RGB_sub_raw,(RGB_sub_raw.shape[0]*16, 3)),alpha=0.1),
        plot_kwargs=dict(color="gray"),
        figure=fig,
        axes=ax2,
        standalone=False,
        show_diagram_colours=False,
        show_spectral_locus=True,
        spectral_locus_colours='RGB',
        transparent_background=False)
    colour.plotting.plot_RGB_chromaticities_in_chromaticity_diagram_CIE1931(
        RGB_sub,
        colourspace=t_colorspace,
        show_whitepoints=False,
        scatter_kwargs=dict(c=np.reshape(RGB_sub,(RGB_sub.shape[0]*16, 3))),
        plot_kwargs=dict(color="red"),
        figure=fig,
        axes=ax2,
        standalone=False,
        show_diagram_colours=False,
        show_spectral_locus=True,
        spectral_locus_colours='RGB',
        transparent_background=False)
    ax2.set_title("CIE 1931 chromaticity diagram")
    ax2.grid(which='both', color='grey', linewidth=0.5, linestyle='-', alpha=0.2)
    ax2.axis([0, 1, 0, 1])
    ax2.set_aspect('equal', 'box')

    fig.set_size_inches(20, 11.25)
    if (args.test_image is not None and (all_emphasis or not args.emphasis)):
        # only tighten the layout if the palette preview is enabled
        fig.tight_layout()
    if args.output is not None:
        if (export_diagrams):
            outpath: str = os.path.splitext(args.output)[0]+"/palette_preview_emphasis_{0:03}.{1}".format(emphasis, args.render_img)
            plt.savefig(outpath)
        elif (export_img):
            outpath: str = os.path.splitext(args.output)[0]+"/palette_preview.{0}".format(args.render_img)
            plt.savefig(outpath)
    if not args.skip_plot:
        plt.show()
    plt.close()

def composite_QAM_plot(voltage_buffer,
    U_comb,
    V_comb,
    U_decode,
    V_decode,
    U_buffer,
    V_buffer,
    buffer_size,
    emphasis,
    luma,
    hue,
    sequence_counter,
    args=None,
    signal_black_point=None,
    signal_white_point=None):

    import matplotlib.pyplot as plt
    import matplotlib.gridspec as gridspec

    fig = plt.figure(tight_layout=True)
    gs = gridspec.GridSpec(3, 2)

    axY = fig.add_subplot(gs[0,0])
    axU = fig.add_subplot(gs[1,0])
    axV = fig.add_subplot(gs[2,0])
    ax1 = fig.add_subplot(gs[:,1], projection='polar')
    fig.suptitle("QAM demodulating ${0:02X} emphasis {1:03b}".format((luma<<4 | hue), emphasis))
    x = np.arange(0,buffer_size)
    Y_avg = np.average(voltage_buffer[0])
    U_avg = np.average(U_buffer)
    V_avg = np.average(V_buffer)

    range_axis = ((signal_white_point / (signal_white_point - signal_black_point)) - signal_black_point) * 140
    axY.set_title("Y decoding")
    axY.set_ylabel("IRE")
    axY.axis([0, buffer_size, -50, range_axis])
    if (args.delay_line_filter):
        axY.plot(x, voltage_buffer[1], 'o-', linewidth=0.7, label='composite signal')
        axY.plot(x, voltage_buffer[0], 'o-', linewidth=0.7, label='composite signal delay')
    else:
        axY.plot(x, voltage_buffer[0], 'o-', linewidth=0.7, label='composite signal')
    axY.plot(x, np.full((buffer_size), Y_avg), 'o-', linewidth=0.7, label='Y value = {:< z.3f}'.format(Y_avg))
    axY.legend(loc='lower right')

    axU.set_title("U decoding")
    axU.set_ylabel("IRE")
    axU.axis([0, buffer_size, -range_axis, range_axis])
    if (args.delay_line_filter):
        axU.plot(x, U_comb, 'o-', linewidth=0.7, label='comb filtered signal')
    axU.plot(x, U_decode * 140 / 2, 'o-', linewidth=0.7, label='U subcarrier')
    axU.plot(x, U_buffer, 'o-', linewidth=0.7, label='signal * U subcarrier')
    axU.plot(x, np.full((buffer_size), U_avg), 'o-', linewidth=0.7, label='U value = {:< z.3f}'.format(U_avg))
    axU.legend(loc='lower right')

    axV.set_title("V decoding")
    axV.set_ylabel("IRE")
    axV.axis([0, buffer_size, -range_axis, range_axis])
    if (args.delay_line_filter):
        axV.plot(x, V_comb, 'o-', linewidth=0.7, label='comb filtered signal')
    axV.plot(x, V_decode * 140 / 2, 'o-', linewidth=0.7, label='V subcarrier')
    axV.plot(x, V_buffer, 'o-', linewidth=0.7, label='signal * V subcarrier')
    axV.plot(x, np.full((buffer_size), V_avg), 'o-', linewidth=0.7, label='V value = {:< z.3f}'.format(V_avg))
    axV.legend(loc='lower right')

    color_theta = np.arctan2(V_avg, U_avg) + 2*np.pi
    if color_theta >= 2*np.pi: color_theta -= 2*np.pi
    color_r =  np.sqrt(U_avg**2 + V_avg**2)
    ax1.axis([0, 2*np.pi, 0, 60])
    ax1.set_title("UV Phasor plot")
    ax1.scatter(color_theta, color_r, label='Hue phase = {:< z.3f}'.format(np.rad2deg(color_theta)))
    ax1.vlines(color_theta, 0, color_r, label='Hue saturation = {:< z.3f}'.format(color_r))
    ax1.legend()

    fig.set_size_inches(16, 9)
    if args.render_img is not None and args.output is not None:
        plt.savefig(os.path.splitext(args.output)[0]+"/QAM_phase_{0:03}.{1}".format(sequence_counter, args.render_img), dpi=96)
    else:
        plt.show()
    plt.close()

def composite_waveform_plot(voltage_buffer, emphasis, luma, hue, sequence_counter, args=None):
    import matplotlib.pyplot as plt

    fig = plt.figure(tight_layout=True)
    fig.suptitle("${0:02X} emphasis {1:03b}".format((luma<<4 | hue), emphasis))
    ax = fig.subplots()
    x = np.arange(0,12)
    y = voltage_buffer
    ax.axis([0, 12, -50, 140])
    ax.set_xlabel("Sample count")
    ax.set_ylabel("IRE")
    ax.plot(x, y, 'o-', linewidth=0.7)

    fig.set_size_inches(16, 9)
    if args.render_img is not None and args.output is not None:
        plt.savefig(os.path.splitext(args.output)[0]+"/waveform_phase_{0:03}.{1}".format(sequence_counter, args.render_img), dpi=120)
    else:
        plt.show()
    plt.close()

def normalize_RGB(RGB_buffer, args=None):
    def color_clip_darken(RGB_buffer):
        for luma in range(RGB_buffer.shape[0]):
            for chroma in range(RGB_buffer.shape[1]):
                # if any of the RGB channels are greater than 1
                if np.any(np.greater(RGB_buffer[luma, chroma], [1.0, 1.0, 1.0])):
                    # subtract all channels by delta of greatest channel
                    # algorithm by DragWx
                    darken_factor = np.max(RGB_buffer[luma, chroma])
                    RGB_buffer[luma, chroma] /= darken_factor

    def color_clip_desaturate(RGB_buffer):
        for luma in range(RGB_buffer.shape[0]):
            for chroma in range(RGB_buffer.shape[1]):
                # if any of the RGB channels are greater than 1
                if np.any(np.greater(RGB_buffer[luma, chroma], [1.0, 1.0, 1.0])):
                    # desaturate that specific color until channels are within range
                    # algorithm by DragWx
                    darken_factor = np.max(RGB_buffer[luma, chroma])
                    YUV_calc = np.matmul(RGB_to_YUV, RGB_buffer[luma, chroma])
                    RGB_buffer[luma, chroma] -= YUV_calc[0]
                    RGB_buffer[luma, chroma] /= darken_factor
                    RGB_buffer[luma, chroma] += YUV_calc[0]

    # clip takes priority over normalize
    if (args.clip is not None):
        match args.clip:
            case "darken":
                color_clip_darken(RGB_buffer)
            case "desaturate":
                color_clip_desaturate(RGB_buffer)
    elif (args.normalize is not None):
        if (args.normalize != "scale clip negative"):
            RGB_buffer -= np.amin(RGB_buffer)
        RGB_buffer /= np.amax(RGB_buffer)

    # clip to 0.0-1.0 to ensure everything is within range
    np.clip(RGB_buffer, 0, 1, out=RGB_buffer)

def pixel_codec_composite(YUV_buffer, args=None, signal_black_point=None, signal_white_point=None):
    colorburst_phase = 0
    colorburst_offset = 0
    next_line_shift = 0
    if (args.ppu == "2C07"):
        # $x7 == -U - 45 degrees
        # will flip to -U + 45 in pal_phase()
        colorburst_phase = 7
        colorburst_offset = 2
        next_line_shift = 2
    else:
        # $x8 = -U
        colorburst_phase = 8
        colorburst_offset = 1.5
        next_line_shift = 4
    
    # on regular NTSC composite, a single delay line filter is enough to cancel
    # the chroma due to the phase being exactly 180 degrees offset on the next
    # line. but since this is NES, the 120 degree offset causes the colors to
    # shift by -30 degrees, and losing a bit of saturation.
    if args.delay_line_filter:
        colorburst_phase -= 1

    colorburst_factor = 6
    colorgen_x_factor = 2
    buffer_size = int(colorburst_factor * colorgen_x_factor)

    # timepoints for generating the subcarrier reference sines
    t = np.arange(buffer_size) - colorburst_offset - colorburst_phase

    # 2x due to integral of sin(2*PI*x)^2
    saturation_correction = 2

    if (args.burst_saturation_enable):
        # SMPTE 170M-2004, page 5, section 8.2
        # value of 40 IRE based on colorburst p-p amplitude defined in
        # SMPTE 170M-2004, page 8, Table 1
        colorburst_amp_reference = 40
        colorburst_amplitude = 140 * (composite_CbH - composite_CbL) # in IRE
        saturation_correction *= colorburst_amp_reference/colorburst_amplitude



    # signal buffers for decoding
    # 11111------1
    # 2222------22
    # 333------333
    # 44------4444
    # 5------55555
    # ------666666
    # -----777777-
    # ----888888--
    # ---999999---
    # --AAAAAA----
    # -BBBBBB-----
    # CCCCCC------

    voltage_buffer = np.empty((2, buffer_size), np.float64)

    # buffer, decode, and comb buffers for plotting
    U_buffer = np.empty((3, buffer_size), np.float64)
    V_buffer = np.empty((3, buffer_size), np.float64)

    def decode_composite(voltage_buffer, U_buffer, V_buffer):
        YUV = np.empty((3), np.float64)

        # bandpass UV component
        # no comb filter, bandpass only first line
        voltage_avg = np.average(voltage_buffer[0])
        U_buffer[2] = voltage_buffer[0] - voltage_avg
        V_buffer[2] = voltage_buffer[0] - voltage_avg

        U_buffer[0] = U_buffer[2] * U_buffer[1]
        V_buffer[0] = V_buffer[2] * V_buffer[1]

        # lowpass UV buffers
        YUV[1] = np.average(U_buffer[0])
        YUV[2] = np.average(V_buffer[0])

        # decode Y
        # also apply brightness and contrast
        YUV[0] = (voltage_avg + emphasis_row_luma) * args.contrast + args.brightness
        return YUV, U_buffer, V_buffer
    
    def decode_composite_dline(voltage_buffer, U_buffer, V_buffer):
        YUV = np.zeros((3), np.float64)

        # bandpass UV components, if PAL
        voltage_bandpass = voltage_buffer.copy()
        voltage_bandpass -= np.average(voltage_bandpass, keepdims=True)
        # bandpass and combine lines in specific way to retrieve U and V
        # based on Single Delay Line PAL Y/C Separator
        # Jack, K. (2007). NTSC and PAL digital encoding and decoding. In Video
        # Demystified (5th ed., p. 450). Elsevier.
        # https://archive.org/details/video-demystified-5th-edition/

        # naive 1D comb chroma bandpass
        V_buffer[2] = (voltage_bandpass[0] - voltage_bandpass[1]) / 2

        if (args.ppu == "2C07"):
            # invert combination to retrieve U
            U_buffer[2] = (voltage_bandpass[0] + voltage_bandpass[1]) / 2
        else:
            U_buffer[2] = (voltage_bandpass[0] - voltage_bandpass[1]) / 2

        # apply hue and saturation in decode wave

        U_buffer[0] = U_buffer[2] * U_buffer[1]
        V_buffer[0] = V_buffer[2] * V_buffer[1]

        # lowpass UV buffers
        YUV[1] = np.average(U_buffer[0])
        YUV[2] = np.average(V_buffer[0])

        # decode Y
        YUV[0] = (np.average(voltage_buffer) + emphasis_row_luma) * args.contrast + args.brightness
        return YUV, U_buffer, V_buffer

    X_main = 26601712.5 if args.ppu == "2C07" else (236250000 / 11)

    # samplerate and colorburst freqs
    # used for FFT phase shift
    PPU_Fs = X_main * colorgen_x_factor
    PPU_Cb = X_main / colorburst_factor

    # repeat signal n times to avoid edge fringes when filtering
    repeat_cycle = 10
    signal_start = 0
    # truncate signal at the end
    if args.phase_skew != 0:
        signal_start = int(buffer_size*(repeat_cycle/2))
    else:
        signal_start = int(buffer_size*repeat_cycle-buffer_size)
    signal_stop =  signal_start + buffer_size

    # phase shift the composite signal
    # https://dsp.stackexchange.com/a/95728
    from numpy.fft import fft, ifft, fftfreq
    def phase_shift(signal, angle):
        buf_size = len(signal)
        n = fftfreq(buf_size, 1/PPU_Fs)
        signal = fft(signal, n=buf_size)
        signal *= np.exp(-1j * np.deg2rad(angle)/PPU_Cb * n)
        signal = ifft(signal, n=buf_size)
        return signal

    # phase shift the composite using a simple lowpass
    # for differential phase distortion
    # https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
    def RC_lowpass(signal, amount):
        v_out = np.zeros(signal.shape, np.float64)
        v_prev = signal[0]
        dt = 1/PPU_Fs
        for i in range(len(signal)):
            # impedance changes depending on DAC tap
            # we approximate this by using the raw signal's voltage
            # https://forums.nesdev.org/viewtopic.php?p=287241#p287241
            # the phase shifts negative on higher levels according to https://forums.nesdev.org/viewtopic.php?p=186297#p186297
            v_prev_norm = signal[i] / composite_white

            # RC constant tuned so that 0x and 3x colors have around 14 degree delta when phd = 3
            # https://forums.nesdev.org/viewtopic.php?p=186297#p186297
            alpha = dt / (v_prev_norm * amount * 1e-8 + dt)
            v_prev = alpha * signal[i] + (1-alpha) * v_prev
            v_out[i] = v_prev
        return v_out

    def QAM_phase(signal):
        buf_size = len(signal)
        U = np.sin(2 * np.pi / buf_size * t)
        V = np.cos(2 * np.pi / buf_size * t)
        signal_U = np.average(signal.real * U)
        signal_V = np.average(signal.real * V)
        return np.atan2(signal_U, signal_V)

    for emphasis in range(8):
        for luma in range(4):
            for hue in range(16):
                # used for image sequence plotting
                sequence_counter = hue + (luma*16) + (emphasis*16*4)
                # zero out buffers to prevent error propagation
                voltage_buffer.fill(0)
                U_buffer.fill(0)
                V_buffer.fill(0)

                # calculate analog effects
                antiemphasis_column_chroma = (
                    args.antiemphasis_phase_skew if (
                        hue == 0x2 or
                        hue == 0x6 or
                        hue == 0xA)
                    else 0)

                emphasis_row_luma = (
                    args.emphasis_luma_attenuation if (
                        hue == 0x4 or
                        hue == 0x8 or
                        hue == 0xC)
                    else 0)


                # encode voltages into composite waveform
                voltage_buffer[0] = ppu.encode_buffer(buffer_size, args.ppu, emphasis, luma, hue, 0, args.sinusoidal_peak_generation)

                # simulate next line by incrementing wave phase and alternating phase
                voltage_buffer[1] = ppu.encode_buffer(buffer_size, args.ppu, emphasis, luma, hue, next_line_shift, args.sinusoidal_peak_generation, True)

                # subcarrier generation is 180 degrees offset
                # due to the way the waveform is encoded, the hue is off by an additional 1/2 of a sample
                # if the subcarrier moves forward in angle, the resulting hue goes backwards
                # hue argument is therefore inverse here
                if args.phase_distortion != 0:
                    # generate U and V decoder waveforms based on phase skew
                    U_offset = 0.5 if args.ppu == "2C07" else 0
                    U_decode = ppu.encode_buffer(buffer_size, args.ppu, 0, 4, ((6+colorburst_phase-1)%12)+1, 0, args.sinusoidal_peak_generation)

                    U_decode = np.tile(U_decode, repeat_cycle)
                    U_decode = RC_lowpass(U_decode, args.phase_distortion)

                    U_decode = U_decode[signal_start:signal_stop]

                    # lowpass to cutoff freq
                    U_decode = fft(U_decode, n=len(U_decode))
                    U_decode *= [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]
                    U_decode = ifft(U_decode, n=len(U_decode))

                    # determine phase shift by QAM demodulating
                    U_phase = QAM_phase(U_decode)

                    # generate a new sine using this phase offset
                    U_buffer[1] = np.cos(2 * np.pi / buffer_size * t - U_phase + U_offset +
                        np.radians(antiemphasis_column_chroma - args.hue)
                    ) * args.saturation * saturation_correction

                    # ditto for V decoder, but shift phase by 90 degrees
                    V_decode = ppu.encode_buffer(buffer_size, args.ppu, 0, 4, ((6+colorburst_phase-1+3)%12)+1, 0, args.sinusoidal_peak_generation)

                    V_decode = np.tile(V_decode, repeat_cycle)
                    V_decode = RC_lowpass(V_decode, args.phase_distortion)

                    V_decode = V_decode[signal_start:signal_stop]

                    V_decode = fft(V_decode, n=len(V_decode))
                    V_decode *= [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]
                    V_decode = ifft(V_decode, n=len(V_decode))

                    V_phase = QAM_phase(V_decode)
                    V_buffer[1] = np.cos(2 * np.pi / buffer_size * t - V_phase + U_offset +
                        np.radians(antiemphasis_column_chroma - args.hue)
                    ) * args.saturation * saturation_correction
                else:
                    U_buffer[1] = np.sin(2 * np.pi / buffer_size * t +
                        np.radians(antiemphasis_column_chroma - args.hue)
                    ) * args.saturation * saturation_correction

                    V_buffer[1] = np.cos(2 * np.pi / buffer_size * t +
                        np.radians(antiemphasis_column_chroma - args.hue)
                    ) * args.saturation * saturation_correction

                if args.phase_skew != 0:
                    phase_skew = args.phase_skew * luma

                    # repeat signal n times to avoid edge fringes
                    voltage_buffer = np.tile(voltage_buffer, repeat_cycle)

                    # phase shift according to phase_skew, if any
                    voltage_buffer[0] = phase_shift(voltage_buffer[0], phase_skew)
                    voltage_buffer[1] = phase_shift(voltage_buffer[1], phase_skew)

                    # truncate signal in the middle to get a seamless cycle
                    signal_start = int(buffer_size*(repeat_cycle/2))
                    signal_stop =  int(buffer_size*(repeat_cycle/2)+buffer_size)
                    voltage_buffer = voltage_buffer[:, signal_start:signal_stop]
                elif args.phase_distortion != 0:
                    voltage_buffer = np.tile(voltage_buffer, repeat_cycle)

                    # phase shift according to phase_skew, if any
                    voltage_buffer[0] = RC_lowpass(voltage_buffer[0], args.phase_distortion)
                    voltage_buffer[1] = RC_lowpass(voltage_buffer[1], args.phase_distortion)
                    voltage_buffer = voltage_buffer[:, signal_start:signal_stop]



                # decode voltage buffer to YUV
                # SMPTE 170M-2004, page 17, section A.5, equation 10
                # N = 0.925(Y) + 7.5 + 0.925*(U)*sin(2*π*f_sc*t) + 0.925*(V)*cos(2*π*f_sc*t)
                # scaling factor (0.925) and luma pedestal (7.5) are already accounted for during normalization

                # shift blanking to 0
                voltage_buffer -= composite_black

                # convert to IRE
                voltage_buffer *= 140

                # apply gain
                voltage_buffer += args.gain

                # decode YUV
                if args.delay_line_filter:
                    YUV_buffer[emphasis, luma, hue], U_buffer, V_buffer = decode_composite_dline(voltage_buffer, U_buffer, V_buffer)
                else:
                    YUV_buffer[emphasis, luma, hue], U_buffer, V_buffer = decode_composite(voltage_buffer, U_buffer, V_buffer)

                # visualize chroma decoding
                if (args.debug):
                    print("${0:02X} emphasis {1:03b}".format((luma<<4 | hue), emphasis) + "\n" + str(voltage_buffer[0]))
                    U_avg = YUV_buffer[emphasis, luma, hue, 1]
                    V_avg = YUV_buffer[emphasis, luma, hue, 2]
                    tau = 2*np.pi
                    color_theta = np.arctan2(V_avg, U_avg) + tau
                    if color_theta >= 2*np.pi: color_theta -= tau
                    color_r =  np.sqrt(U_avg**2 + V_avg**2)
                    delay = (color_theta/tau) * (1/PPU_Cb)
                    print("Hue angle and saturation ${0:02X}: {1}, {2}".format((luma<<4 | hue), np.rad2deg(color_theta), color_r))
                    print("Delay ${0:02X}: {1}".format((luma<<4 | hue), delay))
                if (args.waveforms):
                    composite_waveform_plot(voltage_buffer[0], emphasis, luma, hue, sequence_counter, args)
                if (args.phase_QAM):
                    composite_QAM_plot(voltage_buffer, U_buffer[2], V_buffer[2], U_buffer[1], V_buffer[1], U_buffer[0], V_buffer[0], buffer_size, emphasis, luma, hue, sequence_counter, args, signal_black_point, signal_white_point)

        if not (args.emphasis):
            # clip unused emphasis space
            YUV_buffer = np.split(YUV_buffer, 8, 0)[0]
            break

    return YUV_buffer

def pixel_codec_rgb(YUV_buffer, args=None, signal_black_point=None, signal_white_point=None):
    signal_table_rgb = np.empty([0x40], np.uint16)
    match args.ppu:
        case "2C04-0000"|"2C04-0001"|"2C04-0002"|"2C04-0003"|"2C04-0004":
            # 2C04 "sorted" PPU palette LUT
            # from https://www.nesdev.org/wiki/PPU_palettes#2C04
            signal_table_rgb = np.array([
                0x333,0x014,0x006,0x326,0x403,0x503,0x510,0x420,0x320,0x120,0x031,0x040,0x022,0x111,0x003,0x020,
                0x555,0x036,0x027,0x407,0x507,0x704,0x700,0x630,0x430,0x140,0x040,0x053,0x044,0x222,0x200,0x310,
                0x777,0x357,0x447,0x637,0x707,0x737,0x740,0x750,0x660,0x360,0x070,0x276,0x077,0x444,0x000,0x000,
                0x777,0x567,0x657,0x757,0x747,0x755,0x764,0x770,0x773,0x572,0x473,0x276,0x467,0x666,0x653,0x760
            ], np.uint16)
        case _:
            # RGB PPU palettes LUT
            # from https://forums.nesdev.org/viewtopic.php?p=98955#p98955
            signal_table_rgb = np.array([
                0x333,0x014,0x006,0x326,0x403,0x503,0x510,0x420,0x320,0x120,0x031,0x040,0x022,0x000,0x000,0x000,
                0x555,0x036,0x027,0x407,0x507,0x704,0x700,0x630,0x430,0x140,0x040,0x053,0x044,0x000,0x000,0x000,
                0x777,0x357,0x447,0x637,0x707,0x737,0x740,0x750,0x660,0x360,0x070,0x276,0x077,0x000,0x000,0x000,
                0x777,0x567,0x657,0x757,0x747,0x755,0x764,0x772,0x773,0x572,0x473,0x276,0x467,0x000,0x000,0x000
            ], np.uint16)

    def rgb_oct_triplet_to_float_array(signal_triplet, emphasis):
        red = ((signal_triplet & 0xF00) >> 8) if not (emphasis & 0b001) else 7
        green = ((signal_triplet & 0x0F0) >> 4) if not (emphasis & 0b010) else 7
        blue = ((signal_triplet & 0x00F)) if not (emphasis & 0b100) else 7
        return np.array([red, green, blue], np.float64)

    # 2C04 LUTs
    scramble = np.array(np.arange(0x00, 0x40), np.uint8)
    match args.ppu:
        case "2C04-0001":
            scramble = np.array([
                0x35,0x23,0x16,0x22,0x1C,0x09,0x1D,0x15,0x20,0x00,0x27,0x05,0x04,0x28,0x08,0x20,
                0x21,0x3E,0x1F,0x29,0x3C,0x32,0x36,0x12,0x3F,0x2B,0x2E,0x1E,0x3D,0x2D,0x24,0x01,
                0x0E,0x31,0x33,0x2A,0x2C,0x0C,0x1B,0x14,0x2E,0x07,0x34,0x06,0x13,0x02,0x26,0x2E,
                0x2E,0x19,0x10,0x0A,0x39,0x03,0x37,0x17,0x0F,0x11,0x0B,0x0D,0x38,0x25,0x18,0x3A
            ], np.uint8)
        case "2C04-0002":
            scramble = np.array([
                0x2E,0x27,0x18,0x39,0x3A,0x25,0x1C,0x31,0x16,0x13,0x38,0x34,0x20,0x23,0x3C,0x0B,
                0x0F,0x21,0x06,0x3D,0x1B,0x29,0x1E,0x22,0x1D,0x24,0x0E,0x2B,0x32,0x08,0x2E,0x03,
                0x04,0x36,0x26,0x33,0x11,0x1F,0x10,0x02,0x14,0x3F,0x00,0x09,0x12,0x2E,0x28,0x20,
                0x3E,0x0D,0x2A,0x17,0x0C,0x01,0x15,0x19,0x2E,0x2C,0x07,0x37,0x35,0x05,0x0A,0x2D
            ], np.uint8)
        case "2C04-0003":
            scramble = np.array([
                0x14,0x25,0x3A,0x10,0x0B,0x20,0x31,0x09,0x01,0x2E,0x36,0x08,0x15,0x3D,0x3E,0x3C,
                0x22,0x1C,0x05,0x12,0x19,0x18,0x17,0x1B,0x00,0x03,0x2E,0x02,0x16,0x06,0x34,0x35,
                0x23,0x0F,0x0E,0x37,0x0D,0x27,0x26,0x20,0x29,0x04,0x21,0x24,0x11,0x2D,0x2E,0x1F,
                0x2C,0x1E,0x39,0x33,0x07,0x2A,0x28,0x1D,0x0A,0x2E,0x32,0x38,0x13,0x2B,0x3F,0x0C
            ], np.uint8)
        case "2C04-0004":
            scramble = np.array([
                0x18,0x03,0x1C,0x28,0x2E,0x35,0x01,0x17,0x10,0x1F,0x2A,0x0E,0x36,0x37,0x0B,0x39,
                0x25,0x1E,0x12,0x34,0x2E,0x1D,0x06,0x26,0x3E,0x1B,0x22,0x19,0x04,0x2E,0x3A,0x21,
                0x05,0x0A,0x07,0x02,0x13,0x14,0x00,0x15,0x0C,0x3D,0x11,0x0F,0x0D,0x38,0x2D,0x24,
                0x33,0x20,0x08,0x16,0x3F,0x2B,0x20,0x3C,0x2E,0x27,0x23,0x31,0x29,0x32,0x2C,0x09
            ], np.uint8)

    for emphasis in range(8):
        for luma in range(4):
            for hue in range(16):
                color_byte = (luma << 4) | hue

                # decode palette LUT to RGB
                YUV_buffer[emphasis, luma, hue] = rgb_oct_triplet_to_float_array(
                    signal_table_rgb[scramble[color_byte]],
                    emphasis
                ) / 7

                # convert to IRE scale so that i don't have to complicate black/white points
                YUV_buffer[emphasis, luma, hue] *= 100

                # apply gain
                YUV_buffer[emphasis, luma, hue] += args.gain

                # encode RGB to YIQ for hue and saturation adjustment
                YUV_buffer[emphasis, luma, hue] = np.matmul(RGB_to_YUV, YUV_buffer[emphasis, luma, hue])

                # Titler functionality
                if (args.ppu == "2C05-99"):
                    # reduce saturation by half
                    YUV_buffer[emphasis, luma, hue, 1] *= 0.5
                    YUV_buffer[emphasis, luma, hue, 2] *= 0.5

                # apply brightness and contrast
                YUV_buffer[emphasis, luma, hue, 0] += args.brightness
                YUV_buffer[emphasis, luma, hue, 0] *= args.contrast

                # apply hue
                YUV_buffer[emphasis, luma, hue, 1] = (
                    (YUV_buffer[emphasis, luma, hue, 1] * np.cos(np.radians(args.hue))) -
                    (YUV_buffer[emphasis, luma, hue, 2] * np.sin(np.radians(args.hue)))
                )
                YUV_buffer[emphasis, luma, hue, 2] = (
                    (YUV_buffer[emphasis, luma, hue, 1] * np.sin(np.radians(args.hue))) +
                    (YUV_buffer[emphasis, luma, hue, 2] * np.cos(np.radians(args.hue)))
                )

                # apply saturation
                YUV_buffer[emphasis, luma, hue, 1] *= args.saturation
                YUV_buffer[emphasis, luma, hue, 2] *= args.saturation

        if not (args.emphasis):
            # clip unused emphasis space
            YUV_buffer = np.split(YUV_buffer, 8, 0)[0]
            break

    return YUV_buffer

def output_binary_uint8(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".pal"), mode="wb") as Palette_file:
        Palette_file.write(np.uint8(np.around(RGB_buffer * 0xFF)))

def output_binary_double(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".pal"), mode="wb") as Palette_file:
        Palette_file.write(RGB_buffer)

def output_gimp_pal(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".gpl"), mode="wt", newline='\n') as Palette_file:
        Palette_file.write("GIMP Palette\n")
        Palette_file.write("Name: generated NES/FC palette\n")
        Palette_file.write(f"Columns: {RGB_buffer.shape[1]}\n")
        Palette_file.write("# https://github.com/Gumball2415/palgen-persune")
        for luma in range(RGB_buffer.shape[0]):
            for hue in range(RGB_buffer.shape[1]):
                color_byte = ((luma & 0x3) << 4) | hue
                emphasis_byte = (luma >> 2) & 0x7
                Palette_file.write(
                    "\n{0:4d}{1:4d}{2:4d} ${3:02X} emphasis {4:03b}".format(
                        np.uint8(np.around(RGB_buffer[luma, hue, 0] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 1] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 2] * 0xFF)),
                        color_byte,
                        emphasis_byte))

def output_png(RGB_buffer, args=None):
    if (args.emphasis):
        sys.exit("error: this format does not support emphasis")
    from PIL import Image
    imgindex = np.arange(0, int(RGB_buffer.size/3), dtype=np.uint8)

    img = Image.frombytes('P', (RGB_buffer.shape[1],RGB_buffer.shape[0]), imgindex)
    # convert to list of uint8

    img.putpalette(list(np.uint8(np.around(RGB_buffer * 0xFF)).ravel()), rawmode="RGB")
    img.save((os.path.splitext(args.output)[0] + ".png"), optimize=True)

def output_jasc_pal(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".pal"), mode="wt", newline='\n') as Palette_file:
        Palette_file.write("JASC-PAL\n0100")
        Palette_file.write(f"\n{int(RGB_buffer.size/3)}")

        # JASC-PAL stops loading when it encounters #000000??
        black_entry_exists = False

        for luma in range(RGB_buffer.shape[0]):
            for hue in range(RGB_buffer.shape[1]):
                if (RGB_buffer[luma, hue, :].all() != 0):
                    Palette_file.write(
                        "\n{0:0d} {1:0d} {2:0d}".format(
                            np.uint8(np.around(RGB_buffer[luma, hue, 0] * 0xFF)),
                            np.uint8(np.around(RGB_buffer[luma, hue, 1] * 0xFF)),
                            np.uint8(np.around(RGB_buffer[luma, hue, 2] * 0xFF))))
                else:
                    black_entry_exists = True
        if black_entry_exists:
            Palette_file.write("\n0 0 0")

def output_html_hex(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".txt"), mode="wt", newline='\n') as Palette_file:
        for luma in range(RGB_buffer.shape[0]):
            for hue in range(RGB_buffer.shape[1]):
                Palette_file.write(
                    "#{0:02X}{1:02X}{2:02X}\n".format(
                        np.uint8(np.around(RGB_buffer[luma, hue, 0] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 1] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 2] * 0xFF))))

def output_mediawiki_table(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".txt"), mode="wt", newline='\n') as Palette_file:
        Palette_file.write("{|class=\"wikitable\"\n")
        for luma in range(RGB_buffer.shape[0]):
            Palette_file.write("|-\n")
            for hue in range(RGB_buffer.shape[1]):
                color_r = int(np.around(RGB_buffer[luma, hue, 0] * 0xFF))
                color_g = int(np.around(RGB_buffer[luma, hue, 1] * 0xFF))
                color_b = int(np.around(RGB_buffer[luma, hue, 2] * 0xFF))
                contrast = 0xFFF if ((color_r*299 + color_g*587 + color_b*114) <= 127500) else 0x000
                color_byte = ((luma & 0x3) << 4) | hue
                Palette_file.write("|style=\"border:0px;background-color:#{0:02X}{1:02X}{2:02X};width:32px;height:32px;color:#{3:03x};text-align:center\"|${4:02X}\n".format(
                    color_r,
                    color_g,
                    color_b,
                    contrast,
                    color_byte))
        Palette_file.write("|}\n")

def output_cstyle_table(RGB_buffer, args=None):
    with open((os.path.splitext(args.output)[0] + ".h"), mode="wt", newline='\n') as Palette_file:
        for luma in range(RGB_buffer.shape[0]):
            for hue in range(RGB_buffer.shape[1]):
                Palette_file.write(
                    "0x{0:02x}, 0x{1:02x}, 0x{2:02x},\n".format(
                        np.uint8(np.around(RGB_buffer[luma, hue, 0] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 1] * 0xFF)),
                        np.uint8(np.around(RGB_buffer[luma, hue, 2] * 0xFF))))
            Palette_file.write("\n")

def main(argv=None):
    args = parse_argv(argv or sys.argv)

    if (args.skip_plot) and (args.output is None) and not (args.render_img is not None):
        sys.exit("warning! palette is generated but not plotted or outputted")

    import colour.models
    import colour.plotting.diagrams

    # colour.RGB_Colourspace helper functions
    def colorspace_equality(s_colorspace, t_colorspace):
        equal = True
        equal = equal and s_colorspace.name == t_colorspace.name
        equal = equal and np.equal(s_colorspace.primaries, t_colorspace.primaries).all()
        equal = equal and np.equal(s_colorspace.whitepoint, t_colorspace.whitepoint).all()
        equal = equal and s_colorspace.whitepoint_name == t_colorspace.whitepoint_name
        equal = equal and np.equal(s_colorspace.matrix_RGB_to_XYZ, t_colorspace.matrix_RGB_to_XYZ).all()
        equal = equal and np.equal(s_colorspace.matrix_XYZ_to_RGB, t_colorspace.matrix_XYZ_to_RGB).all()
        equal = equal and s_colorspace.cctf_decoding_name == t_colorspace.cctf_encoding_name
        return equal

    def init_colorspace(colorspace, primaries_r, primaries_g, primaries_b, primaries_w, opto_electronic=None, electro_optic=None):
        # display color profile colorspace
        x_colorspace = colour.RGB_Colourspace(
            colour.RGB_COLOURSPACES[colorspace].name,
            colour.RGB_COLOURSPACES[colorspace].primaries,
            colour.RGB_COLOURSPACES[colorspace].whitepoint,
            colour.RGB_COLOURSPACES[colorspace].whitepoint_name)

        if (primaries_r is not None
            and primaries_g is not None
            and primaries_b is not None):
            x_colorspace.name = "custom primaries"
            x_colorspace.primaries = np.array([
                primaries_r,
                primaries_g,
                primaries_b
            ])
        else:
            x_colorspace.name = colour.RGB_COLOURSPACES[colorspace].name
            x_colorspace.primaries = colour.RGB_COLOURSPACES[colorspace].primaries

        if (primaries_w is not None):
            x_colorspace.whitepoint = primaries_w
            x_colorspace.whitepoint_name = "custom whitepoint"
        else:
            x_colorspace.whitepoint_name = colour.RGB_COLOURSPACES[colorspace].whitepoint_name
            x_colorspace.whitepoint = colour.RGB_COLOURSPACES[colorspace].whitepoint
        
        if (opto_electronic is not None):
            x_colorspace.cctf_encoding = lambda x: colour.cctf_encoding(x, function=args.opto_electronic)
            x_colorspace.cctf_encoding_name = opto_electronic
        else:
            x_colorspace.cctf_encoding = colour.RGB_COLOURSPACES[colorspace].cctf_encoding
            x_colorspace.cctf_encoding_name = colorspace
        
        if (electro_optic is not None):
            x_colorspace.cctf_decoding = lambda x: colour.cctf_decoding(x, function=args.electro_optic)
            x_colorspace.cctf_decoding_name = electro_optic
        else:
            x_colorspace.cctf_decoding = colour.RGB_COLOURSPACES[colorspace].cctf_decoding
            x_colorspace.cctf_decoding_name = colorspace

        return x_colorspace



    # special thanks to NewRisingSun for teaching me how chromatic adaptations work!
    # special thanks to _aitchFactor for pointing out that colour-science has
    # chromatic adaptation functions!

    # suppress warnings for colour-science
    with colour.utilities.suppress_warnings(*[True] * 4):
        # reference color profile colorspace
        electro_optic = args.electro_optic
        opto_electronic = args.opto_electronic

        if (args.colorimetry_disable):
            s_colorspace = init_colorspace(args.display_colorspace,
                args.display_primaries_r,
                args.display_primaries_g,
                args.display_primaries_b,
                args.display_primaries_w,
                electro_optic=opto_electronic)
        else:
            s_colorspace = init_colorspace(args.reference_colorspace,
                args.reference_primaries_r,
                args.reference_primaries_g,
                args.reference_primaries_b,
                args.reference_primaries_w,
                electro_optic=electro_optic)
    
        t_colorspace = init_colorspace(args.display_colorspace,
            args.display_primaries_r,
            args.display_primaries_g,
            args.display_primaries_b,
            args.display_primaries_w,
            opto_electronic=opto_electronic)
        if (args.debug): print(s_colorspace)
        if (args.debug): print(t_colorspace)

        # decoded RGB buffer
        # has to be zero'd out for the normalize function to work
        RGB_buffer = np.zeros([8,4,16,3], np.float64)
        # signal buffer normalization
        signal_black_point = args.black_point if args.black_point is not None else 0
        signal_white_point = args.white_point if args.white_point is not None else 100

        # generate color!
        match args.ppu:
            case "2C03"|"2C04-0000"|"2C04-0001"|"2C04-0002"|"2C04-0003"|"2C04-0004"|"2C05-99":
                # we use RGB_buffer[] as a temporary buffer for YUV
                RGB_buffer = pixel_codec_rgb(RGB_buffer, args, signal_black_point, signal_white_point)
            case "2C02"|"2C07":
                # different whitepoint calculation
                if (args.white_point is None):
                    signal_white_point = 140 * (composite_white - composite_black)

                RGB_buffer = pixel_codec_composite(RGB_buffer, args, signal_black_point, signal_white_point)

        # reshape buffer after encoding
        if (args.emphasis):
            RGB_buffer = np.reshape(RGB_buffer,(32, 16, 3))
        else:
            RGB_buffer = np.reshape(RGB_buffer,(4, 16, 3))

        # convert back to RGB
        YUV_to_RGB_matrix = {
            "None": YUV_to_RGB,
            "CXA2025AS_JP": YUV_to_RGB_CXA_JP,
            "CXA2025AS_US": YUV_to_RGB_CXA_US,
            "bisqwit_NTSC_1953": YUV_to_RGB_bisqwit
        }

        RGB_buffer = np.einsum('ij,klj->kli', YUV_to_RGB_matrix[args.axis_shift], RGB_buffer, dtype=np.float64)

        # apply black and white points
        # this also scales the values back roughly within range of 0 to 1
        RGB_buffer -= signal_black_point
        RGB_buffer /= (signal_white_point - signal_black_point)

        # fit RGB within range of 0.0-1.0
        normalize_RGB(RGB_buffer, args)

        # preserve uncorrected RGB for color plotting
        RGB_uncorrected = RGB_buffer

        if (not args.colorimetry_disable):
            # convert RGB to display output
            if args.gamma is not None:
                # electro-optic transfer via gamma function
                if (not args.electro_optic_disable):
                    RGB_buffer = colour.gamma_function(RGB_buffer, 2.2)
                    RGB_uncorrected = colour.gamma_function(RGB_uncorrected, 2.2)

                # transform color primaries
                if not colorspace_equality(s_colorspace, t_colorspace):
                    if (args.debug): print("gamma colorspace transform!")
                    if (args.inverse_chromatic_transform):
                        if (args.debug): print("inverse transform!")
                        RGB_buffer = colour.RGB_to_RGB(
                            RGB_buffer,
                            t_colorspace,
                            s_colorspace,
                            chromatic_adaptation_transform=args.chromatic_adaptation_transform)
                        if (args.debug): print(colour.matrix_RGB_to_RGB(t_colorspace, s_colorspace))
                    else:
                        RGB_buffer = colour.RGB_to_RGB(
                            RGB_buffer,
                            s_colorspace,
                            t_colorspace,
                            chromatic_adaptation_transform=args.chromatic_adaptation_transform)
                        if (args.debug): print(colour.matrix_RGB_to_RGB(s_colorspace, t_colorspace))

                # clip to 0.0-1.0 to ensure everything is within range
                np.clip(RGB_buffer, 0, 1, out=RGB_buffer)

                # opto-electronic transfer via gamma function
                if (not args.opto_electronic_disable):
                    RGB_buffer = colour.gamma_function(RGB_buffer, 1/args.gamma)
                    RGB_uncorrected = colour.gamma_function(RGB_uncorrected, 1/args.gamma)

            else:
                if not colorspace_equality(s_colorspace, t_colorspace):
                    # perform transform only when colorspaces actually differ
                    if (args.debug): print("colorspace transform!")
                    if (args.inverse_chromatic_transform):
                        if (args.debug): print("inverse transform!")
                        RGB_buffer = colour.RGB_to_RGB(
                            RGB_buffer,
                            t_colorspace,
                            s_colorspace,
                            chromatic_adaptation_transform=args.chromatic_adaptation_transform,
                            apply_cctf_decoding=(not args.electro_optic_disable),
                            apply_cctf_encoding=(not args.opto_electronic_disable))
                        if (args.debug): print(colour.matrix_RGB_to_RGB(t_colorspace, s_colorspace))
                    else:
                        RGB_buffer = colour.RGB_to_RGB(
                            RGB_buffer,
                            s_colorspace,
                            t_colorspace,
                            chromatic_adaptation_transform=args.chromatic_adaptation_transform,
                            apply_cctf_decoding=(not args.electro_optic_disable),
                            apply_cctf_encoding=(not args.opto_electronic_disable))
                        if (args.debug): print(colour.matrix_RGB_to_RGB(s_colorspace, t_colorspace))

            # clip again, the transform may produce values beyond 0-1
            normalize_RGB(RGB_buffer, args)
            normalize_RGB(RGB_uncorrected, args)

        output_format = {
            ".pal uint8": output_binary_uint8,
            ".pal double": output_binary_double,
            ".pal Jasc": output_jasc_pal,
            ".txt HTML hex": output_html_hex,
            ".txt MediaWiki": output_mediawiki_table,
            ".h uint8_t": output_cstyle_table,
            ".gpl": output_gimp_pal,
            ".png": output_png

        }
        if (args.output is not None and args.render_img is None and args.test_image is None):
            output_format[args.file_format](RGB_buffer, args)

        s_colorspace.name = "Reference colorspace: {}".format(s_colorspace.name)
        t_colorspace.name = "Display colorspace: {}".format(t_colorspace.name)


        # preview on indexed image (if provided)
        # if preview is on and render_img is off, render the indexed image
        if args.test_image is not None and args.output is not None and args.render_img is None:
            with open(args.test_image, mode="rb") as index_file:
                ppubuf = np.reshape(np.frombuffer(index_file.read(), dtype=np.uint16), (240, 256))
                preview_image = np.empty([240,256,3], np.float64)
                mask = 0x1F if args.emphasis else 0x03
                for y in range(ppubuf.shape[0]):
                    for x in range(ppubuf.shape[1]):
                        preview_image[y,x] = RGB_buffer[(ppubuf[y,x] >> 4) & mask, (ppubuf[y,x] & 0x0F)]
                from PIL import Image
                image_filename = os.path.splitext(args.output)[0]
                imageout = Image.fromarray(np.ubyte(np.around(preview_image * 255)))
                imageout.save("{0}.png".format(image_filename))
                imageout.close()

        if (args.render_img is not None):
            for emphasis in range(8):
                palette_plot(RGB_buffer, RGB_uncorrected, emphasis, False, True, True, args, s_colorspace, t_colorspace)
                if not (args.emphasis):
                    break

        palette_plot(RGB_buffer, RGB_uncorrected, 0, True, False, (args.render_img is not None), args, s_colorspace, t_colorspace)

if __name__=='__main__':
    main(sys.argv)
