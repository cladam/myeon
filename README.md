# myeon (면)

```
  _ __ ___  _   _  ___  ___  _ __  
 | '_ ` _ \| | | |/ _ \/ _ \| '_ \ 
 | | | | | | |_| |  __/ (_) | | | |
 |_| |_| |_|\__, |\___|\___/|_| |_|
            |___/                  
                                           
      the plane — space for focus
```

`myeon` is a minimalist, keyboard-driven TUI Kanban board built with **Rust** and **Ratatui**. While [ilseon](https://github.com/cladam/ilseon) is designed for **execution** (the single priority), `myeon` is designed for **perspective** (the landscape).

It provides a low-sensory, high-focus environment to triage ideas, organise projects, and decide what earns the right to become your "first line" of work.

## The Philosophy

For neurodivergent users, the "Big Picture" is often a source of anxiety, leading to task paralysis. `myeon` solves this by applying the **Ilseon Stillness** principles to a Kanban structure:

* **Low Sensory Load:** No flashing colors, no "Overdue" alarms, and no visual clutter. Just your tasks in a calm, spatial layout.
* **Contextual Silos:** Filter your entire board by "Work," "Life," or "Project" with a single keystroke. If it’s not relevant now, it doesn't exist.
* **The "Planning vs. Doing" Split:** Use `myeon` at your desk to organise the chaos. Use **ilseon** on your mobile to execute the result.

## Features (Planned)

* **Keyboard-First Navigation:** Vim-like bindings for speed and reduced cognitive load.
* **Zen Focus Mode:** Dim all columns except the one you are currently triaging.
* **WIP Soft-Caps:** Gentle visual cues when a column has too many items, encouraging you to finish instead of start.
* **Idea Landing Strip:** A dedicated column for "Exported" notes from the ilseon app.
* **Local-First & Private:** Your data stays on your machine in a simple, human-readable format.

## Installation

```bash
git clone https://github.com/your-username/myeon
cd myeon
cargo install --path .

```

## Keybindings

* `h/j/k/l`: Move focus across tasks and columns.
* `i`: Quick-capture a new idea into the Inbox.
* `Space`: Toggle **Zen Mode** (Focus on the current column).
* `c`: Change Context (Switch between Work/Personal/Side-project).
* `Enter`: Edit task details.


## About myeon

In the **ilseon** ecosystem, focus is sacred. 

Most Kanban boards are designed for complex project management, often resulting in "information density" that leads to cognitive overwhelm and task paralysis. **myeon** (Korean for *surface* or *plane*) is the spatial counterpart to the ilseon mobile app. 

While **ilseon** helps you walk the path (The Line), **myeon** helps you map the territory (The Plane). It provides a high-contrast, low-stimulation environment to organise your thoughts without the "noise" of traditional productivity software.
