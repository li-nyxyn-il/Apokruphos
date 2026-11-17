# Asset naming convention

Draft 1 — 2026-11-16 (America/Chicago). Script-friendly lowercase `snake_case`. Companion to `sprite-tile-spec.md` and `shot-list.md`.

## Pattern

```
{category}_{subject}_{variant}_{action}_{dir}_{frame}_{mode}_{tier}.png
```

All tokens are lowercase ASCII, digits, and underscores only. No spaces, no hyphens, no camelCase, no leading zeros on anything except `frame`. The extension is `.png`.

### Tokens, in order

| Token | Required? | Vocabulary |
| --- | --- | --- |
| `category` | yes | `char`, `tile`, `token`, `banner`, `sky`, `prop`, `ui`, `item` |
| `subject` | yes | The thing drawn: `kaenyn`, `grass`, `starfield`, `keep`. A unique character name, a terrain material, or a sky pass. |
| `variant` | when it distinguishes the asset | Line, biome, outfit, or role: `audsen`, `meadow`, `void`, `formal`. |
| `action` | characters and other animated subjects | `idle`, and later `walk`, `attack`, and so on. Omit on still tiles, tokens, banners, and sky plates. |
| `dir` | when the asset has a facing | Default set: `se`, `sw`, `ne`, `nw`. Catalogue default is `se`. If 8-facing is adopted later, add `n`, `s`, `e`, `w`. Omit when there is no facing. |
| `frame` | yes, on any sequenced asset | Two-digit, zero-padded: `00`, `01`, `02`, `03`. A still tile uses `00`. |
| `mode` | when a lighting pass is authored | `day`, `night`. Future tokens, not in the first set: `golden`, `overcast`. Omit on characters that use only the core set in every mode. |
| `tier` | yes | Zoom tier tag: `t1`, `t2`, `t3`, `t4`. Matches the style guide's four authored scales. |

### Dropping unused tokens

Do not insert a placeholder such as `none` or `x`. Drop the unused token and keep the remaining order. A parser should read from the ends: the last token before the extension is always the tier (`t1`–`t4`); the token before that is `mode` if it is one of `day`, `night`, `golden`, `overcast`, or `core`; `frame` is the two-digit number; `dir` is a facing code; `action` is a known verb; `category` is the first token.

The atlas `id` is the filename without `.png`.

## First-set examples

Kaenyn Audsen, Tier-1 close idle, south-east rest frame, core palette (no mode token):

```
char_kaenyn_audsen_idle_se_00_t1.png
```

The same cycle, all four default facings, four frames each — sixteen files:

```
char_kaenyn_audsen_idle_se_00_t1.png
char_kaenyn_audsen_idle_se_01_t1.png
char_kaenyn_audsen_idle_se_02_t1.png
char_kaenyn_audsen_idle_se_03_t1.png
char_kaenyn_audsen_idle_sw_00_t1.png
char_kaenyn_audsen_idle_sw_01_t1.png
char_kaenyn_audsen_idle_sw_02_t1.png
char_kaenyn_audsen_idle_sw_03_t1.png
char_kaenyn_audsen_idle_ne_00_t1.png
char_kaenyn_audsen_idle_ne_01_t1.png
char_kaenyn_audsen_idle_ne_02_t1.png
char_kaenyn_audsen_idle_ne_03_t1.png
char_kaenyn_audsen_idle_nw_00_t1.png
char_kaenyn_audsen_idle_nw_01_t1.png
char_kaenyn_audsen_idle_nw_02_t1.png
char_kaenyn_audsen_idle_nw_03_t1.png
```

Isometric meadow-grass floor tile, still frame, day and night passes, Tier 1:

```
tile_grass_meadow_00_day_t1.png
tile_grass_meadow_00_night_t1.png
```

## Later examples (backlog only; do not cut these yet)

```
token_kaenyn_audsen_se_00_t3.png
banner_audsen_00_t4.png
sky_starfield_void_00_night_t1.png
sky_starfield_stars_00_night_t1.png
sky_starfield_constellation_00_night_t1.png
sky_starfield_omen_00_night_t1.png
tile_grass_meadow_00_day_t2.png
tile_granite_alpine_00_day_t1.png
tile_stone_ashlar_block_00_day_t1.png
```

A Line banner is a colour-tendency mark (Audsen maroon and gold, when that banner is cut), not a coat of arms. There is no heraldry. Do not put a crest, a shield device, or a "house" name in the filename.

## Sheets and metadata

A packed atlas is a power-of-two PNG named for its contents and tier, for example `varif_char_t1.png` or `varif_tile_t1.png`. The companion JSON uses the same stem: `varif_char_t1.json`. Individual source files keep the long name above, even when they are later packed.

## Item icons (added 2026-11-16)

Inventory and loot icons use the `item` category. The variant token is the icon size, `s32`, `s48`, or `s64`, and the tier is `t1`. Example: `item_niaviopti_s64_t1.png`. Mythic items live in `items/mythic/`. Upscaled previews and comparison sheets go in a `preview/` subfolder and are exempt from this convention.
