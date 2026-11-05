# Dev 2.35 host presentation lifecycle

## Boundary

- Rollback baseline: `bd816d76152c840f1c9f874b187148f5342aaa58` (Dev 2.34).
- This revision changes only guest/native orientation ownership and the native
  Home-indicator policy while a guest is displayed.
- Dev 2.34 retained-catalog return, library change detection, guest teardown,
  and full-rescan fallback remain unchanged.
- Persistent cold-start indexing remains the next separate performance phase.

## Video evidence reviewed at 8 frames per second

- `11-13-31`: repeated returns show the portrait picker first reaching its
  correct destination and then entering an extra landscape-to-portrait cycle.
- `11-26-08`: the short diagnostic example isolates the same sequence:
  portrait picker visible, then landscape, then portrait again.
- `11-16-29`: two landscape controls show the behavior to preserve: one
  continuous picker-to-landscape-guest rotation and one continuous
  landscape-guest-to-portrait-picker rotation. It also shows the persistent
  native Home indicator over guest content.

## Cause and correction

SDL's orientation mask is process-global and its root controller may remain in
the scene briefly while the short-lived picker window is created. On guest
return the old mask and scene geometry preference still represented the
terminated landscape guest. The portrait-only picker could therefore reach
portrait, then lose scene ownership to that stale guest state, before its own
orientation mask forced a second return to portrait.

Dev 2.35 adds explicit process-lifetime presentation ownership:

- picker ownership is established before picker reconstruction and resets the
  SDL orientation mask to portrait;
- guest ownership begins only immediately before guest `Environment`
  construction;
- guest orientation requests carry the current ownership generation, so work
  queued before teardown is rejected after picker ownership is restored;
- picker installation requests portrait from the actual picker controller,
  while guest requests update only active non-picker roots;
- bounded logs record ownership, generation, request source, visible state,
  transition state, and accepted/ignored/result status.

This preserves genuine landscape guests and changes neither emulated
framebuffer orientation nor guest touch/accelerometer coordinates.

## Home indicator

The SDL guest controller already implements UIKit's supported Home-indicator
preference, but iDared had never selected the visual-auto-hide policy. Dev 2.35
sets `SDL_IOS_HIDE_HOME_INDICATOR=1` before each guest window is created. SDL
then returns `true` from `prefersHomeIndicatorAutoHidden` while leaving
`preferredScreenEdgesDeferringSystemGestures` at `.none`. The persistent white
indicator is hidden without requiring a second swipe or intercepting the real
iOS Home/background gesture.

## Focused physical-device retest

1. Portrait guest: normal return and recoverable crash return should remain
   portrait, with no landscape excursion.
2. Landscape guest: launch and normal/crash return should each use one
   continuous native rotation.
3. Portrait and landscape guest gameplay should not show a persistent native
   Home indicator; deliberate system interaction may reveal it transiently.
4. The real iOS Home/background gesture must still work.
5. The return log must still report `retained-catalog`, with no package rescan
   for an unchanged library.
