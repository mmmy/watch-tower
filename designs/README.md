# Designs

This directory stores the source Pencil design files for Watch Tower.

## Primary file

- `watch-tower-ui.pen`

This is the main product design source for:

- bootstrap and window policy
- main control console
- optional market overview grid
- window behavior center
- slide-out alert popup
- edge widget
- dock / auto-hide states
- hover / click-through states
- multi-window orchestration

## Usage

- Treat `watch-tower-ui.pen` as the canonical design file
- Prefer updating this file instead of creating many parallel variants
- Save design changes into this file so they can be versioned with the codebase

## Notes

- Do not rely on Pencil temporary canvases as the only copy
- If a major redesign happens, create a clearly named successor file instead of overwriting blindly
