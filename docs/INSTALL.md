# PS5 Setup & Instructions

## Setup

> **Note:** Your PS5 and PC must be connected to the exact same network (same Wi-Fi, Ethernet, or PC mobile hotspot).

1. On PS5: Go to **Settings** → **Network** → **Settings** → **Set Up Internet Connection**.
2. Select your active connection, then select **Advanced Settings**.
3. Set **Proxy Server** to **Use**.
4. Enter the **IP Address** and **Port** shown in the app (e.g., `192.168.1.42` and `8080`).
5. Save settings and test the connection.

---

## How to Copy Emblems

1. Turn on **Capture Mode** in the app.
2. On your PS5, view the profile or channel of the player whose emblem you want. The app saves it automatically.
3. In the app, select the captured emblem.
4. Turn on **Show Mode**.
5. Open your PS5 emblem editor. The selected emblem will load in place of your own.
6. Save the emblem.

---

## Game Limitations

* **Session Caching:** Black Ops II only fetches emblem data once per launch. To apply a different emblem after opening the editor, restart the game or switch between Multiplayer and Zombies to clear the cache.
* **Locked Content:** You cannot save emblems that contain shapes or layers you haven't unlocked on your account. The game enforces this natively.

---

## Troubleshooting

### PS5 says "No Internet Connection"

* **Wrong Network:** Confirm both devices share the same IP range. If using a PC hotspot, select the hotspot's IP address in the app, not your router's IP.
* **Firewall:** Windows Firewall may be blocking incoming traffic on port `8080`. Run this in **PowerShell (Admin)** to fix it:
  ```powershell
  New-NetFirewallRule -DisplayName "Black Ops 2 Emblem Swapper" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow -Profile Private
  ```
* **DNS Blockers:** Ad-blocking DNS services can interfere with PlayStation network checks. Switch your PS5 DNS to `1.1.1.1` or `8.8.8.8`.

### Incorrect IP Address Detected

If you use VPNs or virtual network adapters, the app might select the wrong local IP. Find your correct local IP manually:

* **Windows:** Run `ipconfig` in Command Prompt.
* **macOS / Linux:** Run `ifconfig` or `ip addr` in Terminal.

---

## Technical Overview

Black Ops II fetches emblem data via plain HTTP from Demonware servers. This tool acts as a local proxy targeting only those specific emblem endpoints:

* **Capture Mode:** Intercepts and downloads emblem assets from the server to your PC.
* **Show Mode:** Intercepts outgoing requests and feeds your selected local emblem back to the console.

All HTTPS traffic, PSN authentication, and matchmaking data pass through un-decrypted and untouched.
