pub use self::data::*;
use std::mem::MaybeUninit;
use std::sync::Once;

mod data;
mod parsing;

pub fn get_track() -> &'static Track {
    static mut SINGLETON: MaybeUninit<Track> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(parsing::parse_track("res/tracks/test_track.json").unwrap());
        });

        SINGLETON.assume_init_ref()
    }
}
