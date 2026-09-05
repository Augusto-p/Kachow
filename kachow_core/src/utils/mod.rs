use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct Utils;

impl Utils {
    pub fn generate_random_string(long: u8) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(long as usize)
            .map(char::from)
            .collect()
    }
}
