# SlideML

SlideML lets an AI agent make PowerPoint presentations. The agent writes slides in SlideML, a small XML format. The `slideml` compiler turns the XML into a native `.pptx` file. Every shape, text box, chart, table, picture and animation stays editable in PowerPoint. There are no screenshots inside the file.

The package is a skill for [Claude Code](https://claude.com/claude-code). The skill tells the agent how to plan the story, choose a style, find photos, build the slides and check the result in PowerPoint.

![Slides that the agent made with SlideML](docs/images/gallery.jpg)

## Features

- **Native PowerPoint output.** The compiler writes Open XML directly. There is no HTML step and no slide screenshot. Each element is a real PowerPoint object: text, shape, chart with Excel data, table, picture, animation or morph transition. You can edit everything after the agent finishes.
- **Lightweight.** The compiler is one 2.4 MB `slideml.exe`. It has no runtime, no browser and no Node.js. The icons (about 2,150 from Lucide) are inside the executable.
- **Fast.** The compiler builds a 6-slide deck in about 0.25 s.
- **Made for AI agents.** An agent writes short XML, not Open XML. The compiler measures text with the real font files and gives warnings with the fix: text overflow, small font size, overlap, almost-aligned edges. It also writes the frame of every element to a JSON manifest. Thus an agent without vision can check the layout.
- **Checked in PowerPoint.** One command, `check.py`, compiles the deck, opens it in PowerPoint, exports the slide images and reports text overflow. The images are only for the review. They are not in the `.pptx` file.

## Requirements

- Windows 10 or 11.
- Microsoft PowerPoint (Microsoft 365 or 2016 and later). The skill renders every slide in PowerPoint to check it.
- Python 3.
- Claude Code.

You do not need Rust or .NET.

## Install

Paste this text into Claude Code:

```
Install the SlideML skill. Run this command in PowerShell:
irm https://github.com/YanGaev2/slideml/releases/latest/download/install.ps1 | iex
```

You can also run the command in PowerShell yourself.

The installer does these steps:

1. It puts the skill and `slideml.exe` into `~/.claude/skills/slideml`.
2. It installs the fonts of the style presets (Inter, Manrope, Unbounded, Montserrat) for the current user. You do not need admin rights.
3. It installs the Python package Pillow if it is missing.
4. It tells you if PowerPoint is missing.

To update the skill, run the same command again. To remove the skill, delete the folder `~/.claude/skills/slideml`.

If the skill does not appear in Claude Code, restart Claude Code.

## Use

Ask Claude Code for a presentation. Give the facts and numbers. For example:

```
Make a 5-slide presentation for the bank board about the support chatbot pilot.
8 weeks, 41,200 requests, the bot closed 25,956 without an operator.
First answer: 25 s with the bot, 4 min 10 s before. Cost per request: 38 RUB vs 160 RUB.
We ask to roll out the bot to all channels in Q1 2027.
```

The agent does these steps:

1. It writes a plan: the audience, their questions, the numbers and a storyboard.
2. It chooses a style and finds photos with an open license.
3. It writes `deck.xml`.
4. It runs `check.py`. The script compiles the deck, renders it in PowerPoint and finds text overflow and layout problems.
5. It reviews the slide images and makes at least two design passes.

The result is in the folder of the presentation:

```
<name>/
  plan.md            story, numbers, storyboard
  deck.xml           SlideML source
  assets/            photos and SOURCES.md with the photo licenses
  review.json        design review passes
  out/<name>.pptx    the presentation
  out/<name>/        slide images: slideN.png and slideN.overlay.png
```

The agent does not invent numbers. If data is missing, it writes `[уточнить]` ("to clarify") on the slide and tells you what it needs.

The skill instructions are in Russian. You can ask for a presentation in any language.

## The SlideML format

A slide is 960 × 540 pt. You set the position and size of each element, or use `h="auto"` for text.

```xml
<deck lang="en-US">
  <theme font="Manrope" heading="Unbounded" dk1="#0B0B2E" lt1="#FFFFFF" accent1="#FF5A1F"/>
  <slide id="cover" bg="dk1">
    <text x="64" y="80" w="760" h="auto" font="heading" size="40" color="lt1">The bot closes 63 % of requests</text>
    <chart type="column" x="64" y="200" w="560" h="280" labels="1" legend="none" color="lt1/70">
      <cat>Week 1, Week 4, Week 8</cat>
      <series name="Closed by bot, %" values="41, 55, 63" color="accent1"/>
    </chart>
  </slide>
</deck>
```

The full reference is in [`format.md`](.claude/skills/slideml/format.md).

You can run the compiler without the agent:

```
slideml build deck.xml -o deck.pptx
```

The compiler also writes `deck.manifest.json` with the computed frame of every element. It measures text with the font files and prints a warning when text does not fit.

## Repository layout

| Path | Contents |
|---|---|
| `.claude/skills/slideml/` | the skill: instructions, `scripts/check.py`, fonts |
| `proto/` | the compiler in Rust |
| `proto/tools/validate/` | Open XML SDK validator for development (needs .NET 8) |
| `docs/design.md` | design notes |
| `examples/` | example decks |
| `install.ps1` | installer for users |
| `package.ps1` | builds `slideml.exe` and packs the skill |

## Development

1. Install Rust.
2. Run `./package.ps1`. The script builds `slideml.exe`, copies it to `.claude/skills/slideml/bin/` and writes `dist/slideml-skill.zip`.
3. Open Claude Code in this folder. Claude Code finds the skill in `.claude/skills/`.

To make a release, push a tag such as `v0.1.0`. The GitHub workflow builds the zip and attaches it and `install.ps1` to the release.

To test the installer with a local build, set `SLIDEML_ZIP` before you run it:

```
$env:SLIDEML_ZIP = "$PWD\dist\slideml-skill.zip"; Get-Content -Raw install.ps1 | iex
```

## License

The code is under the MIT license, see [LICENSE](LICENSE).

- Icons: [Lucide](https://lucide.dev), ISC license (`proto/vendor/LUCIDE-LICENSE.txt`).
- Fonts: Inter, Manrope, Unbounded and Montserrat, SIL Open Font License 1.1 (`.claude/skills/slideml/fonts/`).
- Photos in `examples/*/assets/`: the license of each photo is in `SOURCES.md` in the same folder.
