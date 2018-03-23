# Passpartout Printer

Draws images into the game [Passpartout: The Starving Artist](http://store.steampowered.com/app/582550/Passpartout_The_Starving_Artist/).

![Drawing our favorite crab. Original draw time 7 minutes.](https://thumbs.gfycat.com/SevereViciousBittern-size_restricted.gif)

This application has one of two main operating modes:

* Image Drawing Mode: takes an input image and draws it to the easel in-game.

* Shape Drawing Mode: takes a list of coordinates and draws that shape in-game.

# Downloading and Compiling

```
git clone https://github.com/ostrosco/passpartout_printer.git
cd passpartout_printer
cargo build --release
```

# Configuration

Before the application can draw to the easel in-game, a configuration
file needs to be created with the screen coordinates of in-game elements.
To do this:

* Start Passpartout.
* Select Endless Mode in the main menu. Pick any of the unlocked scenarios.
* Click on the easel to bring up the drawing interface.
* Run `cargo run --release -- --configure` to start the configuration process.

A prompt will appear in the console to walk through the elements to click
to configure the program. Upon completion, a "coords.json" file will be created
in the top-level directory.

## Configuration Tips

* It's better to click on the white part of the easel than along the edge. If
  a click is too far off the easel, some line draws starting or ending near the
  edges will fail.
* Try to click on the direct center of the colors.


# Usage

```
Passpartout Printer 1.0.0

USAGE:
    passpartout_printer [FLAGS] [OPTIONS]

FLAGS:
        --configure        Configures the application with coordinates in-game.
        --enable-dither    Enables dithering to reduce color banding but increase draw time
    -h, --help             Prints help information
        --no-scale         Disable scaling of the input image.
    -V, --version          Prints version information

OPTIONS:
    -i, --image <IMAGE>        Input image to use
    -w, --mouse-wait <WAIT>    Specify the time to wait between mouse actions
```

# Deciding on a Wait Time

The speed in which passpartout_printer can draw to the easel is limited by the
frame rate in-game. The default wait time between mouse operations is 7ms which
assumes a frame rate of around 142 FPS. To measure your in-game frame rate,
ensure that the Steam overlay is enabled and that the In-Game FPS Counter is
enabled. Initial observations have shown that the FPS can drop between 5% to
15% when drawing, so it's best to go a little slower than the max FPS.
Wait times of 5ms or less seem to cause severe input errors regardless of FPS.

- TODO: include a screenshot of how to turn on the FPS counter.

# Dithering

[Dithering](https://en.wikipedia.org/wiki/Dither) can greatly improve the image
quality at a significant performance tradeoff. Below are two draws of the same
image: one without dithering and one with dithering. Expect dithering to
increase the draw time of an image by 2 to 4 times.

- TODO: draw the Mona Lisa with and without dithering. Note the draw times
given a particular wait time.

# Examples

There are examples in the project that demonstrate the Shape Drawing Mode of
the application. To run them:

```
cargo run --release --example draw_house
```
