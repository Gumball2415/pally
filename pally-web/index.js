/* interface for Pally
 * Copyright (C) 2025 Persune
 * under the MIT license.
 *
 * Based on UI code from PalGen,
 * Copyright (C) 2018 DragWx <https://github.com/DragWx>
 * under the MIT license.
 ****/

import init, { generate_palette } from "./pkg/pally_web.js";

var palette = [];	// DOM element + metadata for each entry in the palette
var appwindow;      // DOM element for the app's container
var paletteIndexTextVisible = true;
var saveLink;       // DOM element for the save link

window.onload = init_window;

function init_window() {
    // Get the DOM element for the app's container.
    appwindow = document.getElementById('app');

    // Create the palette display by using a bunch of DIV elements.
    var palettetable = document.createElement('div');
    palettetable.style.width = "100%";
    palettetable.style.aspectRatio = "1 / 0.25";
    palettetable.style.fontFamily = "monospace";
    palettetable.style.fontSize = "8pt";
    palettetable.style.fontWeight = "bold";
    palettetable.style.border = "2px solid var(--palette1)";

    // Create four rows.
    for (var lum = 0; lum < 4; lum++) {
        var newRow = document.createElement('div');
        newRow.style.height = (100 / 4) + "%";

        // For each row, create 16 cells.
        for (var hue = 0; hue < 16; hue++) {
            var paletteEntry = document.createElement('div');
            var paletteIndex = hue + (lum * 16);
            palette[paletteIndex] = paletteEntry;
            paletteEntry.style.display = "inline-block";
            paletteEntry.style.width = (100 / 16) + "%";
            paletteEntry.style.height = "100%";
            paletteEntry.style.background = "#000";
            paletteEntry.style.color = "black";
            if (paletteIndexTextVisible) {
                paletteEntry.style.textAlign = "left";
                paletteEntry.textContent = "$" + paletteIndex.toString(16).padStart(2, '0');
            }
            newRow.appendChild(paletteEntry);
        }
        palettetable.appendChild(newRow);
    }
    appwindow.appendChild(palettetable);

    appwindow.appendChild(document.createElement('br'));

    // Create document.paletteTweaks
    var temp = document.createElement('form');
    temp.name = "paletteTweaks";
    appwindow.appendChild(temp);

    // generation options
    var currPane = document.createElement('fieldset');
    temp = document.createElement('legend');
    temp.innerHTML = "Generation options";
    currPane.appendChild(temp);
    document.paletteTweaks.appendChild(currPane);

    {
        // PPU type
        var childPane = document.createElement('fieldset');
        var childTemp = document.createElement('legend');
        childTemp.title = "Select PPU variant of which to generate video from";
        childTemp.innerHTML = "PPU type";
        childPane.appendChild(childTemp);
        currPane.appendChild(childPane);

        // Macro for creating a radio input.
        var newRadioOption = function (pane, groupName, value, selected, title) {
            var radio = document.createElement('input');
            radio.type = "radio";
            radio.name = groupName;
            radio.id = groupName + value;
            radio.value = value;
            if (selected === true)
                radio.checked = "checked";
            radio.onclick = generatePalette;
            pane.appendChild(radio);
            var label = document.createElement('label');
            label.id = groupName;
            label.innerHTML = value;
            label.htmlFor = radio.id;
            label.title = title;
            pane.appendChild(label);
        }

        // must match mapping in palgen_persune!
        newRadioOption(childPane, "ppuType", "2C02", true,
            "RP2C02, NTSC composite PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C03", false,
            "RP2C03, RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C04-0000", false,
            "RP2C04-0000, Scrambled palette RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C04-0001", false,
            "RP2C04-0001, Scrambled palette RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C04-0002", false,
            "RP2C04-0002, Scrambled palette RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C04-0003", false,
            "RP2C04-0003, Scrambled palette RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C04-0004", false,
            "RP2C04-0004, Scrambled palette RGB video PPU");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C05-99", false,
            "RC2C05-99, RGB video PPU (with Famicom Titler composite encoder)");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "ppuType", "2C07", false,
            "RP2C07, PAL-B composite video PPU");

        // Normalize
        childPane = document.createElement('fieldset');
        var childLegend = document.createElement('legend');
        childLegend.title = "Normalize all colors within gamut by scaling values";

        // Toggle box
        childTemp = document.createElement('input');
        childTemp.type = "checkbox";
        childTemp.id = "enableNormalizeMethod";
        childLegend.appendChild(childTemp);
        childTemp = document.createElement("label");
        childTemp.innerHTML = "Normalize";
        childTemp.htmlFor = "enableNormalizeMethod";
        childLegend.appendChild(childTemp);
        childTemp = childLegend;
        childPane.appendChild(childTemp);
        currPane.appendChild(childPane);


        // must match mapping in pally!
        newRadioOption(childPane, "normalizeMethod", "Scale", true,
            "Scales values within 0-1");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "normalizeMethod", "ScaleClipNegative", false,
            "Clips negative values to 0, then scales values within 0-1");

        document.paletteTweaks.enableNormalizeMethod.addEventListener(
            "change",
            (event) => {
                document.getElementById("normalizeMethodScale").disabled = !event.target.checked;
                document.getElementById("normalizeMethodScaleClipNegative").disabled = !event.target.checked;
            },
            false,
        );
        document.paletteTweaks.enableNormalizeMethod.onchange = generatePalette;

        // Clip
        childPane = document.createElement('fieldset');
        childLegend = document.createElement('legend');
        childLegend.title = "Clips out-of-gamut RGB colors";

        // Toggle box
        childTemp = document.createElement('input');
        childTemp.type = "checkbox";
        childTemp.id = "enableClipMethod";
        childLegend.appendChild(childTemp);
        childTemp = document.createElement("label");
        childTemp.innerHTML = "Clip";
        childTemp.htmlFor = "enableClipMethod";
        childLegend.appendChild(childTemp);
        childTemp = childLegend;
        childPane.appendChild(childTemp);
        currPane.appendChild(childPane);


        // must match mapping in palgen_persune!
        newRadioOption(childPane, "clipMethod", "Darken", true,
            "If any channels are out of range, the color is darkened until it is completely in range. Algorithm by DragWx.");

        childPane.appendChild(document.createElement('br'));
        newRadioOption(childPane, "clipMethod", "Desaturate", false,
            "If any channels are out of range, the color is desaturated towards its luminance. Algorithm by DragWx.");

        document.paletteTweaks.enableClipMethod.addEventListener(
            "change",
            (event) => {
                document.getElementById("clipMethodDarken").disabled = !event.target.checked;
                document.getElementById("clipMethodDesaturate").disabled = !event.target.checked;
            },
            false,
        );
        document.paletteTweaks.enableClipMethod.onchange = generatePalette;
    }

    // color decoding options
    currPane = document.createElement('fieldset');
    temp = document.createElement('legend');
    temp.innerHTML = "Color decoding options";
    currPane.appendChild(temp);
    document.paletteTweaks.appendChild(currPane);

    currPane.appendChild(makeFancyRangeBox("bri"));
    currPane.appendChild(document.createTextNode("Brightness"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("con"));
    currPane.appendChild(document.createTextNode("Contrast"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("hue"));
    currPane.appendChild(document.createTextNode("Hue"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("sat"));
    currPane.appendChild(document.createTextNode("Saturation"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("gai"));
    currPane.appendChild(document.createTextNode("Gain"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("blp"));
    currPane.appendChild(document.createTextNode("Black point (IRE)"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("whp"));
    currPane.appendChild(document.createTextNode("White point (IRE)"));

    // analog effects options
    currPane = document.createElement('fieldset');
    temp = document.createElement('legend');
    temp.innerHTML = "Analog effects options";
    currPane.appendChild(temp);
    document.paletteTweaks.appendChild(currPane);

    currPane.appendChild(makeFancyRangeBox("phd"));
    currPane.appendChild(document.createTextNode("Differential phase distortion"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("aps"));
    currPane.appendChild(document.createTextNode("Anti-emphasis phase skew"));
    currPane.appendChild(document.createElement("br"));

    currPane.appendChild(makeFancyRangeBox("ela"));
    currPane.appendChild(document.createTextNode("Emphasis luma attenuation"));

    // default:
    document.paletteTweaks.bri.value = "0.0";
    document.paletteTweaks.con.value = "1.0";
    document.paletteTweaks.hue.value = "0.0";
    document.paletteTweaks.sat.value = "1.0";
    document.paletteTweaks.gai.value = "0.0";
    document.paletteTweaks.phd.value = "4.0";
    document.paletteTweaks.aps.value = "0.0";
    document.paletteTweaks.ela.value = "0.0";
    document.paletteTweaks.blp.value = "0.0";
    document.paletteTweaks.whp.value = "100.0";
    // Apply this after the defaults to prevent spurious calls.
    document.paletteTweaks.hue.onchange = generatePalette;
    document.paletteTweaks.sat.onchange = generatePalette;
    document.paletteTweaks.bri.onchange = generatePalette;
    document.paletteTweaks.con.onchange = generatePalette;
    document.paletteTweaks.gai.onchange = generatePalette;
    document.paletteTweaks.phd.onchange = generatePalette;
    document.paletteTweaks.aps.onchange = generatePalette;
    document.paletteTweaks.ela.onchange = generatePalette;
    document.paletteTweaks.blp.onchange = generatePalette;
    document.paletteTweaks.whp.onchange = generatePalette;



    // Palette options
    currPane = document.createElement('fieldset');
    temp = document.createElement('legend');
    temp.innerHTML = "Palette options";
    currPane.appendChild(temp);
    document.paletteTweaks.appendChild(currPane);


    // Red emphasis
    childTemp = document.createElement('input');
    childTemp.type = "checkbox";
    childTemp.id = "viewRedEmphasis";
    currPane.appendChild(childTemp);
    childTemp = document.createElement("label");
    childTemp.innerHTML = "Enable R emphasis";
    childTemp.htmlFor = "viewRedEmphasis";
    currPane.appendChild(childTemp);

    // Green emphasis
    currPane.appendChild(document.createElement("br"));
    childTemp = document.createElement('input');
    childTemp.type = "checkbox";
    childTemp.id = "viewGreenEmphasis";
    currPane.appendChild(childTemp);
    childTemp = document.createElement("label");
    childTemp.innerHTML = "Enable G emphasis";
    childTemp.htmlFor = "viewGreenEmphasis";
    currPane.appendChild(childTemp);

    // Blue emphasis
    currPane.appendChild(document.createElement("br"));
    childTemp = document.createElement('input');
    childTemp.type = "checkbox";
    childTemp.id = "viewBlueEmphasis";
    currPane.appendChild(childTemp);
    childTemp = document.createElement("label");
    childTemp.innerHTML = "Enable B emphasis";
    childTemp.htmlFor = "viewBlueEmphasis";
    currPane.appendChild(childTemp);

    // Blue emphasis
    currPane.appendChild(document.createElement("br"));
    childTemp = document.createElement('input');
    childTemp.type = "checkbox";
    childTemp.id = "saveEmphasis";
    currPane.appendChild(childTemp);
    childTemp = document.createElement("label");
    childTemp.innerHTML = "Save emphasis entires";
    childTemp.htmlFor = "saveEmphasis";
    currPane.appendChild(childTemp);

    document.paletteTweaks.viewRedEmphasis.onchange = generatePalette;
    document.paletteTweaks.viewGreenEmphasis.onchange = generatePalette;
    document.paletteTweaks.viewBlueEmphasis.onchange = generatePalette;
    document.paletteTweaks.saveEmphasis.onchange = generatePalette;

    // colorimetry options
    // colorimetry reference RGB and whitepoint primaries
    // colorimetry display RGB and whitepoint primaries
    appwindow.appendChild(document.createElement('br'));
    saveLink = document.createElement('a');
    saveLink.style.display = "none";
    //saveLink.innerHTML = "[Save palette...]";
    saveLink.href = "";
    saveLink.download = "pally.pal";
    appwindow.appendChild(saveLink);

    temp = document.createElement("button");
    temp.innerHTML = "Save palette..."
    temp.onclick = function () {
        saveLink.click();
    };
    appwindow.appendChild(temp);

    generatePalette();
}

// Create an input box with [-] and [+] buttons.
// buttonStep is actually fixed point (1000 = 1.0f) to avoid rounding shenanigans.
function makeFancyRangeBox(name, buttonStep) {
    if (buttonStep === undefined)
        buttonStep = 50;
    var temp = document.createElement('div');
    temp.style.display = "inline-flex";
    temp.style.flexDirection = "row";

    var valueBox = document.createElement('input');
    valueBox.type = "number";
    valueBox.id = name;
    valueBox.name = name;
    valueBox.size = "5";
    valueBox.step = "0.0001";
    valueBox.style.width = "5em";

    var minusButton = document.createElement('input');
    minusButton.type = "button";
    minusButton.value = "-";
    minusButton.onclick = function () {
        valueBox.value = (Math.round(parseFloat(valueBox.value) * 1000) - buttonStep) / 1000;
        valueBox.onchange();
    }
    minusButton.style.width = "32px";

    var plusButton = document.createElement('input');
    plusButton.type = "button";
    plusButton.value = "+";
    plusButton.onclick = function () {
        valueBox.value = (Math.round(parseFloat(valueBox.value) * 1000) + buttonStep) / 1000;
        valueBox.onchange();
    }
    plusButton.style.width = "32px";

    temp.appendChild(minusButton);
    temp.appendChild(valueBox);
    temp.appendChild(plusButton);
    return temp;
}

function toColorHex(i) {
    if (i < 16)
        i = "0" + i.toString(16);
    else
        i = i.toString(16);
    return i;
}

async function generatePalette() {
    const settings = document.paletteTweaks;
    var renderEmphasis = settings.saveEmphasis.checked
        || settings.viewRedEmphasis.checked
        || settings.viewGreenEmphasis.checked
        || settings.viewBlueEmphasis.checked;

    var emphasisIndex = (settings.viewRedEmphasis.checked
        | settings.viewGreenEmphasis.checked << 1
        | settings.viewBlueEmphasis.checked << 2) * 64 * 3;

    var saveEmphIndex = emphasisIndex;
    var paletteSize = 64 * 3;
    if (settings.saveEmphasis.checked) {
        saveEmphIndex = 0;
        paletteSize *= 8;
    }

    var clip;
    if (settings.enableClipMethod.checked) {
        for (i = 0; i < settings.clipMethod.length; i++) {
            if (settings.clipMethod[i].checked) {
                clip = settings.clipMethod[i].value;
                break;
            }
        }
    } else {
        clip = "";
    }

    var normalize;
    if (settings.enableNormalizeMethod.checked) {
        for (i = 0; i < settings.normalizeMethod.length; i++) {
            if (settings.normalizeMethod[i].checked) {
                normalize = settings.normalizeMethod[i].value;
                break;
            }
        }
    }
    else {
        normalize = "";
    }

    var blackPoint = settings.blp.value;
    if (blackPoint === "") { blackPoint = null; }
    else { blackPoint = parseFloat(blackPoint) }
    var whitePoint = settings.whp.value;
    if (whitePoint === "") { whitePoint = null; }
    else { whitePoint = parseFloat(whitePoint) }

    var brightness = parseFloat(settings.bri.value);
    var contrast = parseFloat(settings.con.value);
    var hue = parseFloat(settings.hue.value);
    var saturation = parseFloat(settings.sat.value);
    var gain = parseFloat(settings.gai.value);
    var gamma = null; //parseFloat(settings.gamma.value)
    var ppu;

    for (var i = 0; i < settings.ppuType.length; i++) {
        if (settings.ppuType[i].checked) {
            ppu = settings.ppuType[i].value;
            break;
        }
    }

    var phaseDistortion = parseFloat(settings.phd.value);

    var textEnable = false;
    // if (settings.showtext.checked) textEnable = true;

    await init();
    const outbuf = generate_palette(
        renderEmphasis,
        clip,
        normalize,
        blackPoint,
        whitePoint,
        brightness,
        contrast,
        hue,
        saturation,
        gain,
        gamma,
        ppu,
        phaseDistortion
    );

    // Build the binary version of the palette for download

    var binPal = "";
    var j = 0;
    for (var i = 0; i < paletteSize; i) {
        var r = outbuf[saveEmphIndex + i++];
        var g = outbuf[saveEmphIndex + i++];
        var b = outbuf[saveEmphIndex + i++];

        r = toColorHex(r);
        g = toColorHex(g);
        b = toColorHex(b);

        binPal += "%" + r;
        binPal += "%" + g;
        binPal += "%" + b;
    }
    // As well as the preview
    var j = 0;
    for (var i = 0; i < 64 * 3; i) {
        var r = outbuf[emphasisIndex + i++];
        var g = outbuf[emphasisIndex + i++];
        var b = outbuf[emphasisIndex + i++];

        var lum = r * 299 + g * 587 + b * 114;

        r = toColorHex(r);
        g = toColorHex(g);
        b = toColorHex(b);

        var textColor = "#FFFFFF";

        if (lum > 127500) {
            palette[j].style.color = "black";
        }
        else {
            palette[j].style.color = "white";
        }

        palette[j++].style.background = "#" + r + g + b;
    }
    saveLink.href = "data:application/octet-stream," + binPal;
}