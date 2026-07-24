# Black Ops 2 Emblem Swapper

A desktop app for copying another player's Black Ops II emblem onto your own PS5 account.

You capture the emblem while looking at that player's profile, then load it into your own in-game emblem editor and save it there. From that point it's yours, the same as anything else you built in the editor by hand.

Built for personal use, on your own PS5 and your own home network.

## How it works

Black Ops II fetches emblem data over plain HTTP, from Treyarch's Demonware servers. There's no encryption on that one endpoint, so a small proxy on your PC can sit between the console and the internet and watch that single request pass by.

The app runs that proxy. You point your PS5's network settings at your PC's IP address, and every other request your console makes still goes straight out to the internet untouched. HTTPS traffic, including PSN sign-in and matchmaking, is tunneled through byte for byte and never decrypted. The proxy only pays attention to one thing: requests whose path matches a player's saved emblem slot.

Three modes control what happens to those requests:

| Mode | Behavior |
|---|---|
| Off | Nothing is touched. The proxy is fully transparent. |
| Capture | The real response from Demonware is saved to disk before being passed along. |
| Show | The real response is swapped out for whichever emblem you've selected in the app. |

Switch to Capture, open a player's profile, and their emblem lands in your library automatically. Select it, switch to Show, and open your own emblem editor. Whatever slot the game asks for, it gets your selected emblem back instead of your own. Save it in the editor and it's permanently yours.

### The emblem format

Each captured slot is a 1408-byte blob: 32 fixed layer records, 44 bytes each. A layer holds a shape ID, an RGBA color, a position, a rotation, and a scale, packed as raw little-endian integers and floats. None of this is documented anywhere. Working out what each field actually meant, and matching the game's own renderer pixel for pixel, took writing values directly into the game's memory and comparing screenshots against the app's own output.

A few things about it aren't what you'd guess:

- Scale isn't linear. The stored value is an exponent: actual scale is `2^value`, so `0` fills the whole box and each step up or down doubles or halves the size.
- Rotation is stored in degrees, clockwise, but has to be negated to match how standard image libraries rotate counter-clockwise.
- The "outlined" flag doesn't draw a filled shape with a border. It draws only the edge: a thin ring computed by dilating and eroding the shape's silhouette and keeping the difference.

The renderer reproduces all of this from scratch in Rust, including a hand-written bicubic resampler and morphological filter, so a captured `.bin` file renders identically to what the game itself would show.

## Using it

1. Turn on Capture mode.
2. On your PS5, open the profile or channel of the player whose emblem you want. It's saved automatically.
3. Click the captured emblem in your library to select it.
4. Turn on Show mode.
5. Open your own emblem editor on the PS5. The selected emblem loads in place of whatever you'd normally see there.
6. Save it, same as anything you made yourself.

Full setup, including the exact PS5 network settings screen and firewall troubleshooting, is in [docs/INSTALL.md](docs/INSTALL.md) and built into the app itself under Setup & Troubleshooting.

**Two things worth knowing:**

- Black Ops II only checks for a new emblem once per game launch. To load a different one after the first, restart the game, or switch to Zombies and back to Multiplayer.
- If a captured emblem uses a shape, rank icon, or weapon qualification you haven't personally unlocked, the game may refuse to load or save it. That's the game enforcing it, not something this tool can work around.

## Architecture

The app is a native Tauri window: a Rust backend handling the proxy, file storage, and rendering, talking to a React and TypeScript frontend over Tauri's IPC bridge. No web server, no browser tab, no polling loop. The Rust side pushes events to the UI the moment something happens.

```
app/
  src/                      React frontend
    components/             UI components
    lib/                    Tauri IPC bindings and shared types
  src-tauri/
    src/
      proxy.rs              the MITM proxy itself
      storage.rs            captured emblems and their labels on disk
      broadcast.rs          which emblem is currently selected
      state.rs              current mode: off, capture, or show
      emblem/
        format.rs           binary layer record parsing
        render.rs           the pixel-accurate renderer
        shape_map.rs        shape ID to reference-image lookup
      commands.rs           the Tauri commands the frontend calls
    resources/
      reference_shapes/     261 reference glyphs used to render captured emblems
```

## Building from source

You'll need [Rust](https://rustup.rs) and [Node.js](https://nodejs.org).

```
cd app
npm install
npm run tauri dev
```

`npm run tauri build` produces a standalone installer under `app/src-tauri/target/release/bundle`.

## Scope

This only intercepts Black Ops II's own emblem-storage requests. Everything else passes through unmodified, including PSN authentication, which stays encrypted the whole time. Emblem data isn't private: it's the same thing already displayed on other players in-game. This project isn't affiliated with Activision, Treyarch, or Sony.

## License

MIT. See [LICENSE](LICENSE).
