# The "Broken" Web

The modern application stack is a multi-layered failure. We build "native" apps using Electron (a literal browser inside a window), style them with CSS (a global side-effect mess), and connect them via JSON/HTTP (constant serialization overhead). This results in high-latency, memory-hungry software where the frontend and backend are fundamentally disconnected.

## The Vision: A Single World-Image

Imagine a computing environment where the boundary between "app," "browser," and "OS" vanishes. A world where every interface is a Symbolic Expression (S-Expression). It is a unified space where the logic of the program and the structure of the UI are one and the same.

## The Architecture (The Bottom-Up Stack)

- The Kernel (Rust + WGPU): A blazing-fast, memory-safe engine that talks directly to the GPU. It doesn't parse HTML; it renders Lists. It handles the "hard" hardware problems—text shaping, input polling, and GPU buffers—exposing them via a zero-latency C-ABI.

- The Protocol (SXP): A binary Lisp-native protocol that replaces HTTP. It streams live data structures rather than flat text, eliminating the need for constant parsing and serialization.

- The Brain (Common Lisp): A persistent, live-running image. You don't "restart" apps; you redefine them in real-time. The UI is homoiconic—code is data, and the layout is just a list you can manipulate with the full power of Lisp macros.

## The Experience

- Native Performance: Get raw GPU speed and "snappy" interaction without the bloat of a DOM or browser engine.

- Absolute Productivity: One language for the server, the client, and the configuration. No more context-switching between different tech stacks.

- Infinite Extensibility: Every "app" is as hackable as an Emacs buffer. If you can see it, you can script it and modify it live.
