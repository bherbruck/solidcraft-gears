# SolidCraft Gears Plugin (`solidcraft-gears` v0.1.15)

An involute spur gear generator plugin for **SolidCraft** (`0.1.44+`) written in Rust and compiled to WebAssembly.

## Features

- **Separate Inch & Metric Menu Entries:** Launches directly from **Gears (in)** or **Gears (mm)** on the SolidCraft menu bar with unit-appropriate defaults (`in` or `mm`) and live preview before committing.
- **Automatic Sequential Naming:** Names generated bodies `Gear 1`, `Gear 2`, `Gear 3`, etc., along with tooth count and outer diameter (`OD`).
- **Auto-Mating & Tooth-Phase Alignment:** Automatically calculates center-to-center pitch distance and rotates mating gears into phase so teeth mesh cleanly (`Meshed` mode) or sit side-by-side with print clearance (`Separated` mode: `0.25 in` / `6.35 mm`).
- **Profile Shift & Center Bore:** Supports addendum modification (`profile_shift`) and optional center bore cutouts.

---

## Installation

1. Download `solidcraft-gears-0.1.15.zip` from the Releases page.
2. Open **SolidCraft → Preferences → Plugins**.
3. Click **Install from ZIP** (or load the archive), select `solidcraft-gears-0.1.15.zip`, and approve the plugin permissions.
4. Two menu items will appear on your SolidCraft menu bar: **Gears (in)** and **Gears (mm)**.

---

## Converting US Diametral Pitch (DP) to Inch Module

Metric gears are specified directly by **Module (mm)**, whereas US customary gears are commonly specified by **Diametral Pitch (DP)**—the number of teeth per inch of pitch diameter.

Because the **Gears (in)** dialog expects **Module (in)** as a decimal number (it does not evaluate math formulas inside the input box), convert DP to Module beforehand:

$$\text{Module (in)} = \frac{1}{\text{DP}}$$

### Example: 20-Tooth, 16 DP Gear
1. Open **Gears (in)** from the SolidCraft menu bar.
2. Calculate the inch module on a calculator:
   $$1 \div 16 = 0.0625$$
3. Enter **`0.0625`** into the **Module (in)** box.
4. Set **Teeth** to **`20`**.
5. Verify the resulting gear dimensions:
   - **Pitch Diameter:** $0.0625 \times 20 = 1.250\text{ in}$
   - **Outer Diameter (OD):** $0.0625 \times (20 + 2) = 1.375\text{ in}$

### Common US Diametral Pitch Reference

| US Diametral Pitch (DP) | Formula ($1 \div \text{DP}$) | Enter in `Module (in)` |
| :--- | :--- | :--- |
| **8 DP** (coarse teeth) | $1 \div 8$ | `0.125` |
| **10 DP** | $1 \div 10$ | `0.1` |
| **12 DP** | $1 \div 12$ | `0.0833` |
| **16 DP** | $1 \div 16$ | `0.0625` |
| **20 DP** | $1 \div 20$ | `0.05` |
| **24 DP** | $1 \div 24$ | `0.0417` |
| **32 DP** (fine teeth) | $1 \div 32$ | `0.03125` |

---

## Building from Source

Requires Rust (`wasm32-unknown-unknown` target) and `wasm-tools`:

```powershell
cargo build --release --lib --target wasm32-unknown-unknown
wasm-tools component new target\wasm32-unknown-unknown\release\solidcraft_gears.wasm -o plugin.wasm
Compress-Archive -Path plugin.json, plugin.wasm -DestinationPath ..\solidcraft-gears-0.1.15.zip -Force