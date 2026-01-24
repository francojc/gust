# Brainstorm: High-Res Interactive Weather TUI in Rust

> `gust` - A terminal-based weather dashboard that focuses on visual data density
> and high-resolution aesthetics, moving away from static tables to trend
> visualization.

Here is an overview, the recommended stack, and a development roadmap for your
robust Rust-based interactive weather TUI, named `gust`.

### 1. Application Overview: "gust"

**The Concept:**

A terminal-based weather dashboard that focuses on visual data density and high-resolution aesthetics. It moves away from static tables and focuses on **trend visualization**.

**The Interface Layout:**

*   **Header (Top 20%):**
    *   **Location Name:** Large text (e.g., "NEW YORK, NY").
    *   **Current Conditions:** Huge text for temperature (e.g., "72°F") and weather icon (using Nerd Font glyphs).
    *   **Metadata:** Feels Like, Wind Speed/Direction, Pressure.
*   **Main View (Middle 60%):**
    *   **The "Model Stage":** A large `Canvas` widget.
    *   **Tabs/Selector:** Interactive keys (1, 2, 3) to switch between
metrics:
        *   **Temp:** Smooth curve showing High/Low trends.
        *   **Precip:** Bar/Line hybrid showing rain probability and volume
(inches).
        *   **Humidity:** Area chart showing relative humidity %.
    *   **X-Axis:** Time (Days/Hours).
*   **Footer (Bottom 20%):**
    *   **Command Line:** An input field to type new city names (e.g., `>
London`).
    *   **Status Bar:** Data source (e.g., "Source: ECMWF (Open-Meteo)"), Last updated time, and keybindings guide.

**Key Interactions:**

*   **Tab/Shift-Tab:** Switch between Temp, Precip, Humidity graphs.
*   **Left/Right Arrows:** Pan the graph timeline (if zoomed in).
*   **`/` or `:`:** Enter "Command Mode" to search for a new location.
*   **`R`:** Refresh data immediately.
*   **`Q`:** Quit.

---

### 2. The Recommended Rust Stack

To handle Imperial conversions, async API calls, and high-res rendering, this
stack is the industry standard for Rust CLIs.


| Component | Crate/Library | Purpose |
| :--- | :--- | :--- |
| **TUI Framework** | `ratatui` | The core UI engine, widgets, and layout management. |
| **Backend** | `crossterm` | Handles terminal input (keyboard), mouse events, and screen resizing. |
| **Async Runtime** | `tokio` | Essential. Allows the UI to remain responsive while waiting for API responses. |
| **HTTP Client** | `reqwest` | Async HTTP client to pull data from weather APIs. |
| **Serialization** | `serde` + `serde_json` | Parsing the JSON responses from the API into Rust structs. |
| **Date/Time** | `chrono` | Handling the timestamps returned by the weather API. |
| **Geocoding** | `reqwest` (API call) | Converting "Place Name" (e.g., "Paris") to "Lat/Lon" required by weather APIs. |


**Data Provider Recommendation:**

Use **Open-Meteo**.

*   **Why?** It is free for non-commercial use, requires no API key, and offers access to the **ECMWF** and **GFS** models (Top Models).
*   **Imperial Support:** It allows you to request `&temperature_unit=fahrenheit` and `&wind_speed_unit=mph` directly in the URL, saving you from doing manual math in Rust.

---

### 3. Stages of Development

Here is the logical progression to build this application, from scratch to a
polished tool.

#### Stage 1: The Skeleton & Geocoding

**Goal:** Setup the project and solve the "Place Name" problem.

*   **Setup:** Initialize Cargo project, add `ratatui`, `tokio`, `reqwest`, `serde`.
*   **Geocoding Logic:** Write a function `get_coordinates(city: &str) -> (f64, f64)`. Use the Open-Meteo Geocoding API (or Google Maps API if you have a key).
*   **Data Structure:** Create a struct `Location` to hold the name and lat/lon.
*   **Milestone:** You can type `cargo run -- paris` in a terminal and it prints `48.85, 2.35`.

#### Stage 2: The Data Layer (The "Model")

**Goal:** Fetch and parse the weather data.

*   **Structs:** Define Rust structs using `serde` to deserialize the JSON response (Current weather, Hourly forecast arrays).
*   **API Client:** Write an async function that takes Lat/Lon and fetches the 3-5 day forecast.
*   **Unit Handling:** Configure the API request to send back Imperial units (Fahrenheit, Inches).
*   **Error Handling:** Ensure network errors or invalid cities return `Result` types effectively so the TUI doesn't crash.
*   **Milestone:** The application successfully fetches a vector of temperature data points for the next 5 days.

#### Stage 3: The Static UI (The "View")

**Goal:** Build the layout without the graphs yet.

*   **Layout:** Use `ratatui::layout::Constraints` to create the Header (Current Weather) and Footer (Input/Status).
*   **Widgets:**
    *   Use `Paragraph` for the City Name and Temperature.
    *   Use `Tabs` for the Temp/Precip/Humidity selector.
*   **Event Loop:** Setup the basic `crossterm` event loop to capture keystrokes (specifically `q` to quit).
*   **Milestone:** A black and white terminal window showing text data for a hardcoded city (e.g., London).

#### Stage 4: The High-Res Graph (The "Canvas")

**Goal:** Implement the visual center of the app.

*   **Canvas Setup:** Create a `Canvas` widget in the center layout.
*   **Metric Logic:**
    *   *Temp:* Draw a smooth `Line` graph in Yellow/Red.
    *   *Precip:* Draw vertical bars (using the `Rectangle` shape in Canvas context) in Blue.
    *   *Humidity:* Draw a filled area or line in Cyan.
*   **Scaling:** Write logic to normalize the data (e.g., map the temperature range [60, 85] to the canvas Y-bounds).
*   **Milestone:** You see a beautiful Braille-based graph rendering in the terminal when you run the app.

#### Stage 5: Interaction & State Management

**Goal:** Make the app alive.

*   **State Enum:** Create an `enum AppState { Running, Searching }`.
*   **Tab Switching:** Capture number keys (1, 2, 3) to swap the data source feeding the Canvas (switching the `draw` closure to render Temp vs Precip).
*   **Search Input:** When the user types `/`, switch `AppState` to `Searching`. Capture keystrokes and buffer them into a string. On `Enter`, trigger the Geocoding and API fetch functions from Stage 1 & 2.
*   **Loading States:** If the API is slow, render a "Loading..." spinner (using the `ratatui::widgets::Spinner` or a custom ASCII animation) so the user knows it's working.
*   **Milestone:** You can type "Tokyo", hit enter, and watch the graph update to Tokyo's weather live.

#### Stage 6: Polish & "Top Models"

**Goal:** Refine the user experience.

*   **Model Selection:** Add a feature to toggle between weather models (e.g., `best_match` vs `ecmwf` vs `gfs`) via a keybinding.
*   **Colors:** Define a cohesive color palette (e.g., a "Cyberpunk" or "Solarized" theme) applied to the graphs and text.
*   **Labels:** Add X-axis labels (Mon, Tue, Wed) using `Paragraph` widgets aligned below the Canvas, as Canvas text rendering is difficult.
*   **Milestone:** A robust, visually striking application that looks like a sci-fi movie interface.
