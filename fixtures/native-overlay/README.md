# Native overlay acceptance fixture

Original Durop test artwork, distributed under the repository MIT license.
`border.gif` is a 320 × 180 single-frame transparent GIF with a 30-pixel opaque
magenta border. Its sidecar centers it with cover fit and loops for at most ten
minutes. It deliberately covers the Dock/menu areas while exposing the target
application through its center. Use the [acceptance procedure](../../docs/overlay-proof.md#native-panel-validation-2026-11-11).

Run `nix develop --command cargo run --locked -- --pack fixtures/native-overlay`,
then enter `play native-proof/border` and `stop` in the attached terminal while
the target application stays frontmost. The fixture is a visual probe, not a
catalog recommendation or a proof of input passthrough.
