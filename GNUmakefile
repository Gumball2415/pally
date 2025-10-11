#!/usr/bin/make -f
# it's recommended to set up a python .venv that installs requirements.txt
# and then run this makefile in that venv

examples_dir := docs/example_palettes
examples_wiki_dir := docs/NESDev
diagrams_dir := docs/diagrams

# taken from NROM template
ifeq (${OS}, Windows_NT)
	EXE_SUFFIX := .exe
	PY := py -3
else
	EXE_SUFFIX :=
	PY := python3
endif

.PHONY: example_palettes example_NESDev diagrams all

all: example_palettes example_NESDev diagrams

clean:
	${RM} -r ${examples_dir} ${examples_wiki_dir} ${diagrams_dir}

example_palettes: ${examples_dir}\
	${examples_dir}/2C02_default.pal\
	${examples_dir}/savtool_replica.pal\
	${examples_dir}/2C02-2C07_aps_ela_persune_neutral.pal\
	${examples_dir}/2C02G_phd.pal\
	${examples_dir}/2C02G_aps_ela_NTSC_persune_tink.pal\
	${examples_dir}/2C02G_aps_ela_NTSC_persune_GVUSB2_NTSC_M_J.pal\
	${examples_dir}/2C02G_aps_ela_NTSC_persune_GVUSB2_NTSC_M.pal\
	${examples_dir}/2C05-99.pal\
	${examples_dir}/2C02G_phs_aps_ela_NTSC.pal\
	${examples_dir}/2C02G_phs_aps_ela_NTSC-1953.pal\
	${examples_dir}/2C02G_phs_aps_ela_NTSC-J.pal\
	${examples_dir}/2C07_phs_aps_ela_PAL.pal\
	${examples_dir}/2C03_DeMarsh_1980s_RGB.pal

# example palettes with different settings

${examples_dir}:
	mkdir $@ -p

# no special options
${examples_dir}/2C02_default.pal:
	${PY} pally.py --skip-plot -e -o $@

# replicating Bisqwit's savtool palette
${examples_dir}/savtool_replica.pal:
	${PY} pally.py --skip-plot -e -hue -15 -sat 0.65 -gam 2.25 -spg -rfc "NTSC (1953)" -rpw 0.3127 0.329 -o $@

# somewhere between NTSC and PAL hue for a "neutral" compromise
${examples_dir}/2C02-2C07_aps_ela_persune_neutral.pal:
	${PY} pally.py --skip-plot -e -hue -3.75 -bse -aps 5 -ela 0.01429 -cld -o $@

# based on measurements of my own composite decoders
${examples_dir}/2C02G_aps_ela_NTSC_persune_tink.pal:
	${PY} pally.py --skip-plot -e -hue 2.5 -bse -aps 5 -ela 0.01429 -cld -o $@

${examples_dir}/2C02G_aps_ela_NTSC_persune_GVUSB2_NTSC_M_J.pal:
	${PY} pally.py --skip-plot -e -hue 12 -sat 0.8 -aps 5 -ela 0.01429 -cld -phd 3 -o $@

${examples_dir}/2C02G_aps_ela_NTSC_persune_GVUSB2_NTSC_M.pal:
	${PY} pally.py --skip-plot -e -hue 12 -sat 0.8 -gai -6.5 -blp 6 -aps 5 -ela 0.01429 -cld -phd 3 -o $@

# Titler palette
${examples_dir}/2C05-99.pal:
	${PY} pally.py --skip-plot -ppu "2C05-99" -e -cld -o  $@

# NTSC standard
${examples_dir}/2C02G_phs_aps_ela_NTSC.pal:
	${PY} pally.py --skip-plot -e -blp 7.5 -aps 5 -ela 0.01429 -phd 3 -o $@

${examples_dir}/2C02G_phs_aps_ela_NTSC-1953.pal:
	${PY} pally.py --skip-plot -e -blp 7.5 -rfc "NTSC (1953)" -aps 5 -ela 0.01429 -phd 3 -o $@

${examples_dir}/2C02G_phs_aps_ela_NTSC-J.pal:
	${PY} pally.py --skip-plot -e -rpr 0.618 0.350 -rpg 0.280 0.605 -rpb 0.152 0.063 -rpw 0.28314501 0.29711289 -aps 5 -ela 0.01429 -phd 3 -o $@
# PAL standard
${examples_dir}/2C07_phs_aps_ela_PAL.pal:
	${PY} pally.py --skip-plot -ppu "2C07" -e -blp 7.5 --delay-line-filter -aps 5 -ela 0.01429 -phd 4 -cld -o $@

# RGB with DeMarsh primaries
${examples_dir}/2C03_DeMarsh_1980s_RGB.pal:
	${PY} pally.py --skip-plot -ppu "2C03" -e -rpr 0.622 0.338 -rpg 0.343 0.590 -rpb 0.153 0.059 -rpw 0.28314501 0.29711289 -o $@

# 2C02 with true differential phase distortion example
# $0x-$3x hue deviation = 13.9979287408074
# based on measurements from this post
# https://forums.nesdev.org/viewtopic.php?p=186297#p186297
${examples_dir}/2C02G_phd.pal:
	${PY} pally.py --skip-plot -e -phd 3 -o $@

# NESDev wiki palettes

example_NESDev: ${examples_wiki_dir}\
	${examples_wiki_dir}/2C02G_wiki_palette_page.txt\
	${examples_wiki_dir}/2C02G_wiki.txt\
	${examples_wiki_dir}/2C02G_wiki.pal\
	${examples_wiki_dir}/2C07_wiki_palette_page.txt\
	${examples_wiki_dir}/2C07_wiki.txt\
	${examples_wiki_dir}/2C07_wiki.pal\
	${examples_wiki_dir}/2C03_wiki_palette_page.txt\
	${examples_wiki_dir}/2C03_wiki.txt\
	${examples_wiki_dir}/2C03_wiki.pal


${examples_wiki_dir}:
	mkdir $@ -p

# 2C02G with phase skew of approx. -5 degrees
${examples_wiki_dir}/2C02G_wiki_palette_page.txt:
	${PY} pally.py --skip-plot -cld -phd 4 -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C02G_wiki.txt:
	${PY} pally.py --skip-plot -cld -phd 4 -e -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C02G_wiki.pal:
	${PY} pally.py --skip-plot -cld -phd 4 -e -o $@

#2C07 with phase skew of -5 degrees and delay line filtering
${examples_wiki_dir}/2C07_wiki_palette_page.txt:
	${PY} pally.py --skip-plot -cld -ppu "2C07" -phd 4 --delay-line-filter -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C07_wiki.txt:
	${PY} pally.py --skip-plot -cld -ppu "2C07" -phd 4 --delay-line-filter -e -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C07_wiki.pal:
	${PY} pally.py --skip-plot -cld -ppu "2C07" -phd 4 --delay-line-filter -e -o $@

#2C03
${examples_wiki_dir}/2C03_wiki_palette_page.txt:
	${PY} pally.py --skip-plot -cld -ppu "2C03" -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C03_wiki.txt:
	${PY} pally.py --skip-plot -cld -ppu "2C03" -e -o $@ -f ".txt MediaWiki"

${examples_wiki_dir}/2C03_wiki.pal:
	${PY} pally.py --skip-plot -cld -ppu "2C03" -e -o $@

#TODO: figure this out
# pally.py --skip-plot -cld -hue -15 -o docs/NESDev/Composite_wiki_palette_page -f ".txt MediaWiki"
# pally.py --skip-plot -cld -hue -15 -e -o docs/NESDev/Composite_wiki -f ".txt MediaWiki"
# pally.py --skip-plot -cld -hue -15 -e -o docs/NESDev/Composite_wiki

# diagrams

diagrams: ${diagrams_dir}\
	usage.txt\
	${diagrams_dir}/addie.png\
	${diagrams_dir}/minae.png\
	${diagrams_dir}/palette_preview_emphasis.gif
	${PY} pally.py --skip-plot -p -w -r png -o ${diagrams_dir} -phd 4
	${PY} pally.py --skip-plot -r png -t docs/smb.bin -o ${diagrams_dir}
	ffmpeg -framerate 2 -i "${diagrams_dir}/QAM_phase_%03d.png" -filter_complex "split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" "${diagrams_dir}/QAM_phase.gif" -y
	ffmpeg -framerate 2 -i "${diagrams_dir}/waveform_phase_%03d.png" -filter_complex "split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" "${diagrams_dir}/waveform_phase.gif" -y

${diagrams_dir}:
	mkdir $@ -p

usage.txt:
	${RM} -r $@
	${PY} pally.py -h >> $@

${diagrams_dir}/addie.png:
	${PY} pally.py --skip-plot -t docs/addie.bin -phd 4 -o $@

${diagrams_dir}/minae.png:
	${PY} pally.py --skip-plot -ppu "2C05-99" -t docs/minae.bin -o $@

${diagrams_dir}/palette_preview_emphasis.gif:
	${PY} pally.py --skip-plot -e -r png -o ${diagrams_dir}
	ffmpeg -framerate 2 -i "${diagrams_dir}/palette_preview_emphasis_%03d.png" -filter_complex "split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" "$@" -y
