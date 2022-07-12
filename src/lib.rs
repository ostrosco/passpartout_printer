//! # Passpartout Printer
//!
//! This library aims to provide a programmatic way of drawing images into Passpartout: The
//! Starving Artist. Most users will want to use the passpartout_printer binary to draw
//! images into the game, but this library provides some finer control and allows drawing of
//! arbitrary shapes instead of reading from images.
//!

/// An interface to the colors that the game defines and provides a means of matching a given
/// color to the closest color that Passpartout provides.
pub mod colors;

/// A very simple coordinate system that supports basic mathematical operations.
pub mod coords;

/// An abstration around the easel in-game and methods to work with and draw onto it.
pub mod easel;

/// A set of functions to pull in images and draw them onto an easel.
pub mod image_drawer;

/// Methods for generating the configuration files for the passpartout_printer application.
pub mod manual_config;
