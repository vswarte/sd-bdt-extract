pub const SD_KEY: &[u8] = include_bytes!("../data/sd_key.pub");
pub const SD_BHD_PATH: &str = "./sd/sd.bhd";
pub const SD_BDT_PATH: &str = "./sd/sd.bdt";

pub const PROGRESS_BAR_TEMPLATE: &str = "Extracting audio assets [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len}";

pub const DICTIONARY_OVERRIDE_PATH: &str = "./dictionary.txt";
pub const STANDARD_DICTIONARY: &str = include_str!("./standard_dictionary.txt");
