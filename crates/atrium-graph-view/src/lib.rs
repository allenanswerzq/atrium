//! # atrium-graph-view
//!
//! Visual specification for architecture graph rendering.
//!
//! This crate defines the **contract** between the Architecture Agent
//! (producer) and the UI renderer (consumer). The agent outputs a
//! [`ViewSpec`] — a complete description of what to draw and how —
//! and the renderer draws it verbatim.
//!
//! The renderer has **zero intelligence**: no layout, no filtering,
//! no styling decisions. It draws boxes at coordinates, lines along
//! paths, and wires mouse events to the actions specified per node.

mod spec;

pub use spec::*;
