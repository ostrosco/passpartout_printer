extern crate failure;
extern crate glutin;
extern crate mouse_coords;

use easel::EaselCoords;
use std::thread;
use std::time::Duration;
use failure::Error;
use self::mouse_coords::MouseQuery;

pub fn create_config(path: &str) -> Result<(), Error> {
    println!("This will walk you through creation of a configuration file.");
    println!("First, let's gather the portrait coordinates.");
    println!("This will reset the cursor after each click.");
    println!("Click when ready.");
    get_pos();

    println!("Please click on the upper left corner of the easel.");
    let portrait_ul = get_pos();

    println!("Please click on the lower right corner of the easel.");
    let portrait_lr = get_pos();

    println!(
        "Please click on the button to change from portrait to landscape."
    );
    let orientation = get_pos();

    println!("Please click on the upper left corner of the easel.");
    let landscape_ul = get_pos();

    println!("Please elick on the lower right corner of the easel.");
    let landscape_lr = get_pos();

    println!("Please click on the button to decrease brush size.");
    let decrease_brush = get_pos();

    println!("Please click on the button to increase brush size.");
    let increase_brush = get_pos();

    println!("Please click on the paintbrush tool.");
    let paintbrush = get_pos();

    println!("Please click on the spray can tool.");
    let spray_can = get_pos();

    println!("Please click on the pen tool.");
    let pen = get_pos();

    println!("Next, we'll be figuring out the distance between colors.");
    println!("Try to click in the center of the color.");
    println!("Please click on black.");
    let color_start = get_pos();

    println!("Please click on grey.");
    let grey = get_pos();

    println!("Please click on dark brown.");
    let dark_brown = get_pos();

    let color_row_step = dark_brown.0 - color_start.0;
    let color_col_step = grey.1 - color_start.1;

    let easel_coords = EaselCoords {
        portrait_bounds: (portrait_ul, portrait_lr),
        landscape_bounds: (landscape_ul, landscape_lr),
        paintbrush: paintbrush,
        spray_can: spray_can,
        pen: pen,
        decrease_brush: decrease_brush,
        increase_brush: increase_brush,
        change_orientation: orientation,
        color_start: color_start,
        color_row_step: color_row_step,
        color_col_step: color_col_step,
    };

    easel_coords.save(path)
}

pub fn get_pos() -> (i32, i32) {
    use glutin::ElementState;
    use glutin::Event::DeviceEvent;
    let mut events_loop = glutin::EventsLoop::new();
    let mut mouse_pos = (0, 0);
    let mouse_coords = mouse_coords::MouseCoords::new();

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let DeviceEvent { event, .. } = event {
                if let glutin::DeviceEvent::Button { button, state, .. } = event
                {
                    if button == 1 && state == ElementState::Pressed {
                        mouse_pos = mouse_coords.get_coords();
                        running = false;
                    }
                }
            }
        });
    }
    thread::sleep(Duration::from_secs(1));
    return mouse_pos;
}
