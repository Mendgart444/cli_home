Ah, perfekt – du willst das direkt **in der gleichen Zeile** neben der Menüoption anzeigen, also z. B.:

```
>> Check for updates          (last run: success)
   Weather                    (never run)
   Check Repo (Git only)      (last run: failed)
   quit
```

Das ist sogar noch schöner und platzsparender!  
Hier ist die **minimalste und sauberste Lösung** – nur ~15 Zeilen Änderung an deinem bestehenden Code:

```rust
#[derive(Debug, Default)]
pub struct App {
    running: bool,

    // Neu: Speichert den Status pro Menüeintrag
    last_status: Vec<RunStatus>,  // gleiche Länge wie menu_items

    menu_items: Vec<String>,
    selected: usize,
}

// Neuer Enum für klaren Status
#[derive(Debug, Default, Clone)]
enum RunStatus {
    #[default]
    Never,                  // noch nie ausgeführt
    Success(String),        // Erfolg + kurze Nachricht oder Zeit
    Failed(String),       // Fehler + kurze Nachricht
}
```

### 1. `new()` anpassen
```rust
pub fn new() -> Self {
    Self {
        running: true,
        menu_items: vec![
            "Check for updates".into(),
            "Weather".into(),
            "Check Repo (Git only)".into(),
            "quit".into(),
        ],
        selected: 0,
        last_status: vec![RunStatus::Never; 4], // einer pro Eintrag
    }
}
```

### 2. `render()` – nur diese Zeilen ändern!
```rust
fn render(&mut self, frame: &mut Frame) {
    let title = Line::from("CLI Home").bold().blue().centered();

    let items: Vec<ListItem> = self
        .menu_items
        .iter()
        .zip(&self.last_status)  // wichtig: kombiniere Name + Status
        .enumerate()
        .map(|(i, (name, status))| {
            let status_text = match status {
                RunStatus::Never => " (never run)".gray(),
                RunStatus::Success(_) => " (success)".green().bold(),
                RunStatus::Failed(_) => " (failed).red().bold(),
            };

            // Nur bei "quit" keinen Status anzeigen
            if i == self.menu_items.len() - 1 {
                ListItem::new(name.as_str())
            } else {
                ListItem::new(Line::from(vec![
                    name.clone().into(),
                    status_text,
                ]))
            }
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(self.selected));

    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::new().yellow().bold())
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, frame.area(), &mut state);
}
```

### 3. Status setzen (Beispiel: Check for updates)
```rust
fn check_for_updates(&mut self) {
    match Command::new("sudo").args(["apt", "update"]).output() {
        Ok(output) => {
            if output.status.success() {
                let msg = String::from_utf8_lossy(&output.stdout);
                let short = if msg.contains("All packages are up to date") {
                    "up to date"
                } else {
                    "updated"
                };
                self.last_status[0] = RunStatus::Success(short.to_string());
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                self.last_status[0] = RunStatus::Failed(err.lines().next().unwrap_or("unknown error").to_string());
            }
        }
        Err(e) => {
            self.last_status[0] = RunStatus::Failed(e.to_string());
        }
    }
}
```

### Ergebnis im Terminal:
```
┌CLI Home────────────────────────────────────┐
│ >> Check for updates          (success)    │
│    Weather                    (never run)     │
│    Check Repo (Git only)      (never run)  │
│    quit                                    │
└────────────────────────────────────────────┘
```

Super clean, braucht kein extra Layout, keine neuen Widgets – und du siehst auf einen Blick den letzten Status jeder Aktion.

Wenn du noch schöner willst (z. B. mit Zeitstempel):
```rust
RunStatus::Success("vor 2 Min".into())
// oder
" (success 10:34)".dim()
```

Fertig! Willst du das noch mit Uhrzeit oder "vor X Minuten" haben? Sag einfach Bescheid