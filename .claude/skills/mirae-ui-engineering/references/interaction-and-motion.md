# Mirae Interaction and Motion

## Frequency rule

- Hundreds of times per day: no open/close animation; respond immediately.
- Tens of times per day: use minimal feedback only.
- Occasional overlays or transitions: use short, interruptible motion.
- Rare onboarding or success moments: allow restrained delight.

## Purpose rule

Animate only for:

- spatial continuity;
- state change;
- feedback;
- preventing a jarring insertion or removal;
- explaining a transition the operator might otherwise misread.

Do not animate solely because the result looks decorative.

## Performance rule

- Prefer `transform` and `opacity`.
- Avoid animating layout properties in high-frequency surfaces.
- Avoid `transition: all`.
- Keep keyboard-triggered command surfaces immediate.
- Make every motion reversible and interruptible when user input can change direction.
- Provide a reduced-motion result that remains understandable.

## Mirae personality

Mirae motion is precise, calm, fast, and operational. Avoid excessive bounce, playful overshoot, slow cinematic transitions, and motion that delays production controls.
