use heapless::String;

#[cfg(feature = "use_defmt")]
use defmt_rtt as _;

#[cfg(feature = "use_defmt")]
use defmt as _;

#[cfg(feature = "use_rtt")]
use rtt_target as _;

pub fn print(s : &str) {
    // Log using the appropriate method
    #[cfg(feature = "use_defmt")]
    defmt::println!("{}", s);

    #[cfg(feature = "use_rtt")]
    rtt_target::rprintln!("{}", s);
}

pub fn test_print(s : &str) {
    #[cfg(feature = "use_defmt")]
    let prefix : &str = "defmt: ";
    #[cfg(feature = "use_rtt")]
    let prefix : &str = "rtt: ";

    let mut concatenated: String<64> = String::new();
    concatenated.push_str(prefix).unwrap();
    concatenated.push_str(s).unwrap();

    print(concatenated.as_str())
}
