# Varif locked palette

**Version 2.1** — 2026-11-21 (America/Chicago). 91 colours: the 79 v2 colours, unchanged, plus **12 new** measured colours. Same three modes: **core / shared** (42), **night / celestia** (5), **daylight / landscape** (44). 35 named ramps (24 from v2, 11 new). This is the production set: do not add a colour without a ramp review.

Companion files: `palette-v2.1.hex`, `palette-v2.1.gpl` (GIMP), `palette-v2.1.aco` (Photoshop swatch, versions 1 and 2), `palette-v2.1.ase` (Adobe Swatch Exchange, grouped by ramp), `palette-v2.1.png` (swatch sheet; v2.1 ramps at the bottom of each mode with new colours framed in orange).

v2 stays beside it, untouched, as `palette.md`, `palette.hex`, `palette.gpl`, `palette.aco`, `palette.ase`, `palette.png`. v1 is archived as `palette-v1.*`.

## What v2.1 adds

Four planned families, each measured from its reference with a script:

| Family | Ramps | New colours | Measured from |
| --- | --- | ---: | --- |
| Price Line (purples, blues, whites) | Price Line | 3 | Emanda Price portrait (`refs/emanda-price.png`) |
| Ptormyk (yellow-green grass, purplish rock, violet snow peaks, lake) | Ptormyk grass, violet rock, snow peaks, lake | 5 | `ptormyk_mountain_valley.jpg` |
| Niaviopti (white gold, pearl pommel, faint orange glow) | Niaviopti | 1 | `niaviopti.jpg` (the full-detail sword painting) |
| Southern Maphlia / Maphyk Hylvanne terrain and old Yhlgori architecture | Old Yhlgori sandstone, spire gold; Maphlia golden grass, tan granite; Hylvanne lakes | 3 | `hylvynhani.jpg`, `hylvynhani_upper_city.jpg`, `hylvynhani_falls.jpg`, `klate.jpg` |

35 ramp steps were measured (including two Price collar steps that are documented but not placed in a ramp). 23 of them landed within OKLab 0.03 of an existing swatch and reuse it, so v2.1 adds 12 colours rather than 35. Every new colour is at least 0.032 from every other colour in the set.

The Audsen Line stays fully reachable: no v2 colour was changed or removed. Audsen maroon is Cloth reds (Cloak Shade `#381B12`, Cloak Mid `#A13E18`, Cloak Bright `#CC1D19`); Audsen gold is the Gold / bronze ramp. Price purple sits clear of Audsen maroon: Price Plum is 0.041 from Cloak Shade in OKLab, 42° apart in hue.

## Why 91 colours

v2 capped the set at 80 and said crossing it would need "a second accent family that the current photos do not yet support." The v2.1 references now supply those families: a Line colour (Price purple), a regional design language (Ptormyk violet and yellow), a mythic item (pearl), and the old Yhlgori builds. The recommended cap is raised to **96**. Most of what the new paintings show was already in v2 (the southern Maphlia golden grass and tan granite add nothing new), so the dedupe kept the increase to 12.

## Day and night

v2.1 keeps the v2 mode split. The new terrain and architecture ramps are **daylight / landscape**. Price and Niaviopti are **core / shared**, used in every lighting mode, like the other cloth and metal ramps. The night / celestia mode is unchanged (blood red sky only). At night, follow the v2 rules: far land goes to Void Ink, Cliff Teal, or Ember Void; torch-lit Yhlgori sandstone uses the Torchlight ramp; lakes go dark and take Star Red reflections. The Niaviopti pommel glow is the one v2.1 accent that reads strongest at night.

## Method (v2.1)

Script: `analysis/extract_v2_1.py` (helpers in `analysis/oklab.py`). Output: `analysis/measured-colors-v2.1.json`, `analysis/extraction-report-v2.1.txt`. Export: `analysis/build_palette_v2_1.py` (writes `analysis/palette-spec-v2.1.json` and the `palette-v2.1.*` files). This document: `analysis/write_palette_md_v2_1.py`.

1. Each image is resized so its long edge is **480 px** (LANCZOS), as in v2, and converted to **OKLab**.
2. Each material is a documented **mask**: a crop (fractions of the image) plus an OKLCh filter (hue range, chroma, lightness). The masks and pixel counts are listed under each ramp.
3. **Even OKLab lightness steps.** Band centres are spaced evenly between the 10th and 90th lightness percentiles of the masked pixels. Each step is the OKLab mean of the masked pixels within ±0.025 L of its centre, so every step is measured, not interpolated.
4. **Dedupe.** A step within **0.03** (OKLab Euclidean) of an existing swatch, and within 20° of its hue (hue ignored when chroma < 0.03), reuses that swatch. Larger samples are processed first, so a small sample (the sword) reuses a swatch from a large one (the cities).
5. For context, each image also has an overall OKLab k-means (k = 12) in the extraction report.

This differs from v2 (RGB MiniBatchKMeans centroids). The v2 colours were not re-measured. All 12 new colours are measured (M); none are adjusted.

**Hue shift.** The new ramps follow the v2 habit of shifting hue with light rather than only darkening. Sandstone warms toward yellow as it lightens (66.7° → 69.3° → 68.7° → 74.7°); spire gold 72.7° → 86.7°; Ptormyk grass 103.7° → 88.8° → 83.3° (greener in shade, golden in sun); Ptormyk rock 272.3° → 281.2° → 292.4° (bluer in shade, violet in light); Price plum 347.9° → 355.9° → 2.5° (toward red in light).

## Loading and updating

- **Photoshop.** Window → Swatches, panel menu → Load Swatches… (or Replace Swatches…), and choose `palette-v2.1.aco` or `palette-v2.1.ase`. The `.ase` keeps the ramp groups as `mode / ramp`. When indexing a finished sheet (Image → Mode → Indexed Color, Custom, Dither None), the colour table is the 91 v2.1 colours.
- **GIMP / Aseprite.** Open `palette-v2.1.gpl` (Aseprite also reads the `.ase`).
- **New references.** For a whole-image look-dev pass, follow the v2 steps in `palette.md`. For a v2.1-style material ramp, add a mask to `MATERIALS` in `analysis/extract_v2_1.py`, run it, name any NEW step in `analysis/build_palette_v2_1.py`, then run `build_palette_v2_1.py` and `write_palette_md_v2_1.py`. None of these touch the v2 or v1 files.

## Changelog

### v2.1 — 2026-11-21

- Added 12 measured colours in 11 new ramps: Price Line, Niaviopti (core); old Yhlgori sandstone and spire gold, southern Maphlia golden grass, Maphyk Hylvanne tan granite and lakes, Ptormyk grass, violet rock, snow peaks, and lake (daylight). Total **91**.
- Measured in OKLab with documented masks and even lightness steps; 23 of 35 measured steps reuse existing swatches.
- Raised the recommended cap from 80 to 96.
- No v2 colour changed or removed. v2 files left as they were.

### v2 — 2026-11-15

- Measured twenty new landscape references with the same k-means method, plus a hue-family breakdown (greens, oranges/golds, sky blues, greys).
- Restructured the palette into **core / shared** (38), **night / celestia** (5), and **daylight / landscape** (36). Total **79** (74 measured, 5 adjusted).
- Kept every v1 night-sky, torchlight, gold, caste, cloth, linen, eye, warm-stone, Cliff Teal, and Cold Ashlar colour.
- Added day-sky, atmospheric-haze, conifer, meadow, straw, sunset-orange, granite, limestone-extension, snow, water, and fireweed ramps.
- Retired the four-colour v1 foliage / autumn ramp and merged Mountain Grey into Granite Shade. See [Retired and merged](#retired-and-merged).
- Extraction and export are now manifest- and spec-driven: `analysis/extract_colors.py`, `analysis/refs-manifest.json`, `analysis/palette-spec.json`, `analysis/build_palette.py`.
- Re-exported `palette.hex`, `.gpl`, `.aco`, `.ase`, and `palette.png` (swatches grouped by mode). v1 files kept as `palette-v1.*`.
- Style guide updated: pillars, atmospheric-perspective rule, time-of-day section, biome notes, expanded reference list.

### v1 — 2026-11-15

- First locked set: 48 colours, 14 ramps, 43 measured / 5 adjusted, from the eleven painted concept references. Archived as `palette-v1.*`.

## Ramps added in v2.1

Each table lists the full ramp in lightness order. **NEW** = added in v2.1. **shared** = the measured step matched an existing swatch (ΔE is the OKLab distance from the measured mean). **assigned** = shared by design, not measured from this reference.

### Mode: Core / shared — 4 new colours

#### Price Line purples, blues, whites — 7 steps, 3 new

Price Line leans purples, blues, and whites. The purples are measured from the Price portrait's plum cloak (bottom quarter, 11,464 px, 19.9% of that crop). The portrait has no blue: 0 of 230,400 px fall in OKLCh hue 220°–290° at chroma ≥ 0.03. So the blues are assigned from the new Ptormyk lake pair and the whites from the existing cool whites. The white collar itself measures warm grey (#95816F at L 0.618 and #AFA38D at L 0.720, 1,488 px), within 0.015 and 0.012 of Granite Lit and Warm Silver; use Warm Silver for a shaded Price collar.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Price Plum Shade | `#1E0C16` | **NEW** · M | `#1E0C16` (0.186) | Price Line purple cloth in deep shadow (velvet folds). |
| 2 | Price Plum | `#351422` | **NEW** · M | `#351422` (0.247) | Default Price Line purple: cloaks, sashes, banner field. |
| 3 | Price Plum Lit | `#4D1D2C` | **NEW** · M | `#4D1D2C` (0.307) | Lit edge of Price Line purple cloth. |
| 4 | Ptormyk Lake Deep | `#17346E` | assigned · v2.1 | — | Price blue: cloth, banner field, trim (assigned; the Price portrait has no blue). |
| 5 | Ptormyk Lake | `#4779B4` | assigned · v2.1 | — | Price light blue: lit blue cloth, banner charge. |
| 6 | Pale Silver | `#BDCECC` | assigned · v2 | — | Price cool white: lace, linen highlight, banner ground. |
| 7 | Ivory | `#EBEADC` | assigned · v2 | — | Price white in light, paper, parchment. |

Mask: emanda_price crop [0, 0.75, 1, 1] hue 300–15° chroma 0.03–1 L 0–1: 11,464 px (19.9% of crop).

#### Niaviopti white gold, pearl, glow — 9 steps, 1 new

Mythic sword. White gold measured from the hilt, guard and pommel cap (1,742 px); the pearl from the pommel (307 px). The source painting has no orange: the faint pommel glow is a game-design addition and uses v2 swatches: Flame Mid and Sunset Mid, as in the shipped mythic icons, plus Sunset Orange as an optional hotter pixel. Only Pearl White is new.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Straw Shade | `#513F29` | shared · v2 (ΔE 0.021) | `#4B3925` (0.361) | White-gold hilt in shadow. |
| 2 | Yhlgori Sandstone | `#7C6954` | shared · v2.1 (ΔE 0.008) | `#7F6B52` (0.539) | White-gold hilt mid (shared with Yhlgori Sandstone). |
| 3 | Warm Silver | `#A7A391` | shared · v2 (ΔE 0.021) | `#B4A187` (0.718) | Lit white gold on the guard and pommel cap. |
| 4 | Lime Lit | `#BFA991` | shared · v2 (ΔE 0.027) | `#B4ACA4` (0.749) | Pearl body in shade, warm grey. |
| 5 | Silver Highlight | `#D0D8D3` | shared · v2 (ΔE 0.014) | `#D9D4CF` (0.873) | Pearl mid, cool sheen. |
| 6 | Pearl White | `#FEFEFD` | **NEW** · M | `#FEFEFD` (0.996) | Pearl specular on the Niaviopti pommel; the only pure white in the set. One or two pixels only. |
| 7 | Flame Mid | `#803518` | assigned · v2 | — | Pommel glow ring and the sparse outer flicker (as in the shipped mythic icons). |
| 8 | Sunset Mid | `#BB6222` | assigned · v2 | — | Pommel glow, lower half of the ring (as in the shipped icons). |
| 9 | Sunset Orange | `#E58334` | assigned · v2 | — | Optional hotter glow pixel for a close-up or night view. The shipped icons do not use it; keep it to one pixel. |

Mask: niaviopti crop [0.4, 0, 1, 0.5] hue 50–100° chroma 0.035–1 L 0.3–1: 1,742 px (3.74% of crop); niaviopti crop [0.76, 0.035, 0.89, 0.095] hue 0–360° chroma 0–0.035 L 0.72–1: 307 px (25.21% of crop).

### Mode: Daylight / landscape — 8 new colours

#### Old Yhlgori sandstone — 4 steps, 1 new

Old Yhlgori architecture, shared by every Yhlgori-built city in the Neck (Hylvynhani, Klate): city walls, aqueducts, arched bridges, terraced Greco-Roman temples. Kept separate from the regional terrain so one building tileset serves all Yhlgori sites.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Straw Shade | `#513F29` | shared · v2 (ΔE 0.015) | `#504233` (0.389) | Sandstone wall and aqueduct in shadow, arch soffits, gorge-side foundations. |
| 2 | Yhlgori Sandstone | `#7C6954` | **NEW** · M | `#7C6954` (0.532) | Default old-Yhlgori sandstone mid: city walls, aqueduct piers, arched bridges, terraced temple courses. Also the mid of southern Maphlia golden grass and of Niaviopti white gold. |
| 3 | Ornament Gold | `#BA9974` | shared · v2 (ΔE 0.030) | `#AC9276` (0.678) | Sunlit sandstone face, temple columns, bridge parapets. |
| 4 | Sunset Gold | `#E5C695` | shared · v2 (ΔE 0.028) | `#DAC09F` (0.822) | Hottest sun-struck sandstone, terrace lips, the bright rim of a cliff city. |

Mask: hylvynhani crop [0, 0.36, 1, 0.75] hue 45–95° chroma 0.03–1 L 0–1: 12,157 px (20.21% of crop); hylvynhani_upper_city crop [0.05, 0.25, 0.5, 0.7] hue 40–100° chroma 0.02–1 L 0–1: 19,156 px (61.59% of crop); hylvynhani_falls crop [0.15, 0.15, 0.9, 0.6] hue 40–100° chroma 0.02–1 L 0–1: 26,874 px (51.84% of crop); klate crop [0.05, 0.35, 0.7, 0.66] hue 40–100° chroma 0.02–1 L 0–1: 7,760 px (25.12% of crop).

#### Old Yhlgori spire gold — 2 steps, 1 new

Gold spires and gilded roofs. Sampled from the three spires only (sunlit pixels, chroma ≥ 0.06, L ≥ 0.62); spire shade was not sampled, so shade spires with Yhlgori Sandstone or Antique Bronze.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Spire Gold | `#AA885C` | **NEW** · M | `#AA885C` (0.648) | Gold spire and gilded temple roof body in sun. Also the lit face of the tan granite peaks of the Maphyk Hylvanne. |
| 2 | Straw Sun | `#EDD09E` | shared · v2 (ΔE 0.017) | `#E7CD93` (0.857) | Spire highlight and gilded edge in full sun. Keep to a few pixels. |

Mask: hylvynhani_upper_city crop [0.5, 0.08, 0.61, 0.42] hue 55–95° chroma 0.06–1 L 0.62–1: 831 px (14.66% of crop).

#### Southern Maphlia golden grass — 3 steps, 0 new

Southern Maphlia / Maphyk Hylvanne piedmont plains. Every measured step already exists, so this ramp adds no colours.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Granite Shade | `#3F3C2F` | shared · v2 (ΔE 0.017) | `#483B2A` (0.362) | Dry grass in shade, plain under cloud shadow. |
| 2 | Yhlgori Sandstone | `#7C6954` | shared · v2.1 (ΔE 0.021) | `#866A4C` (0.545) | Default golden-grass mid of the piedmont plains. |
| 3 | Straw Lit | `#BD9B68` | shared · v2 (ΔE 0.021) | `#C6A069` (0.728) | Sunlit golden grass. |

Mask: hylvynhani crop [0, 0.83, 1, 1] hue 50–100° chroma 0.03–1 L 0–1: 11,322 px (43.15% of crop); hylvynhani_falls crop [0, 0.8, 0.6, 1] hue 40–100° chroma 0.03–1 L 0–1: 14,890 px (80.78% of crop).

#### Maphyk Hylvanne tan granite — 3 steps, 0 new

Tan granite peaks and gorges of the Maphyk Hylvanne. Every measured step already exists, so this ramp adds no colours.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Sandstone | `#665741` | shared · v2 (ΔE 0.025) | `#725B43` (0.488) | Tan granite in shade, gorge walls. |
| 2 | Spire Gold | `#AA885C` | shared · v2.1 (ΔE 0.022) | `#AE8E6A` (0.668) | Lit tan granite face (shared with Spire Gold). |
| 3 | Sunset Gold | `#E5C695` | shared · v2 (ΔE 0.019) | `#E7C5A2` (0.844) | Hottest granite summit in sun. |

Mask: hylvynhani crop [0, 0, 1, 0.42] hue 40–95° chroma 0.03–1 L 0–1: 9,612 px (14.94% of crop); hylvynhani_upper_city crop [0, 0, 1, 0.3] hue 40–100° chroma 0.03–1 L 0–1: 8,813 px (19.13% of crop).

#### Maphyk Hylvanne lakes — 2 steps, 1 new

Blue lakes and plunge pools below the falls. Greener and less saturated (OKLab chroma 0.058) than the Ptormyk lake (0.106).

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Hylvanne Lake | `#1E465C` | **NEW** · M | `#1E465C` (0.377) | Deep piedmont lake and plunge pool below the falls; teal-blue, more teal and about half as saturated as the Ptormyk lake. |
| 2 | Distance Blue | `#61829D` | shared · v2 (ΔE 0.020) | `#617B94` (0.573) | Lit lake surface and far water under open sky. |

Mask: hylvynhani crop [0.3, 0.75, 1, 0.88] hue 190–270° chroma 0.04–1 L 0–1: 2,245 px (16.17% of crop); hylvynhani_falls crop [0.45, 0.6, 0.9, 0.85] hue 190–270° chroma 0.03–1 L 0–1: 751 px (4.35% of crop).

#### Ptormyk grass — 3 steps, 1 new

Ptormyk design language: yellows and purples. The measured grass is golden yellow-green (OKLCh hue 104° in shade to 83° in sun).

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Needle Mid | `#404629` | shared · v2 (ΔE 0.017) | `#44411F` (0.370) | Ptormyk grass in shade, under conifers. |
| 2 | Straw Mid | `#8C6F45` | shared · v2 (ΔE 0.028) | `#826D33` (0.542) | Default Ptormyk grass mid. |
| 3 | Ptormyk Grass Sun | `#C59B44` | **NEW** · M | `#C59B44` (0.713) | Sunlit yellow grass of Ptormyk valleys; hotter and more saturated than Straw Lit. |

Mask: ptormyk_mountain_valley crop [0, 0.25, 1, 0.62] hue 75–125° chroma 0.035–1 L 0–1: 15,533 px (26.97% of crop).

#### Ptormyk violet rock — 3 steps, 2 new

The purplish mountain rock. Hue shifts toward violet as it lightens (272° → 281° → 292°).

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Deep Blue Blood | `#163056` | shared · v2 (ΔE 0.019) | `#252E55` (0.315) | Ptormyk rock in deep shadow (the swatch is shared with the blood colour; that is fine for stone). |
| 2 | Violet Rock | `#4B4D70` | **NEW** · M | `#4B4D70` (0.432) | Ptormyk mountain rock mid: purplish stone, the signature Ptormyk violet. |
| 3 | Violet Rock Lit | `#736F8A` | **NEW** · M | `#736F8A` (0.554) | Lit Ptormyk rock face and violet snow-peak shadow. |

Mask: ptormyk_mountain_valley crop [0, 0.05, 1, 0.5] hue 265–340° chroma 0.02–0.2 L 0.2–0.72: 24,581 px (35.32% of crop).

#### Ptormyk snow peaks — 2 steps, 0 new

The snow on the peaks measures blue-grey (hue 255° and 247°), not violet: the violet read comes from the rock beside it. Both steps already exist.

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Blue Skin | `#8FB4D1` | shared · v2 (ΔE 0.015) | `#9AB3D4` (0.760) | Snow in shadow on violet peaks. |
| 2 | Snow Lit | `#C9D0DE` | shared · v2 (ΔE 0.007) | `#C6D0DA` (0.854) | Sunlit snow. |

Mask: ptormyk_mountain_valley crop [0, 0.05, 0.8, 0.4] hue 230–340° chroma 0.0–0.2 L 0.72–1: 10,904 px (25.13% of crop).

#### Ptormyk lake — 2 steps, 2 new

Saturated blue lake and peak reflections (OKLab chroma 0.106 at both steps).

| Step | Swatch | Hex | Status | Measured mean (OKLab L) | Role |
| ---: | --- | --- | --- | --- | --- |
| 1 | Ptormyk Lake Deep | `#17346E` | **NEW** · M | `#17346E` (0.340) | Deep saturated blue of Ptormyk lakes and peak reflections. Also the Price Line blue for cloth and banners. |
| 2 | Ptormyk Lake | `#4779B4` | **NEW** · M | `#4779B4` (0.568) | Lit Ptormyk lake surface. Also the Price Line light blue. |

Mask: ptormyk_mountain_valley crop [0, 0.6, 1, 1] hue 235–280° chroma 0.06–1 L 0–1: 24,078 px (38.59% of crop).

## Ramps carried from v2 (unchanged)

### Mode: Core / shared — 38 v2 colours

Used in every lighting mode: void, metals, torchlight, caste skin, cloth, linen, eyes, and a short shared stone set.

#### Night black / void — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Void Ink | `#07080E` | M. Deepest night, cave interiors, inverted sclera, outline fallback. Source:  Vakirros Grymsyhl 26.54%. |
| Warm Black | `#0D0A0B` | M. Default shadow and frame black; slightly warmer than void. Source:  v1 overall k=16, 29.01%. |
| Lace Black | `#171718` | M. Black cloth, lace, hair-dark, UI chrome. Source:  Meri Audsen 13.73%. |
| Charcoal | `#1D1C1D` | M. Armour in shadow, mid-void, smoke, dead-pine trunks. Source:  v1 overall k=16, 16.28%. |

#### Gold / bronze — 5, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Bronze Dark | `#3F2517` | M. Gold in shadow, bronze fittings. Source:  Shjana 6.13%. |
| Antique Bronze | `#644329` | M. Mid gold, laurel leaves in shade, coin faces. Source:  Shjana 4.13%. |
| Ornament Gold | `#BA9974` | M. Armour ornament, jewellery, Line devices. Source:  v1 overall k=16, 3.28%. |
| Star Gold | `#D6AA5C` | M. Rare bright metal; also the gold-star accent from the lake painting. Source:  worlds splash 0.49%. |
| Highlight Gold | `#EFD7A5` | M. Specular clicks on metal and the hottest sun-struck edge. Source:  throne 1 3.52%. |

#### Torchlight — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Hearth Dark | `#291108` | M. Torch-lit shadow, warm interiors. Source:  rites 1 25.81%. |
| Flame Mid | `#803518` | M. Fire body, lit wood, warm bounce on armour. Source:  rites 1 5.46%. |
| Lamp Gold | `#BE8B54` | M. Flame highlight, candle cores, warm metal bounce. Source:  rites 1 1.75%. |

#### Warm stone / sand — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Canyon Shade | `#2E2117` | M. Lit canyon shadow, warm earth, desert shade. Shared by the throne painting and the Big Bend photos. Source:  throne 1 10.81%. |
| Sandstone | `#665741` | M. Pillar mid, road, dust. Canyon-mouth #63574B (11.95%) is merged here. Source:  throne 1 7.59%. |
| Sun Stone | `#927955` | M. Sun-struck masonry and sand. Neighbours cliff #856B5D and #9E8A76 were not locked. Source:  throne 1 4.73%. |

#### Cut stone — 2, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Cliff Teal | `#1D2729` | M. North-facing rock, mountain shadow, night stone. Kept from v1; the teal shift is unique to the canyon painting. Source:  throne 1 19.58%. |
| Cold Ashlar | `#404C53` | M. Cut stone, keep walls in cool light. Architectural, not alpine granite (see the day granite ramp). Source:  Kheinyn Belravi 5.08%. |

#### Red-blood skin — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Skin Shade | `#5D3822` | M. Red-blood shadow, tanned mid-tone shade. Source:  Elyssa Dane 9.42%. |
| Skin Mid | `#96704E` | M. Default red-blood mid. Source:  Elyssa Dane 3.07%. |
| Skin Light | `#C9A982` | M. Lit flesh, cheeks. Source:  Elyssa Dane 3.96%. |
| Skin Highlight | `#E5CEB9` | M. Small flesh clicks, fair red-blood highlight. Source:  Kaenyn Audsen 5.19%. |

#### Blue-blood sky — 4, 1 measured, 3 adjusted

| Name | Hex | Role |
| --- | --- | --- |
| Blue Shade | `#3A5560` | A. Night or form-shadow on sky-blue skin. Do not use as a day-sky fill. Source:  from Kheinyn #2D3236 / #404C53, shifted toward sky. |
| Blue Mid | `#5B8FA8` | A. Mid skin. Not a landscape sky step — use Sky Mid or Sky Light for the heavens. Source:  from Kheinyn #5B6A72 / #728B97, raised saturation. |
| Blue Skin | `#8FB4D1` | M. Main sky-blue body. Source:  Kheinyn Belravi highlight 1.09%. |
| Blue Crystal | `#C5E3F0` | A. Daytime crystal highlight. Keep to a few pixels. Source:  daytime crystal highlight for sky-blue skin. |

**Lore vs paint, unchanged from v1.** The brief and the manuscript (`A Pylgrym's Plight`, Halion's bath) describe true blue-blood skin as sky-coloured, "crystal blue," a "royal hue," with **no visible blue veins on the face**. The Kheinyn Belravi concept is a greyer teal; those measured greys were reserved for the cut-stone ramp, and this ramp was pulled toward sky. Samule's travel story in the same manuscript describes southern-isle blue-bloods as tall, dark-haired, "skin the color of the sea," purple eyes, and blood "the color of the sky in the dusk of falling night." Treat sea-green skin and purple irises as a regional variant, not the default sprite recipe. Default blood is Deep Blue Blood, not dusk-violet, unless a southern character is specified.

Do not use this ramp as a day-sky fill. Day heavens use the Day sky blue ramp.

#### Blue blood (wound) — 1, all adjusted

| Name | Hex | Role |
| --- | --- | --- |
| Deep Blue Blood | `#163056` | A. Fresh blue-blood wounds, spatters, ritual cups. Deep blue, not sky-blue and not purple. Source:  lore: deep blue blood. See palette.md for the dusk-indigo variant. |

A dusk-indigo variant is *not* in this locked set. If southern dusk-blood is later required, review it as a new ramp rather than mixing a night purple ad hoc.

#### Black-blood silver — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Silver Shade | `#2E383E` | M. Metallic skin in shadow. Source:  Shjana 6.48%. |
| Silver Mid | `#46565E` | M. Mid silver flesh. Source:  Shjana 3.45%. |
| Warm Silver | `#A7A391` | M. Lit metal-flesh, slightly golden. Source:  Shjana 2.54%. |
| Silver Highlight | `#D0D8D3` | M. Specular clicks on silver skin and steel. Source:  Shjana 1.83%. |

Black-blood skin should look like metal, not grey paint: shade through cool silver and hit Warm Silver or Highlight Gold where the key light lands. The whiteboard notes (`A Pylgrym's Plight — Collated Whiteboard Notes`, "The blood colours") give Kamloran / Myhjodi black-bloods silvered skin and blood-orange eyes. The manuscript also has Samule describe far-western black-bloods whose skin "shined like metal and was the color of bronze and gold" — use Bronze Dark / Antique Bronze as a westward regional variant, not the default. Default irises are Iris Amber or a Cloak Mid + Star Gold mix for blood-orange.

#### Cloth reds — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Cloak Shade | `#381B12` | M. Red cloak in shadow, leather. Source:  Kaenyn Audsen 19.16%. |
| Cloak Mid | `#A13E18` | M. Paludamentum mid, sash, Line red. Source:  Kaenyn Audsen 9.57%. |
| Cloak Bright | `#CC1D19` | M. Lit cloak edge, banner flash. Source:  v1 overall k=16, 0.85%. |

#### White / linen / hair — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Hair Mid | `#9B9B93` | M. Grey-white hair mid, dingy linen. Source:  Meri Audsen 6.85%. |
| Pale Silver | `#BDCECC` | M. White hair, cool linen, lace highlight. Near Sky Pale but less blue; do not use as sky. Source:  Meri Audsen 5.73%. |
| Ivory | `#EBEADC` | M. White cloth in light, paper, UI text, overcast cloud click. Stands in for measured overcast whites #F0F1F4 / #E7EAE8. Source:  Autumn Sky 23.27%. |

#### Eyes and accents — 2, 1 measured, 1 adjusted

| Name | Hex | Role |
| --- | --- | --- |
| Iris Amber | `#B29866` | M. Default amber / gold iris (Kheinyn, Vakirros, black-blood blood-orange base). Source:  Kheinyn Belravi 1.46%. |
| Cloudy Pupil | `#C9C4B8` | A. The pale, glossy, marble-like pupil of inverted (meha nia / cold-flame) eyes. Source:  inverted-eye pale glossy pupil. |

**Inverted eyes** at sprite scale: sclera = Void Ink or Lace Black, iris = Iris Amber (or Cloak Mid / a later purple), pupil = Cloudy Pupil if the canvas has room, otherwise skip the pupil and keep black sclera plus one iris pixel. See the style guide.

### Mode: Night / celestia — 5 colours

The red-star night sky. Do not use day-sky or haze blues here.

#### Blood red sky — 5, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Ember Void | `#1C0101` | M. Red-sky low values, nebula body. Source:  runaway 1 26.81%. |
| Dried Blood | `#3A0504` | M. Sky mid-dark, dried wound on red-bloods. Source:  runaway 1 12.55%. |
| Crimson Dusk | `#680A0A` | M. Sky mid, distant nebula. Source:  runaway 1 5.30%. |
| Signal Red | `#A61313` | M. Brighter sky, small star bodies, UI alerts. Source:  runaway 1 4.04%. |
| Star Red | `#E81F1F` | M. Star cores and spark accents. Keep rare. Source:  runaway 1 3.27%. |

This is the signature night. Default stars use this ramp, not white or cool blue. See the style guide night-sky section. Gold stars in `worlds splash.png` remain a special pass (Star Gold), not the default celestia.

### Mode: Daylight / landscape — 36 v2 colours

Open-sky, atmospheric, biome, and daylight-material ramps. Characters in daylight still use the core caste, metal, and cloth ramps.

#### Day sky blue — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Zenith Blue | `#456EC0` | M. Deep clear zenith, the Big Bend–style open sky. Default day high sky. Source:  canyon mouth 14.64%. |
| Sky Mid | `#5A82C2` | M. Mid day sky, slightly paler than zenith, also the cliff-photo sky body. Source:  Big Bend cliff 16.37%. |
| Sky Light | `#80A4DE` | M. Lower sky, open-meadow blue, light haze at the top of the sky dome. Source:  landscapes overall k=16, 3.52%. |
| Sky Pale | `#BDCDE4` | M. Pale sky, thin cloud, bright atmosphere. Cooler and bluer than Pale Silver. Source:  landscapes overall k=16, 5.15%. |

#### Atmospheric haze / distance — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Near Mass | `#3D3B44` | M. Nearest distance layer: dark treeline and close ridgeline before the playfield. Source:  winter Appalachian 12.84%. |
| Mid Ridge | `#555862` | M. Second distance layer. One step bluer and lighter than Near Mass. Source:  winter Appalachian 8.05%. |
| Distance Blue | `#61829D` | M. Third distance layer. Clearly blue; this is the mid Blue Ridge / Hudson River fade. Source:  winter Appalachian 5.03%. |
| Horizon Veil | `#85B5CE` | M. Farthest ridge and horizon haze. One step from Distance Blue. Do not skip here from Near Mass. Source:  winter Appalachian 3.86%. |

**Aerial-perspective rule.** One ramp step per parallax depth layer, back to front: Horizon Veil → Distance Blue → Mid Ridge → Near Mass → the playfield material ramps. Do not skip a step. Golden hour may tint the two farthest steps toward Sunset Gold and Straw Sun. Night does not use this ramp; far ridges go to void, Cliff Teal, or Ember Void. See the style guide.

#### Conifer greens — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Forest Shade | `#10271B` | M. Deep conifer and leaf shade, forest interiors, backlit tree mass. Source:  backlit forest 37.15%. |
| Needle Dark | `#2A2E1B` | M. Dark needles, shaded spruce and fir. Source:  overcast Rocky valley 14.57%. |
| Needle Mid | `#404629` | M. Default conifer mid, overcast canopy. Source:  overcast Rocky valley 19.29%. |
| Needle Lit | `#545B35` | M. Lit needles, juniper mid, sun on a conifer face. Source:  overcast Rocky valley 16.20%. |

#### Meadow and grass — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Grass Shade | `#3A3F31` | M. Grass in shadow, meadow shade under pines. Source:  sunlit clearing 17.07%. |
| Grass Mid | `#56593B` | M. Default growing-season grass mid. Source:  sunlit clearing 12.16%. |
| Grass Lit | `#B9B05E` | M. Sunlit yellow-green meadow, lit clearing grass. Source:  sunlit clearing 7.79%. |
| Grass Sun | `#D7CB75` | M. Hottest grass highlight. Also the stand-in for yellow alpine flowers (#E0D675 neighbour, not locked). Source:  sunlit clearing 8.60%. |

#### Dry grass / straw — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Straw Shade | `#513F29` | M. Dry-grass shade, harvest shadow, tan field in shade. Source:  canyon trail 15.40%. |
| Straw Mid | `#8C6F45` | M. Default straw and dry grass mid. Tamarisk mid sits nearby. Source:  canyon trail 11.91%. |
| Straw Lit | `#BD9B68` | M. Lit straw, winter tan field, tamarisk gold neighbour (#C3AB71 not locked). Source:  canyon trail 9.89%. |
| Straw Sun | `#EDD09E` | M. Hottest dry-grass click, backlit straw, pale harvest haze. Source:  canyon trail 3.61%. |

#### Autumn / sunset oranges — 4, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Sunset Shade | `#563B1D` | M. Golden-hour shadow, backlit trunk, late-sun earth. Source:  backlit forest 12.70%. |
| Sunset Mid | `#BB6222` | M. Orange sun-through-trees, mid sunset cloud, autumn canopy mid. Source:  backlit forest 4.73%. |
| Sunset Orange | `#E58334` | M. Bright sunset orange, sun disk fringe, rare leaf flash. Keep small. Source:  backlit forest 3.26%. |
| Sunset Gold | `#E5C695` | M. Golden-hour sky and lit granite, Cole/Bierstadt gold light. Neighbour of alpine #CB9162 and Cole #E2B380. Source:  alpine sunset 5.77%. |

#### Granite greys — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Granite Shade | `#3F3C2F` | M. Alpine granite in shadow. Replaces v1 Mountain Grey #30312C. Source:  granite cirque 12.65%. |
| Granite Mid | `#55534A` | M. Default granite mid, boulder field, cirque floor. Source:  granite cirque 17.54%. |
| Granite Lit | `#928773` | M. Lit granite face. For a hotter click use Lime Lit or Sun Stone, not a new grey. Source:  granite cirque 10.40%. |

#### Limestone / sandstone (extension) — 2, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Lime Lit | `#BFA991` | M. Lit limestone and pale sandstone, warmer and lighter than Sun Stone. Source:  canyon mouth 7.76%. |
| Lime Pale | `#E6D2B6` | M. Hottest limestone click, pale desert earth, sun-struck canyon lip. Source:  canyon mouth 3.93%. |

#### Snow — 2, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Snow Mid | `#B8BCC5` | M. Shaded-to-mid snow, slightly blue-grey. Sampled from bright cool pixels in the cirque bowl, not a whole-image cluster. Source:  cirque snow, V 70–85 band (93,886 px). |
| Snow Lit | `#C9D0DE` | M. Sunlit snow and snow highlight. Cooler than Ivory. Deep snow shade may use Mid Ridge or Pale Silver. Source:  cirque snow, V 85–100 band (34,043 px). |

#### Water — 3, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Water Deep | `#44433A` | M. Dark alpine water and shaded tarn, reflecting granite. Source:  tarn surface 30.85%. |
| River Teal | `#6D7A65` | M. Painted river, olive-teal water as photographed (museum light; not a bright cyan). Hudson River School water mid. Source:  Cole-style painting, stream cluster 20.18%. |
| Lake | `#AEBACB` | M. Open water under a blue sky, mid reflection. Brighter clicks use Sky Pale or Snow Lit. Source:  lake water 28.63%. |

#### Fireweed / flower accents — 2, all measured

| Name | Hex | Role |
| --- | --- | --- |
| Fireweed Mid | `#805D4C` | M. Dusty rose-brown fireweed mid. These photos are late-season seed plumes, not peak magenta; a brighter bloom would need a future review. Source:  lake-fireweed mid plumes, V 40–60 band (7,882 px). |
| Fireweed | `#B68D79` | M. Lit fireweed plume and rose stem accent. Keep rare. Yellow flowers use Grass Sun. Source:  lake-fireweed lit plumes, V ≥ 60 band (8,693 px). |

## Counts

| Mode / ramp | Colours | New in v2.1 |
| --- | ---: | ---: |
| Core / shared / Night black / void | 4 | 0 |
| Core / shared / Gold / bronze | 5 | 0 |
| Core / shared / Torchlight | 3 | 0 |
| Core / shared / Warm stone / sand | 3 | 0 |
| Core / shared / Cut stone | 2 | 0 |
| Core / shared / Red-blood skin | 4 | 0 |
| Core / shared / Blue-blood sky | 4 | 0 |
| Core / shared / Blue blood (wound) | 1 | 0 |
| Core / shared / Black-blood silver | 4 | 0 |
| Core / shared / Cloth reds | 3 | 0 |
| Core / shared / White / linen / hair | 3 | 0 |
| Core / shared / Eyes and accents | 2 | 0 |
| Core / shared / Price Line purples, blues, whites | 3 | 3 |
| Core / shared / Niaviopti white gold, pearl, glow | 1 | 1 |
| **Core / shared subtotal** | **42** | **4** |
| Night / celestia / Blood red sky | 5 | 0 |
| **Night / celestia subtotal** | **5** | **0** |
| Daylight / landscape / Day sky blue | 4 | 0 |
| Daylight / landscape / Atmospheric haze / distance | 4 | 0 |
| Daylight / landscape / Conifer greens | 4 | 0 |
| Daylight / landscape / Meadow and grass | 4 | 0 |
| Daylight / landscape / Dry grass / straw | 4 | 0 |
| Daylight / landscape / Autumn / sunset oranges | 4 | 0 |
| Daylight / landscape / Granite greys | 3 | 0 |
| Daylight / landscape / Limestone / sandstone (extension) | 2 | 0 |
| Daylight / landscape / Snow | 2 | 0 |
| Daylight / landscape / Water | 3 | 0 |
| Daylight / landscape / Fireweed / flower accents | 2 | 0 |
| Daylight / landscape / Old Yhlgori sandstone | 1 | 1 |
| Daylight / landscape / Old Yhlgori spire gold | 1 | 1 |
| Daylight / landscape / Southern Maphlia golden grass | 0 | 0 |
| Daylight / landscape / Maphyk Hylvanne tan granite | 0 | 0 |
| Daylight / landscape / Maphyk Hylvanne lakes | 1 | 1 |
| Daylight / landscape / Ptormyk grass | 1 | 1 |
| Daylight / landscape / Ptormyk violet rock | 2 | 2 |
| Daylight / landscape / Ptormyk snow peaks | 0 | 0 |
| Daylight / landscape / Ptormyk lake | 2 | 2 |
| **Daylight / landscape subtotal** | **44** | **8** |
| **Total** | **91** | **12** |

Ramps with 0 colours are made entirely of shared swatches; they are listed for use but have no group in the `.ase`.

## Key measured findings, v2 (for citation)

Hue-family shares are on every pixel of the long-edge-480 images, equal-weighted within each set. v1 portrait overalls were re-run with the new script and match the v1 report exactly (`#0D0A0B` 29.01% and `#1D1C1D` 16.28%).

1. **The new references really are brighter and cooler in the sky.** Across the twenty landscapes equally weighted, sky blues are **15.61%** (mean `#809BC6`), against only **1.25%** in the eleven painted portraits. The landscape overall k=16 includes `#BDCDE4` 5.15%, `#80A4DE` 3.52%, `#527EC9` 3.08%, and `#7690B1` 2.25%. The canyon-mouth and Big Bend cliff photos give deep saturated zeniths (`#456EC0` 14.64% and `#5A82C2` 16.37%). Those cannot be faked with the blue-blood skin ramp.
2. **Greens are now a real family, not a rounding error.** Landscape greens are **11.99%** (mean `#37442E`), against **0.46%** in the portraits. Highest shares: backlit forest 45.84%, juniper 41.57%, overcast Rocky valley 36.86%, conifer trail 24.35%, sunlit clearing 23.76%. They are dark-to-mid olive and conifer, not neon: overcast valley `#404629` 19.29% and `#545B35` 16.20%; juniper `#44533C` 19.62%; backlit forest `#10271B` 37.15%. The sunlit clearing also supplies the bright yellow-greens `#B9B05E` 7.79% and `#D7CB75` 8.60%.
3. **Oranges and golds dominate the new set: 42.30%** of landscape pixels (mean `#7D6A4F`) — dry grass, sandstone, sunset, straw, tamarisk, and the two gold-light paintings. Canyon trail is **82.70%** oranges/golds; prickly pear **89.24%**; the cropped Bierstadt **79.09%**; alpine sunset **68.90%**; the cropped Cole canvas **63.29%**; tamarisk **57.69%**. The backlit forest gives the hot sunset oranges `#E58334` 3.26% and `#BB6222` 4.73%.
4. **Aerial perspective is measured, not invented.** The winter Appalachian valley is **48.67%** greys, with tan near fields (`#977D68` 6.21%) and a clear distance stack: near wooded mass `#302C30` 14.92% / `#3D3B44` 12.84%, mid `#555862` 8.05%, far blue `#61829D` 5.03% and `#85B5CE` 3.86%, under an overcast white `#EFF2EF` 25.76%. The Blue Ridge sunset uses a warmer gold haze instead (`#BCA287` 8.05%, `#DABD98` 6.34%, `#E9DAB7` 5.32%) over a near-black ridge `#12161A` 41.04%.
5. **Greys are substantial but no longer the whole story.** Landscape greys are **17.92%** (mean `#969798`). The waterfall boulder field is **40.30%** grey; the juniper-and-cliff photo **35.07%**; the overcast valley **34.16%**. Combined with the portraits, the 31-image overall is still warm-earth heavy (oranges/golds **41.97%**), but sky blues rise to **10.48%** and greens to **7.85%**, against a portrait-only green share of 0.46%.

v1 findings (portraits only) still hold and still govern night and character work: the horseman's three brightest reds are only 10.69% of that image; 45.29% of the eleven-painting overall sits in the two darkest clusters; the canyon painting splits cool and warm stone; the rites painting is a tight warm ramp (four darkest orange-browns 75.38%); black-blood silver is a lit edge, not a fill (14.30% of the Shjana portrait).

## Retired and merged

These v1 locked colours are **not** in the v2 swatch. Use the replacement named here.

| Retired | v1 hex | Use instead | Why |
| --- | --- | --- | --- |

| Leaf Shade | `#594032` | Straw Shade or Sunset Shade | The v1 autumn foliage ramp is superseded by measured straw and sunset families from the landscape set. |
| Leaf Mid | `#7C5941` | Straw Mid | Merged into the dry-grass ramp. |
| Leaf Dry | `#997755` | Straw Mid or Straw Lit | Merged into the dry-grass ramp. |
| Harvest Haze | `#D0C2A9` | Straw Sun or Lime Pale | Backlit dry vegetation and pale limestone now have tighter measured neighbours. |
| Mountain Grey | `#30312C` | Granite Shade #3F3C2F | Same olive-grey mountain role, updated from the alpine granite photos. |

**Kept but now sharing space with a neighbour** (old hex remains the locked colour):


- Sandstone #665741 absorbs canyon-mouth #63574B (11.95%).
- Sun Stone #927955 neighbours cliff #856B5D and canyon-mouth #9E8A76; those were not locked.
- Ivory #EBEADC stands in for overcast whites #F0F1F4, #E7E6E7, and #E7EAE8.
- Tamarisk gold #C3AB71 sits next to Straw Lit #BD9B68.
- Alpine #CB9162 sits next to Sunset Gold / Sunset Mid.
- Yellow alpine flowers #E0D675 sit next to Grass Sun #D7CB75; use Grass Sun.
- Cascade foam #E2D9D3 is a neighbour of Snow Lit / Ivory; not locked.
- Granite sun #B1A48E can use Lime Lit or Sun Stone.


## Intentionally unused measured neighbours

These centroids or regional means were recorded but not locked, so a later review can recover them. v1 unused neighbours remain available in `analysis/measured-colors-v1.json`.

- Overcast / cloud whites `#F0F1F4`, `#E7E6E7`, `#E7EAE8`, `#D7D9DF` (Ivory stands in)
- Cliff mid `#856B5D` and canyon-mouth `#9E8A76`, `#63574B` (warm-stone neighbours)
- Granite sun `#B1A48E` (use Lime Lit or Sun Stone)
- Tamarisk gold `#C3AB71` (use Straw Lit)
- Alpine sunset `#CB9162` and Cole `#E2B380` (use Sunset Gold / Sunset Mid)
- Yellow alpine-flower sun `#E0D675` (use Grass Sun)
- Cascade foam `#E2D9D3` (use Snow Lit or Ivory)
- Juniper mid `#44533C` (use Needle Mid / Needle Lit)
- Far-ridge extra `#739DB9` (between Distance Blue and Horizon Veil)
- Peak-bloom fireweed was not in these photos; the locked Fireweed colours are late-season dusty rose. A brighter magenta would need a future review.

