# Lambda

A unified computing environment where Browser, Emacs, and native applications fuse into one. Everything is Common Lisp. A native Rust/WGPU kernel handles windowing and GPU rendering via FFI. Users write declarative Lisp UIs and never touch the low-level machinery.

## The Problem

The modern application stack is a multi-layered failure. We build "native" apps using Electron (a literal browser inside a window), style them with CSS (a global side-effect mess), and connect them via JSON/HTTP (constant serialization overhead). This results in high-latency, memory-hungry software where the frontend and backend are fundamentally disconnected.

## The Vision

A computing environment where the boundary between "app," "browser," and "OS" vanishes. Every interface is a Symbolic Expression. The logic of the program and the structure of the UI are one and the same. You don't "restart" apps; you redefine them in real-time. Every "app" is as hackable as an Emacs buffer.

---

## Architecture Decisions

### Stack

| Component | Choice | Rationale |
|---|---|---|
| GPU Rendering | Rust + WGPU | Memory-safe, direct GPU access, instanced rendering |
| Windowing | winit (pump_events polling mode) | Pure Rust, seamless WGPU integration, Lisp drives the loop |
| Lisp Implementation | SBCL | Best performance (native x86-64 compilation), real OS threads, excellent FFI via CFFI |
| FFI | CFFI (Lisp) <-> C-ABI (Rust `#[no_mangle] extern "C"`) | Standard, zero-overhead interop |
| Layout Engine | Pure Common Lisp | Maximum hackability -- layout is just Lisp code users can override |
| UI DSL | Declarative with keyword props | `(box :padding 10 (button :onclick ...))` -- HTML-like but Lisp-native |

### Threading Model: Single-Threaded Rendering, Multi-Threaded Background

The rendering pipeline (poll events -> update state -> layout -> encode -> submit to GPU) runs on a single main thread driven by Common Lisp. Rust is called via FFI for event polling and GPU submission -- it never owns the thread.

SBCL has real POSIX threads (unlike Emacs Lisp which has cooperative concurrency). Background work (network, file I/O, heavy computation) spawns on worker threads. The Swank/Slime server runs on its own thread for live REPL access.

There is no need for dual-threaded rendering (Rust thread + Lisp thread with shared queues). The single-threaded model eliminates all synchronization complexity. If Lisp needs to do heavy work, it spawns a thread -- the main loop stays fast.

```
Main thread:     poll-events -> dispatch -> (app) -> layout -> encode -> submit -> present
Swank thread:    REPL, live redefinition, inspection
Worker threads:  SXP network, file I/O, search, computation (spawned as needed)
```

### Event Loop: Lisp Drives, Rust Serves

Lisp owns the main loop. Rust exposes 5 C-ABI functions:

| Function | Purpose |
|---|---|
| `lambda_init(w, h, title)` | Create window + GPU context |
| `lambda_poll_events() -> *Event, count` | Return pending OS events |
| `lambda_submit_frame(*cmds, count)` | Upload draw commands to GPU |
| `lambda_present()` | Swap buffers, show frame |
| `lambda_measure_text(str, size) -> w, h` | Text measurement for layout |

```lisp
(loop until (kernel:should-close-p)
  (let ((events (kernel:poll-events)))
    (process-events events))
  (kernel:submit-frame (encode (layout (app) w h)))
  (kernel:present))
```

### Layout Engine: Common Lisp, Simple-First

Layout runs in pure Common Lisp. Phase 1 uses full tree rebuild on state change. Fine-grained reactivity (signals, dependency tracking, incremental re-layout) is deferred to a later optimization phase.

Parallel layout across sibling subtrees (via `lparallel` thread pool) can be added later since SBCL supports real threads.

### UI DSL: Declarative with Keyword Props

Similar to HTML's declarative model but Lisp-native. Event handlers like `:onclick` hold closures. Deviates from HTML where it doesn't make sense for performance or ergonomics.

A Lambda page (`.sxp` file) is just the UI. No boilerplate, no main function, no wrapper. Like an HTML file is just markup, an SXP file is just Lisp:

```lisp
;; counter.sxp -- this is the entire file

(defvar *count* 0)

(box :padding 20 :gap 10 :align :center
  (text :size 32 :color :white
    (format nil "Count: ~a" *count*))
  (box :direction :row :gap 10
    (button :onclick (lambda () (decf *count*))
      (text "-"))
    (button :onclick (lambda () (incf *count*))
      (text "+"))))
```

The runtime loads the file, evaluates all top-level forms, and the last form returning an element becomes the root. For re-rendering, the runtime wraps that last element-returning form in a thunk automatically -- it gets re-evaluated each frame so the UI reflects current state. The user never writes that thunk.

Under the hood, `box`, `text`, `button` are functions that return element structs. The user never sees FFI, event pointers, render encoding, or any system internals.

### Page Loading Model

Loading `counter.sxp` is internally equivalent to:

```lisp
;; What the runtime does:
(defvar *count* 0)                              ;; evaluated once
(setf *root-thunk*
      (lambda ()                                ;; wrapped automatically
        (box :padding 20 :gap 10 :align :center
          (text :size 32 :color :white
            (format nil "Count: ~a" *count*))
          (box :direction :row :gap 10
            (button :onclick (lambda () (decf *count*))
              (text "-"))
            (button :onclick (lambda () (incf *count*))
              (text "+"))))))
```

Top-level `defvar`, `defun`, `defstruct`, etc. are evaluated once for side effects. The last form that returns an element is the page's root. State lives in normal Lisp variables. The full language is available -- macros, CLOS, conditions, the entire CL standard.

---

## Internal Architecture

### Element System

```lisp
(defstruct element
  tag        ; :box, :text, :button, etc.
  props      ; plist -- :onclick, :padding, :color, etc.
  children   ; vector of child elements
  ;; Filled in by layout pass:
  x y width height)
```

Element constructors parse `(:key val :key val child child...)` syntax:

```lisp
(defun box (&rest args)
  (multiple-value-bind (props children) (parse-props-and-children args)
    (make-element :tag :box :props props :children children)))
```

### Event Dispatch

Hit-testing walks the layout tree to find the deepest element containing the click coordinates, then calls the appropriate handler from its props (`:onclick`, `:onhover`, etc.). The user's closures are called directly -- no event objects to decode.

### Render Encoding

The laid-out element tree is walked to produce a flat array of draw commands (rect type, x, y, w, h, color) written into a foreign memory buffer. Rust reads this buffer and issues instanced GPU draw calls.

### Performance Path (Future)

When optimization is needed:

- **Incremental re-rendering:** Reactive signals (`defsignal`) with dependency tracking. Only dirty nodes re-evaluate.
- **Parallel layout:** `lparallel` work-stealing pool across sibling subtrees.
- **Zero-copy render buffer:** SBCL writes directly into shared foreign memory via `sb-sys:sap-ref-single`. Double-buffered.
- **SBCL type declarations:** `(optimize (speed 3))`, typed struct slots, vectors for children (cache locality), arena allocation to reduce GC pressure.
- **GPU batching:** Instanced rendering (all rects in one draw call), glyph atlas for text, sorted by texture/shader.

---

## File Structure

```
lambda/
├── kernel/                  # Rust crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # C-ABI exports
│       ├── window.rs        # winit setup, event polling
│       ├── renderer.rs      # wgpu pipeline, instanced drawing
│       └── text.rs          # glyph atlas, text shaping (Phase 7)
│
├── runtime/                 # Common Lisp (ASDF system)
│   ├── lambda.asd           # system definition
│   ├── packages.lisp        # package declarations
│   ├── kernel-ffi.lisp      # CFFI bindings to Rust .so
│   ├── element.lisp         # defstruct element, box, text, button
│   ├── layout.lisp          # layout engine
│   ├── render.lisp          # encode elements -> draw commands
│   ├── events.lisp          # hit testing, event dispatch
│   └── main.lisp            # main loop, startup
│
├── PLAN.md
└── README.md
```

---

## Build Phases

### Phase 1: Rust Kernel Crate

- winit window with `pump_events` (polling mode -- does not own the thread)
- wgpu pipeline that renders instanced colored rectangles
- C-ABI exports: `lambda_init`, `lambda_poll_events`, `lambda_submit_frame`, `lambda_present`
- Draw command struct: `{ type: u8, x: f32, y: f32, w: f32, h: f32, color: u32 }`
- Validate: call from a Rust test, see rectangles on screen

### Phase 2: SBCL + CFFI Bridge

- ASDF system definition for the Lisp side
- Load Rust `.so` via CFFI
- Thin Lisp wrappers: `(kernel:init w h title)`, `(kernel:poll-events)`, etc.
- Validate: open a window and draw a rect from the REPL

### Phase 3: Element Tree (The User API)

- `defstruct element` with tag, props (plist), children (vector)
- Functions: `box`, `text`, `button`, `input`
- `parse-props-and-children` for `(:key val child child...)` syntax
- Event handler props: `:onclick`, `:onhover`, `:onkeydown` hold closures

### Phase 4: Layout Engine

- Single-pass box layout: vertical/horizontal stacking
- Props: `:width`, `:height`, `:padding`, `:gap`, `:direction`, `:align`
- Constraint-based: parent passes available width/height down
- Output: each element gets x, y, w, h filled in
- Type-declared, `(optimize (speed 3))` on hot paths

### Phase 5: Render Encoding + Main Loop

- Walk laid-out tree, emit draw commands (rects)
- Wire up: poll -> dispatch -> (app) -> layout -> encode -> submit -> present
- First visual: a Lisp-defined UI rendered on screen

### Phase 6: Event Dispatch + Interactivity

- Decode Rust events (mouse click, move, keyboard)
- Hit-test: walk layout tree, find deepest element at (x, y)
- Call the element's `:onclick` / `:onhover` / etc.
- State change triggers re-render next frame
- The counter demo works end-to-end

### Phase 7: Text Rendering

- Rust side: font loading, glyph rasterization, texture atlas (fontdue or cosmic-text)
- Glyph atlas texture, text drawn as textured quads
- C-ABI: `lambda_measure_text(str, size) -> w, h` for Lisp layout to use
- Draw command type `TEXT` with glyph info
- Text wrapping in layout engine

### Phase 8: Live Development Experience

- Auto-start Swank server on boot
- Redefining `(defun app ...)` in SLIME/Sly updates the screen next frame
- In-window error display (conditions caught, rendered as red overlay)
- Inspector: click element, REPL prints its struct

### Phase 9: SXP Protocol + Networking

- Binary S-expression protocol over TCP
- Lisp image serves SXP pages to other Lambda instances
- Equivalent of "visiting a website" but it streams live Lisp structures
- Navigation, back/forward, bookmarks -- all in Lisp
