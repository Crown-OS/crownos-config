# CrownOS settings reference

Every setting is one RON file under `~/.config/crownos/`, named after its section. Apps watch their file and apply a change as soon as it is saved, so editing the file is the same as using the settings panel. Generated from `src/schema`; do not edit.

| Section | File | About | Read by |
|---|---|---|---|
| [agents](agents.md) | `agents.ron` | AI agents on this desktop: which programs may drive it, and which applications may show their UI to agents. | The consumer is [crownpositor], which reads this file live. |
| [appearance](appearance.md) | `appearance.ron` |  | every CrownOS app, through crownos-config |
| [bar](bar.md) | `bar.ron` | The status bar: where it sits, how it is shaped and what is on it. | every CrownOS app, through crownos-config |
| [bluetooth](bluetooth.md) | `bluetooth.ron` |  | every CrownOS app, through crownos-config |
| [camera](camera.md) | `camera.ron` | The camera: which device, how it records, and which AI effects are on. | The consumer is crownos-camera. |
| [clipboard](clipboard.md) | `clipboard.ron` | The clipboard manager: how much of what was copied outlives the app that copied it. | The consumer is [crownos-clipboard], which reads this file at startup. |
| [compositor](compositor.md) | `compositor.ron` | Window management: tiling, keybindings, window rules, outputs. | The consumer is [crownpositor], which reads this file live — a rebind takes effect without a restart. |
| [display](display.md) | `display.ron` |  | every CrownOS app, through crownos-config |
| [files](files.md) | `files.ron` | The file manager: how folders are listed and what happens when a drive is plugged in. | The consumer is crownos-files, which follows this file while it runs. |
| [glass](glass.md) | `glass.ron` | The lit edge of the compositor's frosted glass. | every CrownOS app, through crownos-config |
| [input](input.md) | `input.ron` | Text input: for now, dictation. | The consumer is [crowndictator], which reads this file at startup and follows it live — turning the feature off releases its keyboard grab, and changing the shortcut re-arms it without a restart. |
| [keybinds](keybinds.md) | `keybinds.ron` | Desktop-wide keyboard shortcuts. | The consumer is the compositor. It reads this file live, so rebinding takes effect without a restart or a re-login. |
| [lock](lock.md) | `lock.ron` | The look and behaviour of the lock screen, drawn by the compositor. | The compositor reads this section live. |
| [notifications](notifications.md) | `notifications.ron` |  | every CrownOS app, through crownos-config |
| [plugins](plugins.md) | `plugins.ron` | Which themes and plugins are active and what each has been allowed to do. | Written only by `crownplugind`; everything else asks it over IPC. |
| [power](power.md) | `power.ron` |  | every CrownOS app, through crownos-config |
| [recorder](recorder.md) | `recorder.ron` | The voice recorder: how new recordings are captured and saved. | The consumer is crownos-sound-recorder, which writes this file whenever one of its pickers changes. |
| [screenshot](screenshot.md) | `screenshot.ron` | Screenshots and screen recordings: where they land and how they are encoded. | The consumer is crownos-screenshot, which reads this file each time it runs. |
| [sound](sound.md) | `sound.ron` |  | every CrownOS app, through crownos-config |
| [theme](theme.md) | `theme.ron` | Semantic design tokens shared by every CrownOS surface. | every CrownOS app, through crownos-config |
| [visual_context](visual_context.md) | `visual_context.ron` | The visual context engine: what it may look at and how long it remembers. | Two consumers: [crownpositor] reads the exclusion rules live and enforces them in the compositor, so a window stays out of captures even when the engine is not running; [crown-contextd] reads everything else. |
| [voice](voice.md) | `voice.ron` | Voice enhancement for every app's microphone. | The consumers are crown-voiced, which follows this file live, and the settings panel's Voice page, which writes it. |
| [wifi](wifi.md) | `wifi.ron` |  | every CrownOS app, through crownos-config |
