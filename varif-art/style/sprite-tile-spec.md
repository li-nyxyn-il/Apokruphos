# Sprite and tile spec

Draft 1 — 2026-11-16 (America/Chicago). Engine-neutral. Companion to the style guide (draft 2, tile grid updated the same day).

This document locks the floor grid and the character canvas that the style guide left open, and records production defaults for anything still unconfirmed. A value marked **default** may change before a first atlas is cut; a value marked **confirmed** should not change without revising this spec, the style guide, and the shot list together.

Related files: `style-guide.md`, `naming-convention.md`, `shot-list.md`, `palette.md` (v2, 79 colours).

## Confirmed

- **Projection.** 2:1 isometric floor. Three-quarter characters at Tiers 1–2, as in the style guide.
- **Floor tile.** 32×16 pixel diamond. Same pixel budget as the retired 16×16 square default.
- **Character canvas.** 32×32. Two tile-heights tall (32 ÷ 16).
- **First assets.** Kaenyn Audsen's Tier-1 idle sprite, and one isometric meadow-grass floor tile in day and night modes. See `shot-list.md`.

## Defaults (still changeable)

- **Facings.** 4 isometric directions: `se`, `sw`, `ne`, `nw`. These follow the four grid axes. An 8-facing set (adding screen-cardinal `n`, `s`, `e`, `w`) is a later upgrade, not part of the first set.
- **Catalogue facing.** `se` is the default hero angle for stills, reviews, and the first frame of a sheet.
- **Idle.** 4 frames, 200 ms each (5 fps), looping. Frame `00` is the rest pose. Motion is a 1-pixel breathing or cloak sway; the silhouette must stay stable.
- **Character pivot.** Feet, bottom-centre: `(0.5, 0.0)` in Y-up normalised coordinates, or pixel `(16, 0)` from the bottom-left of the 32×32 canvas.
- **Floor-tile pivot.** Centre of the diamond: `(0.5, 0.5)` in Y-up normalised coordinates, or pixel `(16, 8)` from the bottom-left of the 32×16 canvas.
- **Block / wall pivot.** Centre of the *floor* diamond, not the centre of the tall canvas. On a 1-step 32×32 block that is `(0.5, 0.25)`, or pixel `(16, 8)` from the bottom-left. A floor tile and a block in the same cell then share a world position.
- **Block height.** One height step is **16 px**. A 1-step cube or wall uses a 32×32 canvas: a 32×16 floor diamond plus 16 px of volume. A wall of `n` steps uses a canvas of 32 × (16 + 16×n).
- **Map-axis mapping (Y-up screen).** Map +X is `se`, screen `(+16, -8)` per cell. Map +Y is `sw`, screen `(-16, -8)` per cell. Map −X is `nw`, `(−16, +8)`. Map −Y is `ne`, `(+16, +8)`.
- **Pixels per unit.** 32, matched to the tile width, not the character. One floor cell is 1 world unit wide and 0.5 tall; a 32×32 character is 1 unit tall (two tile-heights).
- **Atlas origin.** `y_up`, as in the style guide. Photoshop and Aseprite are Y-down; flip Y when exporting metadata, not when the engine reads it.
- **Night grass recipe.** A night floor tile may use the core set plus the darker steps of the daylight grass ramp (Grass Shade, Grass Mid) and void. It must not use Day sky blue, Grass Sun, or Lime Pale, and must not paint Blood red sky on the ground.

## Floor tile layout

Canvas: **32×16**, transparent outside the diamond. Author only the diamond; do not leave stray pixels in the corners.

Vertices, in Photoshop / editor space (Y-down, origin top-left):

- North: `(15–16, 0)`
- East: `(31, 7–8)`
- South: `(15–16, 15)`
- West: `(0, 7–8)`

The same vertices in atlas space (Y-up, origin bottom-left):

- North: `(15–16, 15)`
- East: `(31, 7–8)`
- South: `(15–16, 0)`
- West: `(0, 7–8)`

The edges follow a 2:1 stairstep: two horizontal pixels for each vertical pixel. The diamond is 32 px at its widest row and 16 px tall. Fill is opaque palette colour; the four corners of the canvas stay fully transparent so adjacent tiles can mesh.

## Block and wall tiles

A block is a floor diamond with volume stacked above it.

- **1-step block (default cube / low wall).** 32×32 canvas. The floor diamond occupies the bottom 16 px (Y-up `y = 0..15`). The top face is a second 32×16 diamond whose south tip sits 16 px above the floor south tip. The two side faces fill the gap. Height of the volume is one tile-height (16 px).
- **Taller walls.** Canvas width stays 32. Canvas height is `16 + 16n` for `n` height steps (2-step = 32×48, 3-step = 32×64). The floor diamond remains in the bottom 16 px, and the pivot stays at that diamond's centre.
- **Depth sort.** Sort by the floor-diamond pivot (cell position), then by height if two volumes share a cell. Do not sort from the top of the sprite, or a tall wall will pop in front of a character standing south of it.

This first shot list does not include a block tile; the rule is here so later wall and keep art uses the same grid.

## Character canvas

- Canvas: **32×32**, transparent padding around the figure.
- The figure may occupy most of the box, but the feet sit on the bottom edge, centred horizontally. Hair, helm, or a raised weapon may reach the top row; they must not be cropped.
- Placing the character at a cell's world position (the floor-diamond centre) puts the feet on the ground in the middle of that tile.
- Tier 1 draws the full close-LOD treatment from the style guide: cloth folds, gold trim, weapon if any, hair or helm, and the eyes. Kaenyn Audsen, from the named concept painting, is a red-blood with fair skin, ice-blue natural eyes (not inverted), gold-ornamented armour, and a red paludamentum. Audsen Line colour tendency is maroon and gold — Cloth reds and the gold metal ramp, from the core set.
- A later 16×16 tactical LOD, if cut, is a separate asset with its own name and atlas entry, not a filtered shrink of the 32×32.

## Facings and idle

**Default: 4 facings.** Draw `se`, `sw`, `ne`, and `nw`. `se` is the default catalogue facing.

Each facing of the idle cycle is 4 frames:

| Frame | Role | Duration (default) |
| --- | --- | --- |
| `00` | Rest pose | 200 ms |
| `01` | Anticipation (1 px cloak or chest) | 200 ms |
| `02` | Peak of the breath or sway | 200 ms |
| `03` | Return | 200 ms |

Loop `00–03`. The same duration applies to every facing. Walk, attack, and other actions are not in the first set; when they arrive they reuse this facing set, pivot, and frame padding (`00`, `01`, …).

## Outline and light

Taken from the style guide; repeated here so a sprite sheet can be drawn from this document alone.

- **Key light, production default:** upper-left, about 10–11 o'clock, 30–45° elevation. Tile bevels and face planes stay consistent across the atlas. Note any exception on the asset (a west-facing cliff at dusk, a shrine lit from a right-hand fire).
- **Iso cubes follow that light.** Top face lit; screen-left face mid-lit; screen-right face in shade. Sun clicks stay small and hard (Grass Sun, Highlight Gold), never a wash.
- **Hue-shifted shading.** Shadows are not a greyer mid. Day foliage shade is Grass Shade or Forest Shade. Night shade goes cool (Cliff Teal, Void Ink). Flesh shade is Skin Shade. A mid that cannot reach its shade in three steps or fewer needs a ramp review, not a new colour.
- **Selective outline.** Close-tier characters may use a cluster outline in Warm Black or Lace Black, or a shade from the same material, not a cartoon ring of Void Ink around every form. Terrain uses outlines only at a cliff break or a tile lip that must read at Tactical zoom. Tokens and banners may use a one-pixel dark rim so they survive the phone floor.
- **Silhouette first.** Read the character in one-bit before spending the gold trim.
- **Nearest neighbour only.** No anti-aliased edges. Integer scale at rest.

## Palette subsets

Locked set: palette v2, 79 colours, in `palette.md`. Do not add a colour without a ramp review. Every finished sprite or tile must have **0 off-palette pixels**.

| Asset kind | Subset | Notes |
| --- | --- | --- |
| Characters, armour, cloth, caste skin, eyes | **Core / shared** (38) | Available in every lighting mode. Daylight does not recolour caste skin into landscape ramps. |
| Day floor and biome tiles | **Core + daylight / landscape** (38 + 36) | Meadow grass uses Grass Shade / Mid / Lit / Sun. Do not paint a day valley in the Blood red sky. |
| Night floor tiles | **Core + darker biome steps** (see default recipe above) | Low-key. No Day sky blue, no Grass Sun, no Lime Pale, no Blood red sky on the ground. Far land, if it appears, goes to Void Ink, Cliff Teal, Charcoal, or Ember Void. |
| Night sky layer | **Core + night / celestia** (38 + 5) | Blood red sky only for the heavens. Default stars are Signal Red and Star Red. Do not fill a night sky with Zenith Blue. |

Pick one lighting mode per scene. The style guide's four modes are Day, Golden hour, Overcast, and Night under the red stars. The first grass tile ships a **day** and a **night** pass only. Golden hour and overcast remain future mode tokens.

## Atlas metadata

Pivots and rects live in a script-friendly JSON atlas, not only inside an editor binary. The shape extends the example in the style guide. It is a recommended format, not a frozen schema. Do not pack engine-specific material data into this file.

Common rules, either engine: point / nearest filtering; no texture compression on sprites or tiles; no interpolated mipmaps for pixel sheets; at-rest camera zoom is an integer scale.

```json
{
  "name": "varif-tier1",
  "texture": "varif-tier1.png",
  "filter": "nearest",
  "origin": "y_up",
  "tile_size": [32, 16],
  "ppu": 32,
  "sprites": [
    {
      "id": "char_kaenyn_audsen_idle_se_00_t1",
      "rect": [0, 0, 32, 32],
      "pivot": [0.5, 0.0],
      "tier": 1,
      "category": "char",
      "subject": "kaenyn",
      "variant": "audsen",
      "action": "idle",
      "dir": "se",
      "frame": 0,
      "duration_ms": 200,
      "line": "Audsen",
      "caste": "red-blood",
      "palette": "core"
    },
    {
      "id": "tile_grass_meadow_00_day_t1",
      "rect": [0, 32, 32, 16],
      "pivot": [0.5, 0.5],
      "pivot_px": [16, 8],
      "tier": 1,
      "category": "tile",
      "subject": "grass",
      "variant": "meadow",
      "frame": 0,
      "mode": "day",
      "kind": "floor",
      "height_steps": 0,
      "palette": "core+daylight"
    }
  ]
}
```

`rect` is `[x, y, w, h]` in texture pixels, with `y` measured from the origin declared in `origin`. `pivot` is normalised `(x, y)` from the bottom-left in Y-up space. `pivot_px` is optional and is the same point in pixels, useful for tall block canvases where the normalised Y is not 0.5.

A 1-step block would set `"kind": "block"`, `"height_steps": 1`, `"pivot": [0.5, 0.25]`, and `"pivot_px": [16, 8]`. Tokens and banners are further sprites with `"tier": 3` or `"tier": 4`.

**Unity (when that path is chosen).** Sprite Filter Mode: Point. Compression: None. Mip maps off. Default PPU 32, as above. Pixel-snap the camera at rest.

**Bevy (when that path is chosen).** Nearest sampling (`ImagePlugin::default_nearest()` or an equivalent sampler). Bevy 2D is Y-up; this JSON is already in Y-up.

File ids in the atlas must match the names in `naming-convention.md`.
