# Mr. Mime source review

Reviewed 2026-10-12. The current canonical mesh's oblique software baseline
rendered at 800×824, exited 0 and was inspected in full. Its 34 parts produce
a pink torso, oversized protruding oval eyes, rod-like four-finger hands,
plain feet and triangular blue head fins. The toy body omits the white torso,
red hip joints, bent limbs, five rounded fingertips and curled shoes.

The official [0122 entry](https://sg.portal-pokemon.com/play/pokedex/0122/)
returned HTTP 200. Its named
[primary artwork](https://sg.portal-pokemon.com/images/pokedex/3bf0162c653873e4ecf5810c830191bac19cb540.png)
was downloaded into ignored review scratch and visually inspected. It shows
a cream torso with a round pink-red chest marking, round shoulder and hip
joints, thin pale pink bent limbs, white knee joints, curled dark blue shoes,
a pale pink round head, fitted small white oval eyes and narrow pupils,
round pink cheeks and an open smile. Dark blue head fans have scalloped
outer silhouettes. Each open white glove has five splayed digits and rounded
pink fingertip pads; the raised and forward hands form an asymmetric pose.

The primary Sketchfab downloadable-model search returned HTTP 200. Four
specific CC BY 4.0 candidate model endpoints returned HTTP 200, but all four
archive endpoints returned HTTP 401:

| Candidate | Creator | Faces | Listed clips | Source concern |
| --- | --- | ---: | ---: | --- |
| [Mr. Mime in the Box](https://sketchfab.com/3d-models/9636a92b36e2498b8839298896fb338d) | kuayarts | 30,934 | 0 | Practical-joke presentation; archive unavailable |
| [Mr. Mime](https://sketchfab.com/3d-models/509a98e38e8f405cb66029156ee40b26) | jtejada272010 | 4,966 | 0 | Author explicitly says the model is not theirs and credits Models Resource/Nintendo/Game Freak/Creatures; the listing alone does not establish original permission |
| [Mr. Mime](https://sketchfab.com/3d-models/8af04addb8bc4a18b30d20349dcbd5fa) | nguyenlouis32 | 5,014 | 0 | Lore description; archive unavailable |
| [Pokemon Mr Mime](https://sketchfab.com/3d-models/ab768f68a9414a26bc678b103984f865) | herogamer0108 | 2,740,230 | 0 | Dense model; archive unavailable |

Other search results included personal-use-only Free3D content listed as
unrigged, paid puppets and NC-ND Sketchfab candidates. No external archive,
mesh, texture, rig or clip was retrieved or imported. Zero listed clips does
not prove the absence of a rig. No usable open rig has been established by
this search; this is not a claim that none exists.

The [original replacement](mr-mime-sculpture.md) is now installed from
`tools/mr_mime_sculpt.py`. Its 67 closed parts and 105,964 triangles restore
the missing anatomy. Fourteen final candidate checks passed in 28.866 seconds,
and final front/oblique/rear software views were individually inspected. The
pure builder has a dedicated dispatch through the preserved Rust species route.
Fourteen installed checks passed in 27.805 seconds; a final-hash explicitly muted native GPU battle capture exited 0 and
was inspected in full. A direct Rust model test containing the complete final SHA passed once
in 3.72 seconds with source/executable stable. No
live-window, browser, rig, animation or gameplay approval is claimed.

The first original render exposed a detached wrist and self-crossed shoe curl;
regressions rejected both. After seating the wrist and widening/tapering the
curl, fourteen checks passed in 24.391 seconds. The intermediate mouth still
looked puckered; denser face geometry, a broad fitted smile lining and smaller
lip projection improved the expression. A detached tongue and excess contour
sampling were caught by root/budget checks and corrected before final review.
