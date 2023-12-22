#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

mod logging;
use rtic::app;

#[app(device = microbit::pac, peripherals = true, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    //use crate::button_pressed_action;
    use crate::log_count;
    use crate::logging;
    use crate::compose_string;
    // NOTE: The defmt version of these macros will log the panic message using defmt
    // and then call core::panic!, so the rtt message will be emitted before panic is invoked
    #[cfg(feature = "use_defmt")]
    use panic_halt as _;

    #[cfg(feature = "use_rtt")]
    use rtt_target::{rtt_init_print};

    #[cfg(feature = "use_rtt")]
    use panic_rtt_target as _;

    use microbit::board::Board;
    use microbit::hal::gpiote::Gpiote;
    use microbit::display::blocking::Display;
    use microbit::hal::Timer;
    use microbit::hal::pac::TIMER0;
    use heapless::String;

    #[shared]
    struct Shared {
        gpiote  : Gpiote,
        display : Display,
        timer   : Timer<TIMER0>,
        key     : String<32>
    }

    #[local]
    struct Local {
        //idle           : u32,
        button_pressed : u32,
        button_a       : u32,
        button_b       : u32,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        #[cfg(feature = "use_rtt")]
        rtt_init_print!();

        logging::test_print("in init");

        let board = Board::new(cx.device, cx.core);

        let display = Display::new(board.display_pins);
        let timer = Timer::new(board.TIMER0);

        let gpiote = Gpiote::new(board.GPIOTE);
        let chan0 = gpiote.channel0();
        chan0.input_pin(&board.buttons.button_a.degrade())
            .hi_to_lo()
            .enable_interrupt();
        let chan1 = gpiote.channel1();
        chan1.input_pin(&board.buttons.button_b.degrade())
            .hi_to_lo()
            .enable_interrupt();

        (
            Shared {
                gpiote,
                display,
                timer,
                key : String::from("hello"),
            },
            // TODO: precompute the led states for button presses and add them as locals
            Local {
                button_a       : 0,
                button_b       : 0,
                button_pressed : 0
            }
        )
    }

    #[task(binds = GPIOTE, priority = 3, shared = [gpiote], local = [button_pressed])]
    fn button_pressed(mut ctx : button_pressed::Context) {
        let button_pressed_count = ctx.local.button_pressed;
        *button_pressed_count += 1;
        log_count("button pressed count: ", *button_pressed_count);

        ctx.shared.gpiote.lock(|gpiote| {
            let chan0 = gpiote.channel0();
            let chan1 = gpiote.channel1();

            if chan0.is_event_triggered() {
                logging::print("Button A pressed");
                chan0.reset_events();
                match button_a_action::spawn() {
                    Ok(()) => (),
                    Err(()) => logging::print("failed to spawn task!"),
                }
            }

            if chan1.is_event_triggered() {
                logging::print("Button B pressed");
                chan1.reset_events();
                match button_b_action::spawn() {
                    Ok(()) => (),
                    Err(()) => logging::print("failed to spawn task!"),
                }
            }

            //button_pressed_action(chan0, "A");
            //button_pressed_action(chan1, "B");
        });
    }

    #[task(priority = 1, shared = [display, timer], local = [button_a])]
    async fn button_a_action(ctx : button_a_action::Context) {
        let button_a_count = ctx.local.button_a;
        *button_a_count += 1;
        log_count("Task A count: ", *button_a_count);

        let mut display = ctx.shared.display;
        let mut timer = ctx.shared.timer;

        let leds_empty = [[0; 5]; 5];
        let mut leds = leds_empty;
        for x in 0..5 {
            for y in 0..5 {
                leds[x][y] = 1;
            }
            (&mut display, &mut timer).lock(|d, t| {
                d.show(t, leds, 400);
            });
            leds = leds_empty;
        }
    }

    #[task(priority = 2, shared = [display, timer], local = [button_b])]
    async fn button_b_action(ctx : button_b_action::Context) {
        let button_b_count = ctx.local.button_b;
        *button_b_count += 1;
        log_count("Task B count: ", *button_b_count);

        let mut display = ctx.shared.display;
        let mut timer = ctx.shared.timer;

        let leds_empty = [[0; 5]; 5];
        let mut leds = leds_empty;
        for x in 0..5 {
            for y in 0..5 {
                leds[y][x] = 1;
            }
            (&mut display, &mut timer).lock(|d, t| {
                d.show(t, leds, 400);
            });
            leds = leds_empty;
        }
    }

    // NOTE: local variable declared here.
    // This does not require the local variable to implement the Send trait.
    #[idle(shared = [display, timer, &key], local = [idle_count : u32 = 0])]
    fn idle(mut ctx : idle::Context) -> ! {
        let idle_count = ctx.local.idle_count;

        logging::print("idling...");
        logging::print(
            compose_string::<32>(
                &[ "The key is: "
                // NOTE: accessing a shared resource without locking
                // ... possible because its a reference
                 , ctx.shared.key.as_str()]
                 ).unwrap().as_str());

        let leds_empty = [[0; 5]; 5];
        let mut leds = leds_empty;
        let led_states = [ (1,1), (1,2), (1,3), (2,3), (3,3), (3,2), (3,1), (2,1) ];
        loop {
            for (x,y) in led_states {
                leds[x][y] = 1;
                ctx.shared.display.lock(|display| {
                    ctx.shared.timer.lock(|timer| {
                        display.show(timer, leds, 250)
                    })
                });
                leds = leds_empty;
            }
            *idle_count += 1;
            log_count("Idle count: ", *idle_count);
        }
    }
}

use microbit::hal::gpiote::GpioteChannel;
use heapless::String;
use core::fmt::Write;

fn button_pressed_action(chan : GpioteChannel, button_name : &str) {
    if chan.is_event_triggered() {
        let message =
            compose_string::<32>(
                &["Button ", button_name, " has been pressed"])
            .unwrap();
        logging::print(message.as_str());
        chan.reset_events()
    }
}

fn compose_string<const N : usize>(xs : &[&str]) -> Result<String<N>, ()> {
    let mut s = String::<N>::new();
    for x in xs {
        s.push_str(x)?;
    }
    Ok(s)
}

fn log_count(message : &str, count : u32) {
    let mut s = String::<10>::new();
    write!(&mut s, "{}", count).unwrap();
    let message =
        compose_string::<32>(&[message, s.as_str()])
        .unwrap();
    logging::print(message.as_str());
}
