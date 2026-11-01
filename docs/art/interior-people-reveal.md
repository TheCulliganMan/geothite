# People behind indoor source objects

Registered source objects use their real triangle surfaces to identify blocked
bodies. Their depth-aware reveal supports both coverage rendering (closed
cabinets) and smooth layered alpha (timber/facility partitions, equipment and
wide tables). Coverage selects the drawing style; it does not decide whether
an already registered source object can reveal a character.

`interior_npc_reveal::source_object` admits an existing depth-aware owner with
residual opacity0.20 and no camera-wall plane. The source owner must exclude
its floor and architectural backing. Keep its existing coverage/smooth recipe,
full mesh, support heights, source claims, shadows and collision.

Player obstruction uses the existing body rays. Indoor NPCs use actual source
object IDs, modeled stature and resolved footing; torso/head obstruction admits
a reveal, while shoes behind a low counter do not. A per-object union encloses
only its blocked bodies. The material clears that projected body area and
retains the object's existing faint form elsewhere. Move clear and the owner
returns fully opaque through the existing hold/restore timing.

Camera-wall planes retain their separate outside-room cutaway. Ordinary owners
without depth-aware registration retain their existing policy. Outdoor building
and foliage behavior remains separate. A new furniture or wall model needs an
authenticated, complete source body range before adopting this reveal; keep its
floor outside that range.

The smooth and coverage live-system tests in `interior_npc_reveal.rs` exercise
NPC-only obstruction with player rays clear, player-only obstruction, opaque
restoration, stable mesh/material counts, camera-wall exclusion and map/cache
identity. Source mesh tests verify full-height partition sides, source openings,
closed ends and floor/support ownership. Static311/225 audits must remain exact
for a reveal-only change.

Review each affected room from both sides and during ordinary movement, against
2D at the same state. Keep native captures offscreen. The ignored
`target/kanto-resume/wall-reveal-review/` evidence covers WiseTriosRoom,
PowerPlant, Rocket-wall controls and the shared wide table. Visibility acceptance
does not certify the room's remaining actor palettes, modeling or services.
