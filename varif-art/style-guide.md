# Varif art style guide

Draft 1 — engine-neutral, handmade pixel art.

This is a production bible for a human artist working in Photoshop or a similar editor. The eleven concept paintings are look-dev, not sources to trace, filter, or run through an image model. All game art is drawn by hand on the locked palette.

Locked colour set: `palette.md`, `palette.png`, `palette.hex`, `palette.gpl`, `palette.aco`, `palette.ase`.

## Art pillars

1. **Scale contrast.** People are small under stone. Robed figures at the foot of colossal pillars, a single rider under a sky of red stars, a valley that could swallow a column: this is the world's first impression.
2. **Low-key gold against black.** Most of the frame lives in the void ramps. Gold, ivory, and torchlight are small, hard, and precious. Chiaroscuro is the default, not a special case.
3. **The red celestia.** Stars are red, and they move from night to night into new constellations. The night sky is a character, built as its own layer, never a generic blue-black starfield.
4. **Roman dress in a harsh mountain world.** Lorica, paludamentum, laurel, and cut stone — not generic medieval Europe. The clothes are imperial; the land is mountains and valleys.
5. **Blood made visible at every zoom.** Red-blood, blue-blood, and black-blood must read as skin (and, at a distance, as a colour block or banner) from a close character view down to a grand-strategy map, including on a phone screen.

## Camera and zoom tiers

The camera is a seamless, isometric-style view that mixes three-quarter characters with a more top-down map, and that zooms between close RPG play and grand strategy. Art is **authored at four fixed tiers**. The engine may interpolate the camera between them; do not author a fifth unofficial scale. At rest, snap to an integer pixel scale (see [Readability floor](#readability-floor-phone-and-browser)).

Technical defaults below are **changeable** and have not been confirmed: **16×16 tiles** and a **32×32 character canvas**. For a 2:1 isometric floor, a 32×16 (or 32×32 diamond) tile is the usual alternative — confirm before a first tileset is cut. Until then, treat 16×16 as the floor grid and 32×32 as a character that stands two tiles tall.

| Tier | Working name | Camera feel | Character LOD | Terrain LOD | What must still read |
| --- | --- | --- | --- | --- | --- |
| 1 | **Close / RPG** | Three-quarter isometric. A character is large enough to show a face. | Full **32×32** sprite (default): cloth folds, gold trim, weapon, hair or helm, inverted eyes. | **16×16** tiles with full material ramps and props. | Caste skin, eye treatment, Line colours on cloak or jewellery, silhouette. |
| 2 | **Tactical / party** | Still three-quarter, pulled back over a street, camp, or skirmish. | Same 32×32 asset nearest-neighbour scaled, or a **16×16 LOD**: silhouette, helm or hair mass, cloak colour. Eyes may collapse to one iris pixel or disappear. | 16×16 tiles, fewer props, broader material blocks. | Caste as a colour block, Line colour, unit role (soldier, rider, civilian). |
| 3 | **Operational / province** | Flatter isometric, almost top-down. Valleys and roads become the subject. | **16×16 unit token**: helm, cloak, shield; a caste tint on the body. No face. | Chunked terrain, roads, keeps, river and ridge masses. | Token type, caste tint, a fragment of Line colour. |
| 4 | **Strategic / map** | Grand-strategy map of the Neck (or a theatre of it). Shallow 2:1 or flattened iso. | **8×8 or 16×16 banner or icon.** No body. | Biome fills, mountain glyphs, city and keep marks, roads as strokes. | Line heraldry, faction or caste colour, terrain mass (mountain, valley, water, settlement). |

Switch LOD at the tier thresholds. A short crossfade is fine during a seamless zoom. Do not bilinear-filter the sprites on the way through.

**How the guide adapts if the projection is revised.** Side-on would turn Close/Tactical into profile or three-quarter side sprites and would drop diamond tiles for a stacked side-view depth sort. A true top-down would flatten characters into tokens earlier (closer to Tier 3). A locked 2:1 isometric is the assumption this draft is written for.

## Readability floor (phone and browser)

The game is **desktop first**, and may also ship on **phone and in a browser**. Author every tier so it still reads on a **small phone in portrait** — take **360 CSS pixels wide** as the floor viewport. Desktop may show more tiles and a looser crop; it must not be the only place a fact is visible.

- **Integer scale at rest.** Prefer 2× or 3× on the floor phone. Letterbox or pillarbox to the next integer scale rather than resting on a fractional zoom. The same rule holds in a browser: do not CSS-`scale` to 1.37.
- **Close tier, minimum character size.** A 32×32 character should not drop below about **64 CSS pixels** tall on the floor phone (2×). Prefer 3× when the layout allows. If a 32×32 canvas cannot hold a feature at that size, the feature does not belong at that tier.
- **Strategic tier, minimum mark size.** A banner or city icon must remain identifiable at about **16–24 CSS pixels** on the long edge (an 8×8 mark at 2–3×). Caste or Line colour is a contiguous block of at least **2×2 source pixels**, never a single isolated pixel.
- **No critical one-pixel secrets.** Caste, Line, unit type, selection, and wounded state must survive a phone crop and a browser downsample to the floor scale. Inverted eyes are a Close-tier feature; they are allowed to vanish at Tactical and beyond, but the caste colour must not.
- **Contrast.** UI text and frames use Ivory (`#EBEADC`) on Void Ink (`#07080E`) or Lace Black. Mid-grey on black is not a UI pairing.
- **Test crop.** Before locking a tileset, sprite, token, or banner, view it at 2× in a 360-pixel-wide frame. If the caste or the Line is gone, it fails the floor.

## Mood and lighting

**Key light, production default:** upper-left, about 10–11 o'clock, 30–45° elevation. Tile bevels and face planes stay consistent across the atlas. If a scene is deliberately flipped (a west-facing cliff at dusk, a shrine lit from a right-hand fire), note the exception on the asset.

**Low-key chiaroscuro is the default.** Most pixels should sit in the bottom two or three values of the active ramp. Highlights are small: a gold click, a lamp core, a cheek plane, a sun-struck pillar edge. The Elyssa portrait and the Shjana portrait are the lighting references for characters; the canyon pillars and the lake silhouette are the references for space.

**Light families, do not mix in one sprite:**

- Night exterior: void ramps + blood-red sky + cool stone. Bounce may be a faint Dried Blood or Cliff Teal, not a second key.
- Day or overcast: warm or cold stone, Ivory atmosphere (see Autumn Sky), gold only on metal and sun edges.
- Interiors and rites: the torchlight ramp. Shadows go to Hearth Dark, not Void Ink, so the hue family stays warm.

**Hue-shifted shading.** Shadows are not a darker version of the mid painted in grey. Night shadows go cool (Cliff Teal, Void Ink). Torch shadows go to Hearth Dark and Cloak Shade. Daytime flesh shadows go to Skin Shade, slightly red, not a cold grey. A mid that cannot reach its shade in three steps or fewer needs a ramp review, not a new colour.

## From the paintings to pixels

The references are oil- and digital-paintings: soft edges, huge value ranges, photographic pores. Pixel art on this project is the opposite discipline.

- **Limited ramps.** Three to five steps per material, taken only from the locked palette. A cloak is Cloth reds. A keep wall is Cold stone or Warm stone, not both in one tile unless a corner actually turns the light.
- **Selective outline.** Close-tier characters may use a cluster outline in Warm Black or Lace Black, or a shade from the same material, not a cartoon ring of Void Ink around every form. Terrain uses outlines only at a cliff break or a tile lip that must read at Tactical zoom. Tokens and banners may use a one-pixel dark rim so they survive the phone floor.
- **Dithering is sparse.** A few pixels at a ramp boundary, or a light dither in a distant red-sky or haze gradient, are allowed. No checkerboard fills, no dither as a substitute for a missing mid.
- **Nearest neighbour only.** No anti-aliased edges, no smudged gold, no filtered downscales. See [Photoshop setup](#photoshop-pixel-art-setup).
- **Silhouette first.** Read the character or token in one-bit before spending the gold trim. If the mass fails without colour, it will fail at Tier 2 and on a phone.

## Photoshop pixel-art setup

Handmade in Photoshop or a close equivalent (Clip Studio, Krita, Aseprite). This is the intended setup:

- **Document.** New file in **pixels**, 72 ppi is fine, with an exact canvas (16×16, 32×32, or a power-of-two atlas). Do not start from a print preset or a 300 ppi sheet.
- **Swatches.** Load `palette.aco` or `palette.ase` into the Swatches panel (panel menu → Load Swatches… or Replace Swatches…). Keep a dedicated Varif set. Pick only from it.
- **Pencil, not Brush.** Use the Pencil tool at 1 px, 100% opacity and hardness. The Brush tool anti-aliases and will dirty the palette.
- **No anti-aliasing.** Turn anti-alias off on the Pencil, Eraser (block / pencil mode), Marquee, Lasso, Magic Wand, Elliptical and Custom Shape tools, and on type if you set any bitmap labels. Do not use Smooth, Align Edges, or stroke with anti-aliased brushes.
- **Nearest-neighbour resampling.** Set image interpolation to **Nearest Neighbor** in Preferences, and again whenever you use Image Size or Free Transform. Never resample a sprite with Bicubic or Bilinear. Judge pixels at 100%, 200%, 400%, or 800% zoom only.
- **Snap and grid.** Turn on a 1 px grid (or a 16 px grid with 1 px subdivisions), enable Snap to Grid / Snap to Pixels, and keep the pixel grid visible at high zoom.
- **Indexed colour against the palette.** When a sprite or a tile sheet is finished in RGB, run **Image → Mode → Indexed Color**. Palette: Custom (load the `.aco`, or an ACT exported from it). **Dither: None.** Forced: None, unless you have a deliberate reason to force black and white. Confirm the colour table is only the 48 locked colours, then save. Working in RGB with the swatch panel is fine day to day; index as a check before a sheet is committed, so no stray off-palette pixel leaves the file.
- **Export.** PNG, sharp pixels, no baked preview filtering. PNG-8 indexed or PNG-24 that still contains only palette colours. Do not pre-scale to screen size in the file; the engine scales with point sampling.

Aseprite or a similar specialist tool is compatible with this guide (import `palette.gpl` or `palette.ase`, nearest-neighbour, no anti-alias). The same rules apply.

## Character rules

**Dress and armour.** Roman-inspired: muscled or segmented cuirass, pteruges or a short tunic, paludamentum (the red cloak) for nobles and officers, laurel or a decorated helm, caligae or simple boots, gold ornament on dark metal. Civilians: tunics, stolas, dark lace and ivory linen for a high-born mourning or formal look (Meri, Elyssa). This is not knightly plate and not a generic hooded-fantasy kit. The autumn knight in `Autumn Sky.png` is a mood and foliage reference; his dark plate is allowed for a field captain, but the Line's formal look is the Kaenyn / Vakirros gold-and-cloak language.

**Silhouette.** Broad shoulders and a cloak fold should read at Tier 2. Riders need a horse-and-rider mass that does not collapse into a blob at Tier 3. Hair and helm are the head's identifier once the face is gone.

**Blood castes at sprite scale.**

| Caste | Close (32×32 default) | Tactical / operational | Strategic |
| --- | --- | --- | --- |
| **Red-blood** | Red-blood skin ramp. Ordinary human complexion. Price "blood eyes" may use Cloak Mid or Iris Amber. | Skin mid as a small face or hand block; otherwise cloak and hair do the work. | No body. Banner or icon in Line colours. |
| **Blue-blood** | Blue-blood sky ramp. Skin is **sky blue**, a "royal hue" and "crystal blue" in sun (Blue Crystal is a few pixels only). **No visible blue veins on the face.** Default irises may be Iris Amber (as in the Kheinyn concept) or, for southern-isle characters, a later purple — do not invent a purple into the locked set without a review. Inverted eyes are common on true blue-bloods (see below). | A contiguous sky-blue block on the token, large enough for the phone floor. | Sky-blue or Deep Blue Blood as a mark on the banner or icon, plus Line heraldry. |
| **Black-blood** | Black-blood silver ramp. Skin reads as **metal**, not grey paint: cool silver in shadow, Warm Silver or a gold click in the light. Default irises are Iris Amber, or Cloak Mid mixed with Star Gold for blood-orange. | A silver or warm-silver block on the token. | Silver or bronze mark plus Line (or Kamloran) colour. |

Cite: user brief; `A Pylgrym's Plight` (Halion's bath: sky-coloured / crystal-blue skin, inverted eyes; Samule's travel story: southern "skin the color of the sea," purple eyes, dusk-coloured blood, and far-western black-bloods whose skin "shined like metal and was the color of bronze and gold"); `A Pylgrym's Plight — Collated Whiteboard Notes`, section "The blood colours" (Kamlorans / Myhjodi: silvered skin, blood-orange eyes; Prices' "blood eyes"; Brienwyn's purple eyes). Default sprite recipes follow the brief (sky-blue skin, deep blue blood, metallic silver skin). Regional variants are noted in `palette.md` and must not overwrite the default ramp.

**Inverted / meha nia / cold-flame eyes.** Black (inky) whites, a coloured iris, a pale cloudy pupil. The manuscript description is the authority: whites "inky and dark," irises that "shimmered a bright yellow-gold," pupils "pale and glossy, as if plucked from eyes of a marble statue" (`A Pylgrym's Plight`, Halion's bath). At 32×32, budget two to four pixels: Void Ink or Lace Black for the sclera, Iris Amber (or the character's iris colour) for the iris, Cloudy Pupil if there is a pixel to spare. If there is not, drop the pupil and keep black sclera plus one iris pixel. At 16×16, one iris pixel on a dark socket is enough. At map scale, omit the eyes.

Noble families are **Lines**, never Houses. Heraldry lives on cloaks, shields, belt ornaments, and banners. Name the Line on the asset, not a "house crest."

## Environment rules

Mountains and valleys are a central theme, not a backdrop. Give ridges a cool-stone mass and a hard lip; give valleys a broader, warmer or greener floor once a green pass exists (it is not locked yet — autumn ramps stand in for dry grass and harvest). The canyon pillars painting is the architecture reference: colossal, dressed stone, a little sand, a lot of air.

**Scale contrast.** A Close scene should still be able to show a figure against something that dwarfs them (a keep door, a cliff, a pillar base). At Operational and Strategic tiers, the land is the character: ridge, pass, lake, and road do more work than any token.

**Parallax.** Build in layers, back to front, for example: night-sky (see below), far mountain silhouette, mid valley or canyon wall, playfield tiles, props and tokens, close weather or foreground grass. Far layers use fewer ramps and no dither except a light sky grain. Do not put gameplay collision in a parallax layer.

**Hexagonal Krypts** appear as a later architectural type (whiteboard notes, summary of *causal knot theory*): hexagonal temples / engines. They are not in the first tileset, but leave room in the language of cut stone for a hexagonal plan.

## The night sky

Build the celestia as a **separate layer** (or a small stack of layers), not painted into the terrain.

Suggested stack:

1. **Void field** — Void Ink / Warm Black, possibly a very slow nebula of Ember Void and Dried Blood (the horseman painting).
2. **Star field** — a sparse sprite sheet or particle set of Signal Red and Star Red, with a few Dimmer Dried Blood points. Default stars are **red**, never white, never cool blue-white.
3. **Constellation overlay** — named groups (the Dragon, the Lamb, the Fox, the Hare, the Raven, the Crow, Creation's Chisel, the Blood Sage, and others as they are catalogued) as their own sprites, so they can be moved, reshaped, or swapped **from night to night**. The manuscript is explicit: stars mix and move, and constellations take on different forms and meanings, "just as the moon changes shape" (`A Pylgrym's Plight`, Brienwyn to the nameless man). The whiteboard also records the red-shift of the stars as a cosmological fact (`Collated Whiteboard Notes`, B6 and the *causal knot theory* summary): "why are the stars red?"
4. **Moon / omen pass** — a brazen or pale moon, used sparingly, on its own sprite.

The lake painting (`worlds splash.png`) shows **gold** stars. That is not the default celestia. Treat Star Gold as a rare atmospheric or regional pass (lamplight reflected, a brazen moon, a specific omen), never as a replacement for the red stars.

At Strategic zoom the sky can flatten to a red-tinted void and a few constellation marks; at Close zoom the nebula and individual stars may show.

## UI framing

Frame the playfield like a dark shrine or a tablet: Void Ink or Lace Black chrome, thin Antique Bronze / Ornament Gold rules, Ivory text. Do not pour Cloak Bright or Star Red into panels except as a small alert or a selected-Line flash.

On the phone floor, a frame that eats more than a modest margin will fail; keep chrome thin and contrast high. Line colours, not "house" labels, mark the player's allegiance. A suggested religious motif, if one is needed later, is the Serpent of Time — a snake knotted seven times, eating its tail, an eye in each loop (`Collated Whiteboard Notes`, Plot Notes / Hesseh's lesson). It is not a first-pass HUD requirement.

## Engine notes

The simulation prototype is data-driven in Rust. The render engine is **undecided between Unity and Bevy**. There are no engine assets yet. Keep sheets and metadata engine-neutral; both paths below assume **point sampling** and the same atlas JSON.

**Common rules, either engine.**

- Point / nearest filtering only. No texture compression on sprites or tiles (no crunch, no DXT/ETC/ASTC on the art; keep lossless or uncompressed RGBA).
- Disable interpolated mipmaps, or do not generate them, for pixel sheets.
- At-rest camera zoom is an integer scale so one source pixel maps to an integer number of screen pixels. The phone floor in this guide is the minimum for that test.
- Pivots and rects live in a script-friendly **JSON atlas** (see below), not only inside an editor binary.

**Unity.**

- **Pixels Per Unit:** default **16**, so one 16×16 tile is 1 world unit and a 32×32 character is 2 units tall. If the tile size changes, change PPU to match the tile, not the character.
- Sprite **Filter Mode: Point**. Compression: **None**. Mip maps off for sprite atlases.
- Pixel Snap or a camera script that keeps the camera on a pixel grid at rest. During a seamless zoom you will leave that grid; still sample nearest, and snap again when the zoom settles.

**Bevy.**

- Install nearest sampling globally for this art with `ImagePlugin::default_nearest()` (or an equivalent nearest sampler on the sprite and tile materials).
- Bevy 2D is **Y-up**. Store pivots in Y-up space in the JSON so a bottom-centre character pivot is `(0.5, 0.0)` from the bottom-left of the sprite. Editors that are Y-down (Photoshop's pixel origin, Aseprite) must flip Y when they export metadata, not when the engine reads it.
- Same integer-scale camera rule as Unity.

**Atlas metadata (recommended JSON).** A single format both a Unity importer and a Bevy asset loader can read. Example shape, not a frozen schema:

```json
{
  "name": "varif-characters",
  "texture": "varif-characters.png",
  "filter": "nearest",
  "origin": "y_up",
  "tile_size": [16, 16],
  "sprites": [
    {
      "id": "kaenyn_idle_se",
      "rect": [0, 0, 32, 32],
      "pivot": [0.5, 0.0],
      "tier": 1,
      "line": "Audsen",
      "caste": "red-blood"
    }
  ]
}
```

`rect` is `[x, y, w, h]` in texture pixels, with `y` measured from the origin declared in `origin`. Tokens and banners are further sprites with `"tier": 3` or `"tier": 4`. Do not pack engine-specific material data into this file.

## Technical defaults (changeable)

These are provisional and should be confirmed before a first atlas is cut:

- Floor tile: **16×16**. Iso alternative to confirm: 32×16 or 32×32 diamond at 2:1.
- Character canvas: **32×32**, two tiles tall at the default tile size.
- Unit token: **16×16**.
- Banner / strategic icon: **8×8** or **16×16**.
- Projection: 2:1 isometric, three-quarter characters at Tiers 1–2.
- Palette: 48 colours, locked, see `palette.md`.

## Reference list

The eleven concept paintings, with Drive filenames from `Varif_Project_Files / Pictures / Portraits`. They were matched by file size and by content after the original chat attachments did not all arrive on disk. Use them as lighting, costume, and landscape references. Do not trace or index them into sprites.

1. **`runaway 1.webp`** — A horseman, a cloak, a horse, all in silhouette, under a red nebula packed with red stars. *Take:* the default night sky, scale of a single rider, black-on-red contrast. Palette source for the blood-red sky ramp.
2. **`throne 1.webp`** — Two tiny robed figures at the foot of colossal dressed-stone pillars in a canyon of sand and cool rock. *Take:* monumental architecture, scale contrast, the warm-stone and cold-stone split, a single shaft of sun.
3. **`rites 1.webp`** — A torchlit oath, warm orange, candle and cloak, a ring of witnesses. *Take:* interior light family, ceremony staging, paludamentum and linen in firelight. (`rites 2.webp`, a related courtyard oath, is on Drive but was not one of the eleven.)
4. **`Kaenyn Audsen.png`** — Close Roman portrait: gold ornamented armour, red paludamentum, fair red-blood skin, ice-blue natural eyes. *Take:* formal Line dress, cloth-red and gold language, red-blood highlight.
5. **`Meri Audsen.png`** — White-haired woman in black lace, a child in ivory linen. *Take:* mourning or formal civilian dress, lace black, white-hair ramp, intimate low-key portrait.
6. **`Autumn Sky.png`** — A dark-armoured man with two children, autumn leaves, a pale overcast. *Take:* foliage ramp, Ivory atmosphere, field-armour silhouette, family scale against a quiet landscape.
7. **`Elyssa Dane.png`** — Chiaroscuro woman in a white wrap, gold jewellery, a black-and-gold mantle. *Take:* the lighting model for Close-tier faces, ivory-and-gold against void, red-blood skin steps.
8. **`worlds splash.png`** — A silhouetted figure by a lake under a sky of gold stars, a small warm lamp. *Take:* water and reflection, foreground silhouette, stillness. Gold stars are a special pass, not the default celestia.
9. **`Shjana.png`** — Metallic silver skin, a gold laurel helm, a dark veil. The black-blood reference. *Take:* metal-flesh ramps, laurel and gold in shadow, low-key portrait lighting.
10. **`Kheinyn Belravi.png`** — Blue-grey / teal skin, amber eyes, gold cloak on dark armour. *Take:* cool flesh in shadow, amber iris, a night-lit blue-blood. His measured greys informed the cold-stone ramp and, hue-shifted, the sky-blue caste ramp; do not copy the grey as the default blue-blood body colour.
11. **`Vakirros Grymsyhl.png`** — Imperial gold crown and armour, a red cloak, a sword, amber eyes, harsh low-key light. *Take:* a second formal-dress reference, heavier and more warlike than Kaenyn; gold against void.

## Sources and conflicts

Visual facts taken from Drive (read only; nothing on Drive was edited) are marked with their document names above. The folder is **`Varif_Project_Files`**.

**Used.**

- `A Pylgrym's Plight` (manuscript): Lines, not Houses; sky-blue / crystal-blue skin and inverted eyes; Samule on southern blue-bloods and metallic bronze-gold black-bloods; silver-skinned men on the Ptormyk docks; moving constellations and red "Creation's Chisel"; named star-groups (Dragon, Lamb, Fox, Hare, Raven, Crow, Blood Sage).
- `A Pylgrym's Plight — Collated Whiteboard Notes`: blood-colour table (silvered Kamloran / Myhjodi skin, blood-orange eyes, Price blood eyes, purple eyes, dusk-coloured southern blood); Krypts and the red-shift of the stars; ecology of the High and Low Valleys; hexagonal Krypts (*causal knot theory* summary); Serpent of Time; atzhelm as "land of the valley."
- `Reworks.txt`: plot notes only; no additional visual facts used.
- `Pictures / Portraits` (and the Variff subfolder): the eleven references above.

**Noted, not treated as sprite law.**

- Southern-isle "sea" skin, purple irises, and dusk-indigo blood (manuscript / whiteboard) are regional variants. Default blue-blood art is sky-blue skin and Deep Blue Blood.
- Far-western black-bloods the colour of bronze and gold (manuscript) are a regional metal variant. Default black-blood art is silver, per the brief and Shjana.
- Book 2 board title "House Dane" in the whiteboard is a board slip; the manuscript and this guide use **Line**.

**Could not be used, or was left alone.**

- `dictionary.txt` is cited in the whiteboard and was not found on Drive.
- `theAncientPainting` (in `The_Manifold`) was not edited and was not used as a visual bible; the whiteboard already summarises *That Ancient Painting* as a parable, not as an art guide.
- `Variff.zip` / `Variff complete.png` and other large archives were not unpacked.
- `family trees.csv` was not needed for visual facts.
- The original eleven chat attachments did not all materialise on disk; the same pictures were recovered from Drive by size and content. `rites 2.webp` was inspected and is not in the locked reference set.

This draft does not assume a final tile size, character canvas, or engine. Confirm the changeable defaults before cutting the first atlas.
