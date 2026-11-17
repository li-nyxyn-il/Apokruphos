# Shot list

Opened 2026-11-16 (America/Chicago). First set confirmed the same day: Kaenyn Audsen's Tier-1 idle sprite, and one isometric meadow-grass floor tile in day and night modes. Grid and canvas are in `sprite-tile-spec.md`. Filenames follow `naming-convention.md`.

Do not invent subjects. Kaenyn Audsen is the style guide's named example sprite. Audsen Line colour tendency is maroon and gold. There is no heraldry; backlog banners are Line colour marks, not coats of arms.

## First set — status `todo`

| id | asset | tier | canvas | frames | palette mode | status | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SL-001 | Kaenyn Audsen idle | 1 | 32×32 | 4 frames × 4 facings (`se`, `sw`, `ne`, `nw`), 200 ms each (default) | core | todo | Red-blood; fair skin; ice-blue natural eyes, not inverted; gold-ornamented armour; red paludamentum; Audsen maroon and gold from the core cloth and metal ramps. Pivot: feet, `(0.5, 0.0)` Y-up. Catalogue facing: `se`. Files: `char_kaenyn_audsen_idle_{dir}_{frame}_t1.png`. |
| SL-002 | Meadow grass floor tile | 1 | 32×16 diamond | 1 still frame (`00`) | day and night | todo | One footprint, two lighting passes: `tile_grass_meadow_00_day_t1.png` (core + daylight grass ramp) and `tile_grass_meadow_00_night_t1.png` (core + darker grass steps and void; no Day sky, no Grass Sun, no Blood red sky on the ground). Pivot: diamond centre, `(0.5, 0.5)` Y-up. |

Sixteen character files and two tile files complete this set.

## Backlog — not in the first set

These rows are implied by the style guide's zoom tiers, biome notes, and night-sky stack. They are listed so next week's review has something to measure against. Status stays `backlog` until a row is promoted.

| id | asset | tier | canvas | frames | palette mode | status | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SL-010 | Kaenyn Audsen tactical LOD | 2 | 16×16 | 4 facings, frame count unset | core | backlog | Silhouette, helm or hair mass, cloak colour. Eyes may collapse to one pixel or disappear. A new asset, not a filtered shrink of SL-001. |
| SL-011 | Unit token, Audsen | 3 | 16×16 | still, facing unset | core | backlog | Helm, cloak, shield; caste tint on the body; no face. Line colour as a contiguous block, not a crest. |
| SL-012 | Line banner or strategic icon, Audsen | 4 | 8×8 or 16×16 (default still open) | still | core | backlog | Colour-tendency mark in maroon and gold. No body, no coat of arms, no heraldry. Must read at about 16–24 CSS pixels on the phone floor. |
| SL-013 | Red-star sky layer | 1 (flatten at 3–4) | plate or sparse sheet, size unset | still plates; constellations swap from night to night | night (core + night / celestia) | backlog | Separate layer, not painted into the terrain. Style-guide stack: (1) void field, (2) red star field, (3) constellation overlay using only groups already catalogued (the Dragon, the Lamb, the Fox, the Hare, the Raven, the Crow, Creation's Chisel, the Blood Sage), (4) moon / omen pass, used sparingly. Default stars are Signal Red and Star Red, not gold, not white, not cool blue. |
| SL-014 | Further meadow terrain | 1 | 32×16 floor; blocks 32×32 for 1-step | still | day and night | backlog | Straw edges, rare fireweed accent, water / lake tile from the meadow notes. A 1-step block if a lip or low wall is needed. |
| SL-015 | Other biome floor tiles | 1 | 32×16 diamond | still | day and night | backlog | One tileset per style-guide biome, sharing core stone: alpine granite, conifer valley, desert canyon, Appalachian ridge. |
| SL-016 | Shared architecture | 1 | 32×16 floor; 32×(16+16n) walls | still | day and night | backlog | Roads, keeps, cut ashlar. Cold Ashlar, Cliff Teal, and the warm-stone ramp in every biome, so a Line keep does not change hue family when the map tiles do. Hexagonal Krypts stay later, as in the style guide. |

Promote a backlog row into a dated first-set (or next-set) table when it is scheduled to be drawn. Do not add named characters, Lines, or constellations that are not already in the style guide.
