# Integrations specification

BetterAimaira connects intranet data with external applications and operating system features.

---

## 1. Dynamic iCal calendar sync

- Synchronize Aimaira timetables with external calendar applications (Google Calendar, Apple Calendar, Microsoft Outlook, Thunderbird).
- Local HTTP Server / Webcal: Tauri exposes a `.ics` endpoint (for example, `webcal://localhost:14201/calendar.ics?token=...`).
- RFC 5545 compliance: Generates standard `VEVENT` entries with `SUMMARY` (course name), `LOCATION` (room number), `DESCRIPTION` (instructor and course type), and `RRULE` where recurring.
- Configurable synchronization intervals.

---

## 2. Discord and Telegram webhooks

Automated alerts can be configured for Discord or Telegram channels:

1. **New grade notifications.** Triggered when new grades appear during application grade checks, sending subject, grade, coefficient, and updated average.
2. **Schedule change alerts.** Triggered when class times or rooms change, sending a formatted diff summary.

---

## 3. Desktop and mobile widgets

| Platform | Widget Type | Features |
|---|---|---|
| **Android / iOS** | Home Screen Widget (`Glance` / `WidgetKit`) | Next class name, room, countdown, and attendance status |
| **Windows** | System Tray and Taskbar badge | Current class, next room, attendance quick view |
| **macOS** | Menu Bar Item | Compact countdown display (for example, `Next: B204 in 12m`) |

---

## 4. In-app grade alerts

- Triggered on authenticated application launch when `/Note` is fetched and compared with SQLite grade fingerprints.
- The initial sync stores existing grades without alert popups.
- Subsequent syncs display new grades in a Home banner and notification drawer.
- Grade checks run only while the application is active; no background daemon, cloud relay, or push service is used.
