//! A random name generator to identify clients.

use rand;

static ADJECTIVES: &[&str] = &[
    "accidental",
    "accurate",
    "ancient",
    "animated",
    "boundless",
    "bright",
    "capable",
    "careful",
    "charming",
    "chivalrous",
    "classy",
    "clever",
    "cluttered",
    "crowded",
    "cuddly",
    "cultured",
    "defiant",
    "diligent",
    "efficient",
    "enchanted",
    "endurable",
    "entertaining",
    "enthusiastic",
    "exuberant",
    "fabulous",
    "friendly",
    "glorious",
    "groovy",
    "hilarious",
    "holistic",
    "honorable",
    "inquisitive",
    "instinctive",
    "invincible",
    "knowledgeable",
    "literate",
    "luxuriant",
    "nebulous",
    "obsequious",
    "overjoyed",
    "periodic",
    "polite",
    "quizzical",
    "serious",
    "sharp",
    "shiny",
    "silent",
    "skillful",
    "splendid",
    "spotless",
    "steady",
    "sturdy",
    "successful",
    "succinct",
    "swanky",
    "terrific",
    "zany",
];

static NOUNS: &[&str] = &[
    "aardvark",
    "alpaca",
    "badger",
    "bear",
    "beaver",
    "buffalo",
    "butterfly",
    "camel",
    "caribou",
    "cheetah",
    "chimpanzee",
    "crow",
    "dinosaur",
    "dolphin",
    "elephant",
    "giraffe",
    "goldfish",
    "grasshopper",
    "kangaroo",
    "koala",
    "lion",
    "horse",
    "mallard",
    "manatee",
    "monkey",
    "moose",
    "mouse",
    "panda",
    "platypus",
    "porcupine",
    "rabbit",
    "raccoon",
    "reindeer",
    "rhinoceros",
    "snail",
    "squirrel",
    "swan",
    "tiger",
    "turkey",
    "walrus",
    "zebra",
];

/// Generate a random name
pub fn generate() -> String {
    let adjective = ADJECTIVES[rand::random::<usize>() % ADJECTIVES.len()];
    let noun = NOUNS[rand::random::<usize>() % NOUNS.len()];

    // Uppercase first letter
    fn upper_first(word: &str) -> String {
        if word.len() == 0 {
            String::new()
        } else {
            word.chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect()
        }
    }
    let adjective = upper_first(adjective);
    let noun = upper_first(noun);

    format!("{} {}", adjective, noun)
}
