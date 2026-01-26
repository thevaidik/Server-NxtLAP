# TheSportsDB API Fields Reference

This document lists the data fields available from the **TheSportsDB Source API** (`eventsseason.php`). We currently assume only a subset of these, but others are available for future features.

## ✅ Core Fields (Currently Used/Available)

| Field | Example Value | Description |
| :--- | :--- | :--- |
| `idEvent` | `"2225616"` | Unique Event ID |
| `strEvent` | `"Chinese Grand Prix Sprint"` | Name of the event |
| `strLeague` | `"Formula 1"` | Name of the series/league |
| `strSeason` | `"2025"` | Season year |
| `dateEvent` | `"2025-03-22"` | Date of event (Local) |
| `strTime` | `"15:30:00"` | Start time (Local or UTC) |
| `strTimestamp`| `"2025-03-22T15:30:00"` | ISO 8601 Timestamp (Primary Source) |
| `strSport` | `"Motorsport"` | Sport Category (Used for filtering) |
| `strDescriptionEN`| `"The season’s first sprint..."` | Detailed text description |
| `strVenue` | `"Shanghai International Circuit"` | Name of the track |
| `strCountry` | `"China"` | Host country |

## 🖼️ Visuals (Available but Not Stored)

These URLs point to images hosted by TheSportsDB.

| Field | Description |
| :--- | :--- |
| `strThumb` | **Thumbnail:** 16:9 Event image (e.g., car on track) |
| `strPoster` | **Poster:** Vertical poster image (often null for races) |
| `strBanner` | **Banner:** Horizontal banner |
| `strSquare` | **Square:** 1:1 Icon or League Logo |
| `strMap` | **Circuit Map:** Layout of the track |
| `strFanart` | **Fanart:** High-res background image |

## ℹ️ Extra Details (Available)

| Field | Example Value | Description |
| :--- | :--- | :--- |
| `strCity` | `"Shanghai"` | City where venue is located |
| `intRound` | `"2"` | Round number in the championship |
| `strStatus` | `"Match Finished"` | Current status (`Not Started`, `Live`) |
| `strResult` | `"Verstappen (P1)..."` | Text summary of results (Post-race) |
| `strVideo` | `youtube.com/...` | link to highlights (sometimes) |
| `strPostponed`| `"no"` | If event is postponed |
| `strLocked` | `"unlocked"` | Internal DB status |
