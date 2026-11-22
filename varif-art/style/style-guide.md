# Varif art style guide

Draft 2 — 2026-11-15 (America/Chicago), tile grid updated 2026-11-16, palette moved to v2.1 on 2026-11-21. Engine-neutral, handmade pixel art.

This is a production bible for a human artist working in Photoshop or a similar editor. The concept paintings and the landscape photographs are look-dev, not sources to trace, filter, or run through an image model. All game art is drawn by hand on the locked palette.

Locked colour set: **`palette-v2.1.md` (v2.1, 91 colours)**, `palette-v2.1.png`, `palette-v2.1.hex`, `palette-v2.1.gpl`, `palette-v2.1.aco`, `palette-v2.1.ase`. v2.1 is the 79 v2 colours, unchanged, plus 12 measured colours for the Price Line, Ptormyk, Niaviopti, and southern Maphlia / old Yhlgori (see [Palette change log](#palette-change-log)). v2 stays beside it as `palette.md` and `palette.*`. The v1 set and this guide's previous draft are archived as `palette-v1.*` and `style-guide-v1.md`.

## Art pillars

1. **Scale contrast.** People are small under stone. Robed figures at the foot of colossal pillars, a single rider under a sky of red stars, a valley that could swallow a column: this is the world's first impression.
2. **Two lighting worlds, one core.** Night is still low-key gold against black: most of a night frame lives in the void ramps, and gold, ivory, and torchlight are small, hard, and precious. Day is daylight grandeur — open sky, sunlit grass, long views — without abandoning the same metals, cloth, and caste colours. Do not paint a day valley in the blood-red celestia, and do not fill a night sky with Zenith Blue.
3. **Aerial perspective.** Layered ridges fade one ramp step at a time toward Distance Blue and Horizon Veil. Depth is a colour rule, not a blur. This is the Blue Ridge, the winter Appalachian valley, and the Hudson River School / Bierstadt / Cole light: air as a stack of hues.
4. **The red celestia.** Stars are red, and they move from night to night into new constellations. The night sky is a character, built as its own layer, never a generic blue-black starfield.
5. **Roman dress in a harsh mountain world.** Lorica, paludamentum, laurel, and cut stone — not generic medieval Europe. The clothes are imperial; the land is mountains, valleys, meadows, and canyons.
6. **Blood made visible at every zoom.** Red-blood, blue-blood, and black-blood must read as skin (and, at a distance, as a colour block or banner) from a close character view down to a grand-strategy map, including on a phone screen.

## Camera and zoom tiers

The camera is a seamless, isometric-style view that mixes three-quarter characters with a more top-down map, and that zooms between close RPG play and grand strategy. Art is **authored at four fixed tiers**. The engine may interpolate the camera between them; do not author a fifth unofficial scale. At rest, snap to an integer pixel scale (see [Readability floor](#readability-floor-phone-and-browser)).

The floor grid is a **2:1 isometric 32×16 diamond** (confirmed 2026-11-16). The character canvas remains **32×32**, two tile-heights tall. Details, pivots, and atlas metadata live in `sprite-tile-spec.md`. The first assets to cut are listed in `shot-list.md`.

| Tier | Working name | Camera feel | Character LOD | Terrain LOD | What must still read |
| --- | --- | --- | --- | --- | --- |
| 1 | **Close / RPG** | Three-quarter isometric. A character is large enough to show a face. | Full **32×32** sprite (default): cloth folds, gold trim, weapon, hair or helm, inverted eyes. | **32×16** diamond tiles with full material ramps and props. | Caste skin, eye treatment, Line colours on cloak or jewellery, silhouette. |
| 2 | **Tactical / party** | Still three-quarter, pulled back over a street, camp, or skirmish. | Same 32×32 asset nearest-neighbour scaled, or a **16×16 LOD**: silhouette, helm or hair mass, cloak colour. Eyes may collapse to one iris pixel or disappear. | 32×16 diamond tiles, fewer props, broader material blocks. Atmospheric layers start to tell. | Caste as a colour block, Line colour, unit role (soldier, rider, civilian). |
| 3 | **Operational / province** | Flatter isometric, almost top-down. Valleys and roads become the subject. | **16×16 unit token**: helm, cloak, shield; a caste tint on the body. No face. | Chunked terrain, roads, keeps, river and ridge masses. Distance blues do real work. | Token type, caste tint, a fragment of Line colour, biome mass. |
| 4 | **Strategic / map** | Grand-strategy map of the Neck (or a theatre of it). Shallow 2:1 or flattened iso. | **8×8 or 16×16 banner or icon.** No body. | Biome fills, mountain glyphs, city and keep marks, roads as strokes, a simplified aerial-perspective stack. | Line heraldry, faction or caste colour, terrain mass (mountain, valley, water, meadow, desert, settlement). |

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

**Hue-shifted shading.** Shadows are not a darker version of the mid painted in grey. Night shadows go cool (Cliff Teal, Void Ink). Torch shadows go to Hearth Dark and Cloak Shade. Daytime flesh shadows go to Skin Shade, slightly red, not a cold grey. Daytime foliage shadows go to Forest Shade or Grass Shade, not Charcoal. A mid that cannot reach its shade in three steps or fewer needs a ramp review, not a new colour.

## Time of day and lighting modes

Pick **one** lighting mode per scene (and per sprite, except for a deliberate mixed-light interior). The core ramps — void, gold, torch, caste, cloth, linen, shared stone — are available in every mode. The night-sky ramp and the daylight landscape ramps are not interchangeable.

### Day

Clear or lightly clouded daylight. The heavens use **Day sky blue**: Zenith Blue at the top of the dome, Sky Mid and Sky Light lower, Sky Pale for thin cloud and the brightest atmosphere. The land uses the biome ramps at full contrast. Sun edges are small and hard: Grass Sun, Lime Pale, Highlight Gold, Snow Lit — a click, not a wash. Far ridges follow the [atmospheric-perspective](#atmospheric-perspective) rule, cool and blue.

References: the Yellowstone meadow, the sunlit clearing, the lake with fireweed, the waterfall, the Big Bend cliff and canyon mouth, the juniper-and-sandstone cliff.

### Golden hour

Low sun, long shadows, gold in the air. Shadows go to Forest Shade, Sunset Shade, or Canyon Shade, not Void Ink. Near the sun, the sky and lit stone shift toward Sunset Gold, Sunset Orange, and Straw Sun. The opposite sky may stay at Sky Mid or Sky Pale. Far ridges may take the two farthest atmospheric steps *tinted* toward Sunset Gold and Straw Sun rather than Horizon Veil. Keep Sunset Orange rare — a sun-disk fringe, a backlit leaf, a fire-edge.

References: the alpine sunset over granite, the Blue Ridge layered-ridge sunset, the backlit orange forest, Cole's gold light on the crag, the cropped Bierstadt.

### Overcast

Soft, even light, little or no hard gold click. The sky is Ivory, Pale Silver, and Mid Ridge, not Zenith Blue. Greens stay in Needle Mid and Grass Mid; Grass Sun and Lime Pale almost disappear. Granite and straw do more work than gold. This is the winter Appalachian valley and the Rocky Mountain valley under grey cloud.

References: the overcast Rocky valley, the storm-cloud forested valley, the winter Appalachian tan fields and blue ridges, the granite cirque under a mixed sky.

### Night under the red stars

The v1 low-key rule still governs. Most pixels sit in the bottom two or three values of the void ramps. The heavens are the **Blood red sky** ramp only. Default stars are Signal Red and Star Red, never white, never Sky Pale, never Horizon Veil. Far land goes to Void Ink, Cliff Teal, Charcoal, or Ember Void — not Distance Blue. Torch interiors use the torchlight ramp; their shadows go to Hearth Dark, not Void Ink, so the hue family stays warm.

References: `runaway 1.webp`, `worlds splash.png` (gold stars are a special pass, not the default), the night portraits, and the Blue Ridge silhouette as a dusk-into-night ridge.

## Atmospheric perspective

Build depth as a **stack of colour steps**, one step per parallax layer. Do not simulate distance with a blur, a transparent wash, or a new ad-hoc blue.

**Default day stack, back to front:**

| Layer | Role | Ramp step |
| --- | --- | --- |
| 0 | Sky dome | Day sky blue (Zenith Blue → Sky Pale, top to horizon) |
| 1 | Farthest ridge / horizon | **Horizon Veil** `#85B5CE` |
| 2 | Far ridge | **Distance Blue** `#61829D` |
| 3 | Mid ridge | **Mid Ridge** `#555862` |
| 4 | Near mass (far edge of the playfield, close treeline) | **Near Mass** `#3D3B44` |
| 5 | Playfield | Full biome ramps (granite, conifer, grass, straw, stone, water) |
| 6 | Foreground / close props | Same ramps, slightly darker (Forest Shade, Canyon Shade, Granite Shade); full contrast |

Do not skip a step — a playfield tile should not jump to Horizon Veil. At Operational and Strategic zoom the stack may flatten, but the order stays the same: farther is cooler and lighter, nearer is darker and more olive or tan.

**Golden hour.** Layers 1–2 may shift toward Sunset Gold and Straw Sun. Layers 3–4 stay closer to Mid Ridge and Near Mass, or take a little Sunset Shade. The playfield warms; it does not become a second sunset wash.

**Overcast.** Layers 1–2 may sit between Horizon Veil, Pale Silver, and Ivory. Contrast across the stack is shallower.

**Night.** Do not use the haze ramp. Far silhouettes are Void Ink, Charcoal, Cliff Teal, or Ember Void against the blood-red sky.

This is also the language of the Cole and Bierstadt references: air as layered light, a bright distant cleft, a darker near shore.

## Biome notes

Mountains and valleys remain the thematic centre, but they are no longer a single brown-and-teal stand-in. Author tilesets per biome, then share the atmospheric stack and the core stone ramps.

**Alpine granite.** Granite Shade / Mid / Lit, Snow Mid / Snow Lit on the high faces and cirque floor, Water Deep and Lake or River Teal in a tarn, Needle Dark in krummholz patches, Sky Mid or an overcast Ivory dome. The cirque with snow and the alpine sunset are the references. Keep snow as a cool, slightly blue-grey — not Ivory unless it is a specular click.

**Conifer valley.** Forest Shade interiors, Needle Dark / Mid / Lit for spruce and fir, trail beds in Straw Shade or Granite, Grass Mid in the clearings. The overcast Rocky valley, the forest trail, the storm-cloud valley, and the waterfall boulder field are the references. Dead standing pines may use Charcoal and Granite Shade, not a green.

**Meadow.** Grass Shade / Mid / Lit / Sun as the floor, straw at the dry edges, Fireweed or Fireweed Mid as a rare vertical accent (late-season dusty rose in these photos, not peak magenta). A lake uses the water ramp under Sky Light / Sky Pale. The Yellowstone meadow, the sunlit clearing, and the fireweed lake are the references.

**Desert canyon.** Canyon Shade, Sandstone, Sun Stone, then Lime Lit and Lime Pale on the hottest limestone. Straw through the wash and the trail. Sparse Needle Lit or juniper-dark against the wall. The sky is often the deepest Zenith Blue — this is the Big Bend pair (cliff and canyon mouth), the canyon trail, the prickly-pear rock, and the golden tamarisk. Prickly pear pads may use Needle Mid with a Straw or Lime edge; do not invent a cactus green.

**Appalachian ridge.** The atmospheric ramp *is* the subject: stacked ridges fading to Horizon Veil, tan fields in Straw Mid / Straw Lit, near treelines in Near Mass and Charcoal (winter) or Needle Dark (summer). Overcast Ivory skies. The winter valley and the Blue Ridge sunset (warmer, golden-hour variant) are the references.

**Ptormyk (v2.1).** Yellows and purples, with Japanese-Norse timber-on-stone buildings. Valley grass is Needle Mid → Straw Mid → Ptormyk Grass Sun (golden in sun, greener in shade). Mountains are Deep Blue Blood → Violet Rock → Violet Rock Lit; the snow on them stays cool blue-grey (Blue Skin, Snow Lit), and the violet read comes from the rock. Lakes are Ptormyk Lake Deep and Ptormyk Lake, the most saturated water in the set. Reference: `ptormyk_mountain_valley.jpg`.

**Southern Maphlia / Maphyk Hylvanne piedmont (v2.1).** Golden-grass plains (Granite Shade → Yhlgori Sandstone → Straw Lit), tan granite peaks and gorges (Sandstone → Spire Gold → Sunset Gold), waterfalls by the v2 cascade rule (Snow Lit or Ivory), and teal-blue lakes (Hylvanne Lake, Distance Blue). Most of this region was already in v2; only Hylvanne Lake is new for the terrain. References: `hylvynhani.jpg`, `hylvynhani_upper_city.jpg`, `hylvynhani_falls.jpg`, `klate.jpg`.

**Old Yhlgori architecture (v2.1).** The cities the Yhlgori Empire built after it conquered the Neck (Hylvynhani, Klate) share one building language wherever they stand: sandstone cliff and hill cities, aqueducts, arched bridges, terraced Greco-Roman temples, city walls, gold spires. Walls run Straw Shade → Yhlgori Sandstone → Ornament Gold → Sunset Gold; spires are Spire Gold with Straw Sun on the sunlit edge. Author it as one tileset, separate from the regional terrain, so a Yhlgori site in any biome uses the same blocks.

**Line colours (v2.1).** Audsen keeps maroon and gold: Cloth reds plus the Gold / bronze ramp, unchanged. Price uses Price Plum Shade / Plum / Plum Lit for purple, Ptormyk Lake Deep and Ptormyk Lake for blue, and Pale Silver and Ivory for white. Price Plum and Audsen Cloak Shade are 42° apart in hue, so the two Lines stay distinct on a banner.

**Niaviopti (v2.1).** White gold is Straw Shade → Yhlgori Sandstone → Warm Silver; the pearl pommel is Lime Lit → Silver Highlight → Pearl White (one or two pixels); the faint orange glow is Flame Mid and Sunset Mid, as in the shipped mythic icons.

Shared architecture (keeps, cut ashlar, roads) still uses Cold Ashlar, Cliff Teal, and the warm-stone ramp in every biome, so a Line keep does not change hue family when the map tiles do. Old Yhlgori builds are the exception above: they keep their own sandstone and gold in every biome.

## From the paintings (and photos) to pixels

The references are oil paintings, museum photos, and photographs: soft edges, huge value ranges, photographic pores. Pixel art on this project is the opposite discipline.

- **Limited ramps.** Three to five steps per material, taken only from the locked palette. A cloak is Cloth reds. A keep wall is Cut stone or Warm stone, not both in one tile unless a corner actually turns the light. A day meadow is the grass ramp, not a mix of grass and blue-blood skin.
- **Selective outline.** Close-tier characters may use a cluster outline in Warm Black or Lace Black, or a shade from the same material, not a cartoon ring of Void Ink around every form. Terrain uses outlines only at a cliff break or a tile lip that must read at Tactical zoom. Tokens and banners may use a one-pixel dark rim so they survive the phone floor.
- **Dithering is sparse.** A few pixels at a ramp boundary, or a light dither in a distant sky or haze gradient, are allowed. No checkerboard fills, no dither as a substitute for a missing mid.
- **Nearest neighbour only.** No anti-aliased edges, no smudged gold, no filtered downscales. See [Photoshop setup](#photoshop-pixel-art-setup).
- **Silhouette first.** Read the character or token in one-bit before spending the gold trim. If the mass fails without colour, it will fail at Tier 2 and on a phone.

## Photoshop pixel-art setup

Handmade in Photoshop or a close equivalent (Clip Studio, Krita, Aseprite). This is the intended setup:

- **Document.** New file in **pixels**, 72 ppi is fine, with an exact canvas (32×16 floor tile, 32×32 character or 1-step block, 16×16 token, 8×8 or 16×16 banner, or a power-of-two atlas). Do not start from a print preset or a 300 ppi sheet.
- **Swatches.** Load `palette-v2.1.aco` or `palette-v2.1.ase` into the Swatches panel (panel menu → Load Swatches… or Replace Swatches…). Keep a dedicated Varif set. Pick only from it.
- **Pencil, not Brush.** Use the Pencil tool at 1 px, 100% opacity and hardness. The Brush tool anti-aliases and will dirty the palette.
- **No anti-aliasing.** Turn anti-alias off on the Pencil, Eraser (block / pencil mode), Marquee, Lasso, Magic Wand, Elliptical and Custom Shape tools, and on type if you set any bitmap labels. Do not use Smooth, Align Edges, or stroke with anti-aliased brushes.
- **Nearest-neighbour resampling.** Set image interpolation to **Nearest Neighbor** in Preferences, and again whenever you use Image Size or Free Transform. Never resample a sprite with Bicubic or Bilinear. Judge pixels at 100%, 200%, 400%, or 800% zoom only.
- **Snap and grid.** Turn on a 1 px grid (or a 16 px grid with 1 px subdivisions; a 32×16 diamond guide is useful for floor tiles), enable Snap to Grid / Snap to Pixels, and keep the pixel grid visible at high zoom.
- **Indexed colour against the palette.** When a sprite or a tile sheet is finished in RGB, run **Image → Mode → Indexed Color**. Palette: Custom (load the `.aco`, or an ACT exported from it). **Dither: None.** Forced: None, unless you have a deliberate reason to force black and white. Confirm the colour table is only the 91 locked v2.1 colours, then save. Working in RGB with the swatch panel is fine day to day; index as a check before a sheet is committed, so no stray off-palette pixel leaves the file.
- **Export.** PNG, sharp pixels, no baked preview filtering. PNG-8 indexed or PNG-24 that still contains only palette colours. Do not pre-scale to screen size in the file; the engine scales with point sampling.

Aseprite or a similar specialist tool is compatible with this guide (import `palette-v2.1.gpl` or `palette-v2.1.ase`, nearest-neighbour, no anti-alias). The same rules apply.

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

Daylight does not recolour caste skin into the landscape ramps. A blue-blood in a meadow is still the blue-blood sky ramp, standing on Grass Mid; a black-blood on limestone is still silver, with Lime Lit only as bounce if the canvas has a pixel to spare.

## Environment rules

Mountains and valleys are a central theme, not a backdrop. Give ridges a granite or cut-stone mass and a hard lip; give valleys a broader floor in grass, straw, or water, then fade the far walls through the atmospheric ramp. The canyon pillars painting remains the architecture reference: colossal, dressed stone, a little sand, a lot of air. The new photographs supply the biome materials that painting could not.

**Scale contrast.** A Close scene should still be able to show a figure against something that dwarfs them (a keep door, a cliff, a pillar base, a cirque wall). At Operational and Strategic tiers, the land is the character: ridge, pass, lake, and road do more work than any token.

**Parallax.** Build in layers, back to front, following the [atmospheric-perspective](#atmospheric-perspective) table. Far layers use fewer ramps and no dither except a light sky grain. Do not put gameplay collision in a parallax layer.

**Hexagonal Krypts** appear as a later architectural type (whiteboard notes, summary of *causal knot theory*): hexagonal temples / engines. They are not in the first tileset, but leave room in the language of cut stone for a hexagonal plan.

## The night sky

Build the celestia as a **separate layer** (or a small stack of layers), not painted into the terrain. This stack is a **night-mode** object.

Suggested stack:

1. **Void field** — Void Ink / Warm Black, possibly a very slow nebula of Ember Void and Dried Blood (the horseman painting).
2. **Star field** — a sparse sprite sheet or particle set of Signal Red and Star Red, with a few dimmer Dried Blood points. Default stars are **red**, never white, never cool blue-white, never Sky Pale.
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

- **Pixels Per Unit:** default **32**, so one 32×16 floor tile is 1 world unit wide and 0.5 tall, and a 32×32 character is 1 unit tall (two tile-heights). This is a default; keep PPU matched to the tile width, not the character. See `sprite-tile-spec.md`.
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
  "tile_size": [32, 16],
  "sprites": [
    {
      "id": "char_kaenyn_audsen_idle_se_00_t1",
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

## Technical defaults

The floor grid and character canvas were confirmed on 2026-11-16. Remaining items are still defaults. See `sprite-tile-spec.md`.

- Floor tile: **32×16** diamond at 2:1 (confirmed 2026-11-16).
- Character canvas: **32×32**, two tile-heights tall (confirmed 2026-11-16).
- Unit token: **16×16** (default).
- Banner / strategic icon: **8×8** or **16×16** (default).
- Projection: 2:1 isometric, three-quarter characters at Tiers 1–2 (confirmed 2026-11-16).
- Palette: **91 colours**, locked, see `palette-v2.1.md` (v2.1). v2 (79) kept as `palette.md`.

## Reference list

### Concept paintings (v1, eleven)

Drive filenames from `Varif_Project_Files / Pictures / Portraits`. They were matched by file size and by content after the original chat attachments did not all arrive on disk. Use them as lighting, costume, and monumental-architecture references. Do not trace or index them into sprites.

1. **`runaway 1.webp`** — A horseman, a cloak, a horse, all in silhouette, under a red nebula packed with red stars. *Take:* the default night sky, scale of a single rider, black-on-red contrast. Palette source for the blood-red sky ramp.
2. **`throne 1.webp`** — Two tiny robed figures at the foot of colossal dressed-stone pillars in a canyon of sand and cool rock. *Take:* monumental architecture, scale contrast, the warm-stone and cut-stone split, a single shaft of sun.
3. **`rites 1.webp`** — A torchlit oath, warm orange, candle and cloak, a ring of witnesses. *Take:* interior light family, ceremony staging, paludamentum and linen in firelight. (`rites 2.webp`, a related courtyard oath, is on Drive but was not one of the eleven.)
4. **`Kaenyn Audsen.png`** — Close Roman portrait: gold ornamented armour, red paludamentum, fair red-blood skin, ice-blue natural eyes. *Take:* formal Line dress, cloth-red and gold language, red-blood highlight.
5. **`Meri Audsen.png`** — White-haired woman in black lace, a child in ivory linen. *Take:* mourning or formal civilian dress, lace black, white-hair ramp, intimate low-key portrait.
6. **`Autumn Sky.png`** — A dark-armoured man with two children, autumn leaves, a pale overcast. *Take:* family scale against a quiet landscape, Ivory atmosphere; dry foliage now shares the straw / sunset ramps.
7. **`Elyssa Dane.png`** — Chiaroscuro woman in a white wrap, gold jewellery, a black-and-gold mantle. *Take:* the lighting model for Close-tier faces, ivory-and-gold against void, red-blood skin steps.
8. **`worlds splash.png`** — A silhouetted figure by a lake under a sky of gold stars, a small warm lamp. *Take:* water and reflection, foreground silhouette, stillness. Gold stars are a special pass, not the default celestia.
9. **`Shjana.png`** — Metallic silver skin, a gold laurel helm, a dark veil. The black-blood reference. *Take:* metal-flesh ramps, laurel and gold in shadow, low-key portrait lighting.
10. **`Kheinyn Belravi.png`** — Blue-grey / teal skin, amber eyes, gold cloak on dark armour. *Take:* cool flesh in shadow, amber iris, a night-lit blue-blood. His measured greys informed the cut-stone ramp and, hue-shifted, the sky-blue caste ramp; do not copy the grey as the default blue-blood body colour.
11. **`Vakirros Grymsyhl.png`** — Imperial gold crown and armour, a red cloak, a sword, amber eyes, harsh low-key light. *Take:* a second formal-dress reference, heavier and more warlike than Kaenyn; gold against void.

### Landscape references (v2, twenty)

Mostly the user's own photographs, plus two museum / reproduction paintings. Working files live under `refs/landscapes/`. Do not trace them into tiles; take lighting, biome, and aerial-perspective structure.

12. **`652703d5.jpg`** — Hazy Blue Ridge–style sunset over layered ridges. *Take:* golden-hour atmosphere, near-black foreground ridge, warm gold haze on the far layers.
13. **`18a2338d.jpg`** — Yellowstone-style meadow with dead pines. *Take:* open day sky, tan-green meadow, dead wood in charcoal / granite, distant treeline.
14. **`d451c096.jpg`** — Conifer forest trail. *Take:* needle ramps, a narrow path, sky as a vertical slit.
15. **`050fa191.jpg`** — Sunlit grass clearing in a conifer stand. *Take:* the bright yellow-green grass ramp (Grass Lit / Grass Sun) against dark firs.
16. **`4d18ba47.jpg`** — Lake under a blue sky with fireweed. *Take:* Lake water, pale day sky, late-season dusty-rose fireweed plumes (not peak magenta).
17. **`07a610a1.jpg`** — Rocky Mountain valley under overcast sky. *Take:* overcast lighting mode, the core conifer ramp (`#404629`, `#545B35`), Ivory atmosphere.
18. **`65c69405.jpg`** — Alpine sunset over granite. *Take:* golden hour on rock, Sunset Gold in the sky, long foreground shadow.
19. **`d26d83ac.jpg`** — Granite cirque with snow and a tarn. *Take:* granite ramps, cool snow, dark tarn water reflecting stone.
20. **`9b085a9b.jpg`** — Forested valley under storm clouds. *Take:* overcast-to-storm sky, mid-distance conifer mass, granite lips.
21. **`401840a0.jpg`** — Waterfall over a boulder field. *Take:* wet granite, foam (use Snow Lit / Ivory), yellow alpine flowers (use Grass Sun).
22. **`272d9663.jpg`** — Winter Appalachian valley with blue ridges and tan fields. *Take:* the aerial-perspective ramp itself, straw fields, overcast Ivory.
23. **`c5288580.jpg`** — Juniper branch with a sandstone cliff. *Take:* near foliage against a receding warm cliff and a pale sky.
24. **`5def54fe.jpg`** — Big Bend–style limestone cliff under a deep blue sky. *Take:* Zenith Blue / Sky Mid, sun-struck limestone, desert shade.
25. **`133ebb93.jpg`** — Prickly pear on rock. *Take:* warm desert earth and stone, cactus as Needle Mid with a straw or lime edge.
26. **`06992e87.jpg`** — Golden tamarisk against a deep blue sky. *Take:* straw-gold canopy, deep day sky.
27. **`f3c86a5b.jpg`** — Canyon trail through dry grass. *Take:* the dry-grass / straw ramp, a narrow wash, a slit of Zenith Blue.
28. **`766e72f4.jpg`** — Big Bend–style limestone canyon mouth under a deep blue sky. *Take:* monumental desert scale, Lime Lit / Lime Pale, the deepest day zenith (`#456EC0`).
29. **`a219bdbf.jpg`** — Thomas Cole–style painting, castle on a crag, gold light (museum gold frame cropped before measuring). *Take:* Hudson River School composition, gold atmosphere, olive-teal River Teal, a bright distant cleft.
30. **`4901f966.png`** — Albert Bierstadt painting, from a phone screenshot of an X post (phone UI and compose button cropped or masked before measuring). *Take:* cathedral light in a mountain valley, gold air, a luminous far gap, dark near shore.
31. **`8649050e.jpg`** — Sun through forest, backlit orange. *Take:* the sunset-orange ramp, Forest Shade as the tree mass, a small hard sun.

### Game references (2026-11-20 sweep, four)

These are direction only. Do not copy, trace, or borrow designs. Take zoom readability, lighting, and terrain-structure lessons.

32. **Songs of Conquest** (Lavapotion) — [Steam](https://store.steampowered.com/app/867210/Songs_of_Conquest/). Pixel units and towns that stay readable on a big lit adventure map, with strong silhouettes when zoomed out. *Take:* zoom-out readability and silhouette. *Use:* the grand-strategy zoom tier, for low-detail units and settlements.
33. **Blood of Rome** (Pixelatus, isometric pixel, planned December 2026) — [Steam](https://store.steampowered.com/app/2659460/Blood_of_Rome/). Roman helmets, shields, and cloaks that keep their weight at small isometric scale and stay clear in tight formations. *Take:* clarity of Roman kit at small scale and in formation. *Use:* Roman-inspired troop sprites on the 32×16 grid; clarity lessons only, not their soldier designs.
34. **Laysara: Summit Kingdom** (Quite OK Games, February 2026) — [Steam](https://store.steampowered.com/app/1823950/Laysara_Summit_Kingdom/). Mountains split into vegetation zones by altitude, with weather and avalanches that change the terrain. *Take:* altitude vegetation bands and weather that rewrites terrain. *Use:* Shentz Valley and Utzkva'hjun tiles, with those bands mapping onto palette v2 (golden-green valleys, then grey rock, then snow).
35. **Sea of Stars** (Sabotage Studio) — [seaofstarsgame.co](https://seaofstarsgame.co/). Pixel sprites lit with normal maps while the sun and moon move. *Take:* normal-mapped pixel lighting with a moving sun and moon. *Use:* palette v2 day and night modes, so the red stars and the bronze moon can throw real light across the terrain at night.

### Concept art (v2.1 palette sources, five plus two)

36. **`ptormyk_mountain_valley.jpg`** — Timber-and-stone shrine with a curved tiled roof beside a blue lake under violet peaks. *Take:* the Ptormyk yellows and purples. Palette source for the Ptormyk grass, violet rock, snow, and lake ramps.
37. **`hylvynhani.jpg`** — Cliff city on an aqueduct above a waterfall, granite spires, a lake and golden plain below. *Take:* old Yhlgori sandstone, tan granite, golden grass, lake.
38. **`hylvynhani_upper_city.jpg`** — The upper city: terraced sandstone and three gold spires over a gorge river. *Take:* spire gold, sandstone mass.
39. **`hylvynhani_falls.jpg`** — The falls, aqueduct, and lake from the plain. *Take:* sandstone, plain, lake.
40. **`klate.jpg`** — Walled hill city, terraced temples, an arched bridge over a gorge, sunset. *Take:* Yhlgori walls and terraces.
41. **`niaviopti.jpg`** — The Niaviopti sword painting. *Take:* white gold and the pearl pommel.
42. **`emanda-price.png`** — Emanda Price portrait (Price Line). *Take:* the Price plum.

Further landscape (and other) references will be added. When they arrive, follow `analysis/README.md` and the update steps in `palette-v2.1.md` rather than colour-picking ad hoc.

## Sources and conflicts

Visual facts taken from Drive (read only; nothing on Drive was edited) are marked with their document names above. The folder is **`Varif_Project_Files`**. The twenty landscape photographs and two painting photos were supplied by the user as chat attachments in November 2026.

**Used.**

- `A Pylgrym's Plight` (manuscript): Lines, not Houses; sky-blue / crystal-blue skin and inverted eyes; Samule on southern blue-bloods and metallic bronze-gold black-bloods; silver-skinned men on the Ptormyk docks; moving constellations and red "Creation's Chisel"; named star-groups (Dragon, Lamb, Fox, Hare, Raven, Crow, Blood Sage).
- `A Pylgrym's Plight — Collated Whiteboard Notes`: blood-colour table (silvered Kamloran / Myhjodi skin, blood-orange eyes, Price blood eyes, purple eyes, dusk-coloured southern blood); Krypts and the red-shift of the stars; ecology of the High and Low Valleys; hexagonal Krypts (*causal knot theory* summary); Serpent of Time; atzhelm as "land of the valley."
- `Reworks.txt`: plot notes only; no additional visual facts used.
- `Pictures / Portraits` (and the Variff subfolder): the eleven concept paintings above.
- The twenty v2 landscape attachments listed above.

**Noted, not treated as sprite law.**

- Southern-isle "sea" skin, purple irises, and dusk-indigo blood (manuscript / whiteboard) are regional variants. Default blue-blood art is sky-blue skin and Deep Blue Blood.
- Far-western black-bloods the colour of bronze and gold (manuscript) are a regional metal variant. Default black-blood art is silver, per the brief and Shjana.
- Book 2 board title "House Dane" in the whiteboard is a board slip; the manuscript and this guide use **Line**.
- Late-season fireweed in the lake photo is dusty rose, not peak magenta. A brighter bloom would need a future palette review.
- The Cole river, as photographed in museum light, measures as olive-teal (River Teal), not a bright cyan. Paint the *measured* water, not an imagined postcard turquoise.

**Could not be used, or was left alone.**

- `dictionary.txt` is cited in the whiteboard and was not found on Drive.
- `theAncientPainting` (in `The_Manifold`) was not edited and was not used as a visual bible; the whiteboard already summarises *That Ancient Painting* as a parable, not as an art guide.
- `Variff.zip` / `Variff complete.png` and other large archives were not unpacked.
- `family trees.csv` was not needed for visual facts.
- The original eleven chat attachments did not all materialise on disk; the same pictures were recovered from Drive by size and content. `rites 2.webp` was inspected and is not in the locked reference set.

The floor grid (32×16, 2:1 isometric) and the 32×32 character canvas were confirmed on 2026-11-16. The render engine is still undecided. See `sprite-tile-spec.md`, `naming-convention.md`, and `shot-list.md`.

## Palette change log

- **2026-11-21 — palette v2.1 (91 colours).** The guide now points to `palette-v2.1.*`. Added 12 measured colours in 11 new ramps: Price Line purples (3), Pearl White for Niaviopti (1), Ptormyk Grass Sun, Violet Rock, Violet Rock Lit, Ptormyk Lake Deep, Ptormyk Lake (5), Yhlgori Sandstone, Spire Gold, Hylvanne Lake (3). Measured in OKLab from the concept art above with documented masks and even lightness steps; 23 of 35 measured steps matched an existing swatch within OKLab 0.03 and reuse it. No v2 colour changed or removed. Recommended cap raised from 80 to 96. Biome notes added for Ptormyk, southern Maphlia / Maphyk Hylvanne, old Yhlgori architecture, Line colours, and Niaviopti. Details: `palette-v2.1.md`.
- **2026-11-15 — palette v2 (79 colours).** Core / night / daylight modes from the twenty landscape references. See `palette.md`.
- **2026-11-15 — palette v1 (48 colours).** First locked set. Archived as `palette-v1.*`.
