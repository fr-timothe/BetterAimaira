[← Documentation index](README.md)

# Integrations specification

BetterAimaira connects intranet data with external applications and operating system features.

Nothing here ships today. Both sections are specifications for work that has not
started: no `.ics` endpoint, widget or tray surface exists in the codebase yet.

---

## 1. Dynamic iCal calendar sync — planned

- Synchronize Aimaira timetables with external calendar applications (Google Calendar, Apple Calendar, Microsoft Outlook, Thunderbird).
- Local HTTP Server / Webcal: Tauri exposes a `.ics` endpoint (for example, `webcal://localhost:14201/calendar.ics?token=...`).
- RFC 5545 compliance: Generates standard `VEVENT` entries with `SUMMARY` (course name), `LOCATION` (room number), `DESCRIPTION` (instructor and course type), and `RRULE` where recurring.
- Configurable synchronization intervals.

---

## 2. Desktop and mobile widgets — planned

| Platform | Widget Type | Features |
|---|---|---|
| **Android / iOS** | Home Screen Widget (`Glance` / `WidgetKit`) | Next class name, room, countdown, and attendance status |
| **Windows** | System Tray and Taskbar badge | Current class, next room, attendance quick view |
| **macOS** | Menu Bar Item | Compact countdown display (for example, `Next: B204 in 12m`) |
