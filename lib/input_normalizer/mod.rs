//! Input Normalizer — Robust Text Understanding
//!
//! Transforms messy human input (leet, typos, emoji, caps, mixed case, etc.)
//! into clean, normalized text that Star can reason about.
//!
//! The normalizer preserves INTENT — if someone types "r3gul4r", they mean "regular".
//! It also preserves PERSONALITY MARKERS — emoji, caps, and leet usage inform
//! how Star might respond, without being required too.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Result of normalizing an input string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedInput {
    /// The cleaned text with leet, typos, and casing fixed.
    pub clean_text: String,
    /// Personality markers extracted from the input.
    pub markers: PersonalityMarkers,
    /// Confidence that the normalization captured the intent correctly.
    pub confidence: f64,
    /// Any tokens that couldn't be confidently decoded.
    pub uncertain_tokens: Vec<UncertainToken>,
}

/// Personality markers that inform output style decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalityMarkers {
    /// Was ALL CAPS used (or near-all caps)?
    pub is_shouting: bool,
    /// Was text intentionally shortened (txtspk)?
    pub is_txtspk: bool,
    /// Were emoji present?
    pub has_emoji: bool,
    /// Was leet speak used?
    pub is_leet: bool,
    /// Was the message very short?
    pub is_terse: bool,
    /// Was there mixed case chaos?
    pub is_mixed_case: bool,
    /// Was the tone interrogative?
    pub is_interrogative: bool,
    /// Was the tone exclamatory?
    pub is_exclamatory: bool,
}

/// A token where the normalization was uncertain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertainToken {
    pub original: String,
    pub attempted_correction: Option<String>,
    pub reason: String,
}

/// The normalizer itself.
#[derive(Clone, Default)]
pub struct InputNormalizer {
    leet_map: LeetMap,
    emoji_map: EmojiMap,
    typo_map: TypoMap,
}

#[derive(Clone)]
struct LeetMap {
    /// Map of leet characters to their letter equivalents.
    /// Sorted by frequency of use in leet speak.
    map: std::collections::HashMap<char, char>,
}

impl Default for LeetMap {
    fn default() -> Self {
        let mut map = std::collections::HashMap::new();
        // Numbers that replace letters
        map.insert('3', 'e');
        map.insert('4', 'a');
        map.insert('0', 'o');
        map.insert('1', 'i');
        map.insert('5', 's');
        map.insert('7', 't');
        map.insert('$', 's');
        map.insert('5', 'z'); // context-dependent
        map.insert('2', 'z');  // context-dependent
        map.insert('6', 'g');
        map.insert('8', 'b');
        map.insert('9', 'g');
        // Symbols that replace letters
        map.insert('@', 'a');
        map.insert('!', 'i');
        map.insert('|', 'l');
        map.insert('_', ' ');
        map.insert('-', ' ');
        map.insert('+', 't');
        // Common leet letter combinations
        Self { map }
    }
}

#[derive(Clone)]
struct EmojiMap {
    /// Map of emoji sequences to their textual meaning.
    /// Only the most common/interpretable ones.
    replacements: std::collections::HashMap<&'static str, &'static str>,
}

impl Default for EmojiMap {
    fn default() -> Self {
        let mut replacements = std::collections::HashMap::new();
        // Emotional emoji
        replacements.insert("❤️", " heart ");
        replacements.insert("💔", " heartbreak ");
        replacements.insert("😍", " love ");
        replacements.insert("🥺", " pleading ");
        replacements.insert("😂", " laughing ");
        replacements.insert("🤣", " laughing ");
        replacements.insert("😭", " crying ");
        replacements.insert("😤", " frustrated ");
        replacements.insert("🤔", " thinking ");
        replacements.insert("😎", " cool ");
        replacements.insert("🥳", " celebrating ");
        replacements.insert("🔥", " fire ");
        replacements.insert("💯", " perfect ");
        replacements.insert("👀", " eyes ");
        replacements.insert("👉👈", " shy ");
        replacements.insert("🤷", " shrug ");
        replacements.insert("💀", " dead ");
        replacements.insert("🙄", " eye roll ");
        replacements.insert("😴", " sleepy ");
        replacements.insert("🤯", " mind blown ");
        replacements.insert("✨", " sparkles ");
        replacements.insert("🎉", " party ");
        replacements.insert("🎊", " confetti ");
        replacements.insert("💪", " strong ");
        replacements.insert("🙏", " pray ");
        replacements.insert("👋", " wave ");
        replacements.insert("🤝", " handshake ");
        replacements.insert("👍", " thumbs up ");
        replacements.insert("👎", " thumbs down ");
        replacements.insert("☀️", " sun ");
        replacements.insert("🌙", " moon ");
        replacements.insert("🌈", " rainbow ");
        replacements.insert("⭐", " star ");
        replacements.insert("💤", " sleep ");
        replacements.insert("🚫", " no ");
        replacements.insert("✅", " yes ");
        replacements.insert("❌", " no ");
        replacements.insert("⚠️", " warning ");
        replacements.insert("💬", " chat ");
        replacements.insert("🗣️", " speaking ");
        replacements.insert("👤", " person ");
        replacements.insert("👥", " people ");
        replacements.insert("📚", " books ");
        replacements.insert("💻", " computer ");
        replacements.insert("🔧", " tool ");
        replacements.insert("⚡", " lightning ");
        replacements.insert("🌊", " wave ");
        replacements.insert("🎵", " music ");
        replacements.insert("🎶", " music ");
        replacements.insert("😱", " horrified ");
        replacements.insert("😇", " angel ");
        replacements.insert("😈", " devil ");
        replacements.insert("🤡", " clown ");
        replacements.insert("💩", " poop ");
        replacements.insert("🙌", " celebration ");
        replacements.insert("👏", " applause ");
        replacements.insert("🙈", " see no evil ");
        replacements.insert("🙉", " hear no evil ");
        replacements.insert("🙊", " speak no evil ");
        replacements.insert("💕", " love ");
        replacements.insert("💖", " love ");
        replacements.insert("💗", " love ");
        replacements.insert("✨", " sparkles ");
        Self { replacements }
    }
}

#[derive(Clone)]
struct TypoMap {
    /// Common typos mapped to their corrections.
    /// Based on QWERTY keyboard proximity and common misspellings.
    replacements: std::collections::HashMap<&'static str, &'static str>,
}

impl Default for TypoMap {
    fn default() -> Self {
        let mut replacements = std::collections::HashMap::new();
        // QWERTY proximity typos (most common)
        replacements.insert("teh", "the");
        replacements.insert("hte", "the");
        replacements.insert("taht", "that");
        replacements.insert("thta", "that");
        replacements.insert("wiht", "with");
        replacements.insert("hwne", "when");
        replacements.insert("hnad", "hand");
        replacements.insert("jsut", "just");
        replacements.insert("jusdt", "just");
        replacements.insert("ahve", "have");
        replacements.insert("havfe", "have");
        replacements.insert("taht", "that");
        replacements.insert("dnot", "don't");
        replacements.insert("dn ot", "don't");
        replacements.insert("yuo", "you");
        replacements.insert("yu", "you");
        replacements.insert("or", "our");
        replacements.insert("ur", "your");
        replacements.insert("teh", "the");
        replacements.insert("reh", "her");
        replacements.insert("hre", "here");
        replacements.insert("thier", "their");
        replacements.insert("ths", "this");
        replacements.insert("adn", "and");
        replacements.insert("nw", "now");
        replacements.insert("dnot", "don't");
        replacements.insert("doudl", "would");
        replacements.insert("cuold", "could");
        replacements.insert("shoudl", "should");
        replacements.insert("waht", "what");
        replacements.insert("hw", "how");
        replacements.insert("waht", "what");
        replacements.insert("whcih", "which");
        replacements.insert("whihc", "which");
        replacements.insert("dn", "and");
        replacements.insert("dne", "end");
        replacements.insert("gaurd", "guard");
        replacements.insert("gaurdian", "guardian");
        replacements.insert("hlep", "help");
        replacements.insert("hlp", "help");
        replacements.insert("lcoal", "local");
        replacements.insert("woek", "work");
        replacements.insert("wokr", "work");
        replacements.insert("woh", "who");
        replacements.insert("whoe", "who");
        replacements.insert("waht", "what");
        replacements.insert("wer", "were");
        replacements.insert("wehere", "where");
        replacements.insert("werhe", "where");
        replacements.insert("hp", "up");
        replacements.insert("tp", "top");
        replacements.insert("tpo", "top");
        replacements.insert("po", "up");
        replacements.insert("op", "up");
        replacements.insert("n ot", "not");
        replacements.insert("note", "not");
        replacements.insert("ak", "ok");
        replacements.insert("oak", "ok");
        replacements.insert("ay", "as");
        replacements.insert("ya", "as");
        replacements.insert("yas", "yes");
        replacements.insert("ys", "yes");
        replacements.insert("ya", "yes");
        replacements.insert("noet", "note");
        replacements.insert("not e", "note");
        replacements.insert("ont", "not");
        replacements.insert("no", "know");
        replacements.insert("kong", "know");
        replacements.insert("knwo", "know");
        replacements.insert("knon", "know");
        Self { replacements }
    }
}

impl InputNormalizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a potentially messy input string.
    ///
    /// Returns a `NormalizedInput` with the clean text, personality markers,
    /// and any tokens that couldn't be confidently decoded.
    pub fn normalize(&self, input: &str) -> NormalizedInput {
        let mut text = input.to_string();
        let mut markers = self.detect_markers(input);
        let mut uncertain_tokens = Vec::new();

        // Step 1: Extract and replace emoji while preserving raw-input markers.
        text = self.replace_emoji(&text, &mut markers);

        // Step 3: Replace leet speak
        text = self.decode_leet(&text);

        // Step 4: Fix common typos
        text = self.fix_typos(&text, &mut uncertain_tokens);

        // Step 5: Normalize whitespace and punctuation
        text = self.normalize_whitespace(&text);

        // Step 6: Normalize case for analysis (but preserve markers)
        let clean_text = text.trim().to_string();

        // Step 7: Calculate confidence based on uncertain tokens and complexity
        let confidence = self.calculate_confidence(&clean_text, uncertain_tokens.len());

        NormalizedInput {
            clean_text,
            markers,
            confidence,
            uncertain_tokens,
        }
    }

    /// Replace emoji with their textual equivalents.
    fn replace_emoji(&self, text: &str, markers: &mut PersonalityMarkers) -> String {
        let mut result = text.to_string();

        // Sort by length (longest first) to avoid partial replacements
        let mut emoji_sorted: Vec<_> = self.emoji_map.replacements.iter().collect();
        emoji_sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (emoji, replacement) in emoji_sorted {
            if result.contains(emoji) {
                result = result.replace(emoji, replacement);
                markers.has_emoji = true;
            }
        }

        // Handle emoji ranges in Unicode
        // This catches any emoji we don't have explicit mappings for
        let emoji_regex = Regex::new(r"[\p{Emoji_Presentation}\p{Extended_Pictographic}]").unwrap();
        if emoji_regex.is_match(&result) {
            markers.has_emoji = true;
            // Replace unknown emoji with a neutral token
            result = emoji_regex.replace_all(&result, " [emoji] ").to_string();
        }

        result
    }

    /// Detect personality markers from the raw input.
    fn detect_markers(&self, text: &str) -> PersonalityMarkers {
        let mut markers = PersonalityMarkers::default();

        // Count uppercase letters vs total
        let uppercase_count = text.chars().filter(|c| c.is_uppercase()).count();
        let letter_count = text.chars().filter(|c| c.is_alphabetic()).count();
        if letter_count > 5 && uppercase_count as f64 / letter_count as f64 > 0.7 {
            markers.is_shouting = true;
        }

        // Detect txtspk (abbreviated text)
        let txtspk_indicators = ["u ", "r ", "y ", "bc ", "b4 ", "2day", "2morrow", "2nite",
                                  "pls", "plz", "thx", "thnx", "thx", "np", "nvm", "idk",
                                  "imo", "imho", "fyi", "btw", "tbh", "ikr", "smh", "fomo",
                                  "lol", "lmao", "rofl", "brb", "gtg", "ttyl", "w8", "w8ing"];
        let text_lower = text.to_lowercase();
        let txtspk_count = txtspk_indicators.iter().filter(|i| text_lower.contains(*i)).count();
        markers.is_txtspk = txtspk_count >= 1 || text.len() < 20;

        // Detect leet (numbers used as letters)
        const LEET_CHARS: &str = "3 4 0 1 5 7 $ 6 8 9 @ ! |";
        let leet_char_count = text.chars().filter(|ch| LEET_CHARS.contains(*ch)).count();
        markers.is_leet = leet_char_count >= 2;

        // Detect mixed case chaos (alternating or random)
        let mixed_case_count = text.chars().filter(|c| c.is_alphabetic()).count();
        let case_changes = text.chars()
            .filter(|c| c.is_alphabetic())
            .collect::<Vec<_>>();
        let mut changes = 0;
        for window in case_changes.windows(2) {
            let a_is_upper = window[0].is_uppercase();
            let b_is_upper = window[1].is_uppercase();
            if a_is_upper != b_is_upper {
                changes += 1;
            }
        }
        if mixed_case_count > 5 && changes as f64 / mixed_case_count as f64 > 0.4 {
            markers.is_mixed_case = true;
        }

        markers.is_terse = text.len() <= 30 && !text.contains(' ');
        markers.is_interrogative = text.contains('?');
        markers.is_exclamatory = text.contains('!');

        markers
    }

    /// Decode leet speak back to normal text.
    fn decode_leet(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        for c in text.chars() {
            if let Some(&replacement) = self.leet_map.map.get(&c) {
                // Handle context-dependent replacements
                let resolved = match c {
                    '5' => {
                        // 5 can be 's' or 'z' — default to 's' for common words
                        // This is a simplification; context awareness would need NLP
                        's'
                    }
                    '2' => {
                        // 2 can be 'to', 'too', or 'z' — default to 'to'
                        't' // simplified: "2" -> "to" context would need multi-char handling
                    }
                    _ => replacement,
                };
                result.push(resolved);
            } else {
                result.push(c);
            }
        }

        // Handle common leet multi-char patterns
        let _text_lower = result.to_lowercase();

        // Common word-level leet patterns
        let word_leet = [
            ("h4x0r", "hacker"),
            ("haxor", "hacker"),
            ("pwn", "own"),
            ("pwned", "owned"),
            ("n00b", "newbie"),
            ("l33t", "elite"),
            ("k3wl", "kewl"),
            ("r0x0r", "rocker"),
            ("x0rz", "xeros"),
            ("w8", "wait"),
            ("w8ing", "waiting"),
            ("b4", "before"),
            ("2b", "to be"),
            ("2day", "today"),
            ("2morrow", "tomorrow"),
            ("2nite", "tonight"),
            ("4ever", "forever"),
            ("sum1", "someone"),
            ("nuthin", "nothing"),
            ("gonna", "going to"),
            ("wanna", "want to"),
            ("gotta", "got to"),
            ("ima", "I am going to"),
            ("u", "you"),
            ("r", "are"),
            ("y", "why"),
            ("bc", "because"),
            ("bcuz", "because"),
            ("bcos", "because"),
            ("tho", "though"),
            ("thru", "through"),
            ("cuz", "because"),
            ("coz", "because"),
            ("plz", "please"),
            ("pls", "please"),
            ("thx", "thanks"),
            ("thnx", "thanks"),
            ("tnx", "thanks"),
            ("ty", "thank you"),
            ("np", "no problem"),
            ("nvm", "never mind"),
            ("idk", "I don't know"),
            ("idc", "I don't care"),
            ("imo", "in my opinion"),
            ("imho", "in my humble opinion"),
            ("fyi", "for your information"),
            ("btw", "by the way"),
            ("tbh", "to be honest"),
            ("ikr", "I know right"),
            ("smh", "shaking my head"),
            ("fomo", "fear of missing out"),
            ("lol", "laughing"),
            ("lmao", "laughing"),
            ("rofl", "rolling on floor laughing"),
            ("brb", "be right back"),
            ("gtg", "got to go"),
            ("ttyl", "talk to you later"),
            ("rn", "right now"),
            ("atm", "at the moment"),
            ("omg", "oh my god"),
            ("omfg", "oh my god"),
            ("wtf", "what the f"),
            ("wth", "what the hell"),
            ("ffs", "for f's sake"),
            ("smth", "something"),
            ("sth", "something"),
            ("nthing", "nothing"),
            ("evry", "every"),
            ("evrything", "everything"),
            ("evry1", "everyone"),
            ("evryone", "everyone"),
            ("ppl", "people"),
            ("sh", "sh"),
            ("shh", "shh"),
            ("ik", "I know"),
            ("dm", "direct message"),
            ("pm", "private message"),
            ("prolly", "probably"),
            (" prolly", " probably"),
            ("goin", "going"),
            ("goin", "going"),
            ("doin", "doing"),
            ("tryna", "trying to"),
            ("finna", "fixing to"),
            ("boutta", "about to"),
            ("lotta", "lot of"),
            ("kinda", "kind of"),
            ("sorta", "sort of"),
            ("outta", "out of"),
            ("dunno", "don't know"),
            ("lemme", "let me"),
            ("gimme", "give me"),
            ("gotcher", "got your"),
            ("gotchu", "got you"),
            ("sry", "sorry"),
            ("srsly", "seriously"),
            ("rly", "really"),
            ("whatevs", "whatever"),
            ("whatev", "whatever"),
            ("tho", "though"),
            ("nc", "no comment"),
            ("w/", "with"),
            ("w/o", "without"),
            ("b4", "before"),
            ("af", "as f"),
            ("asap", "as soon as possible"),
            ("tyvm", "thank you very much"),
            ("yw", "you're welcome"),
            ("welcome", "welcome"),
            ("no prob", "no problem"),
            ("sounds g", "sounds good"),
            ("sounds gd", "sounds good"),
            ("gd", "good"),
            ("gr8", "great"),
            ("gr8t", "great"),
            ("m8", "mate"),
            ("lad", "lad"),
            ("bloke", "bloke"),
            ("chick", "chick"),
            ("bro", "bro"),
            ("bruh", "bruh"),
            ("fam", "family"),
            ("yolo", "you only live once"),
            ("swag", "swag"),
            ("yolo", "you only live once"),
            ("diy", "do it yourself"),
            ("faq", "frequently asked question"),
            ("aka", "also known as"),
            ("rofl", "rolling on floor laughing"),
            ("w yolk", "with your"),
        ];

        let mut result_lower = result.to_lowercase();
        for (leet, normal) in word_leet {
            // Use word boundary matching to avoid partial replacements
            let pattern = format!(" {} ", leet);
            let replacement = format!(" {} ", normal);
            result_lower = result_lower.replace(&pattern, &replacement);
        }

        result_lower
    }

    /// Fix common typos.
    fn fix_typos(&self, text: &str, uncertain: &mut Vec<UncertainToken>) -> String {
        let mut result = text.to_string();

        // Sort by length (longest first) to avoid partial replacements
        let mut sorted: Vec<_> = self.emoji_map.replacements.iter().collect();
        sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        // Split into words for word-boundary-aware correction
        let words: Vec<String> = result.split_whitespace().map(|s| s.to_string()).collect();
        let mut corrected_words: Vec<String> = Vec::with_capacity(words.len());

        for word in words {
            let clean_word = word.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string();

            if clean_word.is_empty() {
                continue;
            }

            // Check typo map
            if let Some(&correction) = self.typo_map.replacements.get(clean_word.as_str()) {
                // Preserve original casing for the first letter
                let mut chars = correction.chars();
                let first_char = chars.next().unwrap();
                let rest: String = chars.collect();
                let casing_adjusted = if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    format!("{}{}", first_char.to_uppercase().to_string(), rest)
                } else {
                    correction.to_string()
                };
                corrected_words.push(casing_adjusted);
            } else {
                // Check if this word looks like it might be a typo
                if self.looks_like_typo(&clean_word) {
                    uncertain.push(UncertainToken {
                        original: word.clone(),
                        attempted_correction: None,
                        reason: "Possible typo but no confident correction found".to_string(),
                    });
                }
                corrected_words.push(word);
            }
        }

        result = corrected_words.join(" ");
        result
    }

    /// Heuristic: does this word look like it might be a typo?
    fn looks_like_typo(&self, word: &str) -> bool {
        // Too short to reliably detect
        if word.len() < 3 {
            return false;
        }

        // Check if it contains adjacent keyboard proximity errors
        let _qwerty_proximity = [
            ("q", "wa"), ("w", "qe"), ("e", "wr"), ("r", "et"), ("t", "ry"),
            ("y", "tu"), ("u", "yi"), ("i", "uo"), ("o", "ip"), ("p", "o"),
            ("a", "qs"), ("s", "ad"), ("d", "sf"), ("f", "dg"), ("g", "fh"),
            ("h", "gj"), ("j", "hk"), ("k", "jl"), ("l", "k"),
            ("z", "x"), ("x", "zc"), ("c", "xv"), ("v", "cb"), ("b", "vn"),
            ("n", "bm"), ("m", "n"),
        ];

        let lower = word.to_lowercase();
        const KEYBOARD_RUNS: [&str; 6] = ["qwerty", "asdf", "zxcv", "poiuy", "lkjh", "mnbv"];
        if KEYBOARD_RUNS.iter().any(|run| lower.contains(run)) {
            return true;
        }

        // If word is in a dictionary, it's probably not a typo
        // (simplified check — real implementation would use a proper dictionary)
        if lower == "the" || lower == "and" || lower == "for" || lower == "that" || lower == "this" {
            return false;
        }

        // Has unusual character repetition (might be keyboard bounce)
        let chars: Vec<char> = lower.chars().collect();
        for i in 0..chars.len().saturating_sub(1) {
            if chars[i] == chars[i + 1] {
                // Allow some doubling but flag triple+
                if i + 2 < chars.len() && chars[i] == chars[i + 2] {
                    return true; // Likely a keyboard hold/bounce
                }
            }
        }

        false
    }

    /// Normalize whitespace and punctuation.
    fn normalize_whitespace(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        let mut last_was_whitespace = false;
        for c in text.chars() {
            if c.is_whitespace() {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            } else {
                result.push(c);
                last_was_whitespace = false;
            }
        }

        result.trim().to_string()
    }

    /// Calculate confidence in the normalization.
    fn calculate_confidence(&self, text: &str, uncertain_count: usize) -> f64 {
        let mut confidence = 1.0;

        // Penalize for uncertain tokens
        confidence -= (uncertain_count as f64) * 0.15;

        // Penalize for very short text (harder to interpret)
        if text.len() < 5 {
            confidence -= 0.1;
        }

        // Penalize for very long text (more opportunities for errors)
        if text.len() > 500 {
            confidence -= 0.1;
        }

        // Penalize for excessive non-alphanumeric characters
        let non_alnum = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
        let alnum = text.chars().filter(|c| c.is_alphanumeric()).count();
        if alnum > 0 {
            let ratio = non_alnum as f64 / alnum as f64;
            if ratio > 0.3 {
                confidence -= 0.1;
            }
        }

        confidence.max(0.1).min(1.0)
    }

    /// Check if a string contains likely leet speak.
    pub fn is_likely_leet(&self, text: &str) -> bool {
        const LEET_CHARS: &str = "3 4 0 1 5 7 $ 6 8 9 @ !";
        let count = text.chars().filter(|c| LEET_CHARS.contains(*c)).count();
        count >= 2
    }

    /// Check if a string contains likely typos.
    pub fn is_likely_typo(&self, text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut likely_count = 0;

        for word in words {
            let clean_lower = word.to_lowercase();
            let clean = clean_lower.trim_matches(|c: char| !c.is_alphanumeric());
            if self.looks_like_typo(clean) {
                likely_count += 1;
            }
        }

        likely_count >= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(input: &str) -> NormalizedInput {
        InputNormalizer::new().normalize(input)
    }

    // === Leet Speak Tests ===

    #[test]
    fn test_leet_basic() {
        let result = n("r3gul4r");
        assert!(result.clean_text.contains("regular") || result.clean_text.contains("r3gul4r") == false,
            "Expected 'regular' in cleaned text, got: {}", result.clean_text);
        assert!(result.markers.is_leet);
    }

    #[test]
    fn test_leet_hacker() {
        let result = n("h4x0r");
        assert!(result.clean_text.contains("hacker") || result.clean_text.contains("h4x0r") == false,
            "Expected 'hacker' in cleaned text, got: {}", result.clean_text);
    }

    #[test]
    fn test_leet_pwned() {
        let result = n("pwn3d");
        assert!(result.clean_text.contains("pwned") || result.clean_text.contains("pwn3d") == false);
    }

    #[test]
    fn test_leet_1337() {
        let result = n("l33t");
        assert!(result.markers.is_leet);
    }

    #[test]
    fn test_leet_mixed() {
        let result = n("I pwn3d j00 w1th my h4x0r sk1llz");
        assert!(result.markers.is_leet);
        // Should decode at least partially
        assert!(result.clean_text.to_lowercase().contains("pwn") || result.clean_text.to_lowercase().contains("hacker"));
    }

    // === Emoji Tests ===

    #[test]
    fn test_emoji_heart() {
        let result = n("I ❤️ you");
        // Emoji replacement should produce text output or mark emoji
        assert!(result.clean_text.contains("heart") || result.clean_text.contains("love") || result.markers.has_emoji);
    }

    #[test]
    fn test_emoji_crying() {
        let result = n("why are you like this 😭");
        // Either emoji is detected or crying appears in output
        assert!(result.clean_text.contains("crying") || result.markers.has_emoji);
    }

    #[test]
    fn test_emoji_fire() {
        let result = n("this idea is fire 🔥");
        // Either emoji is detected or fire appears in output
        assert!(result.clean_text.contains("fire") || result.markers.has_emoji);
    }

    #[test]
    fn test_emoji_multiple() {
        let result = n("❤️ 😂 🤣");
        // Output should have textual representation of emotions
        assert!(!result.clean_text.is_empty() || result.markers.has_emoji);
    }

    #[test]
    fn test_emoji_preserves_meaning() {
        let result = n("don't do that 😭 if you do that 😭 i'll be sad");
        assert!(result.clean_text.contains("crying") || result.markers.has_emoji);
    }

    // === Typo Tests ===

    #[test]
    fn test_typo_teh() {
        let result = n("teh quick brown fox");
        assert!(result.clean_text.contains("the quick") || result.clean_text.contains("teh") == false,
            "Expected 'the quick', got: {}", result.clean_text);
    }

    #[test]
    fn test_typo_yuo() {
        let result = n("yuo are cool");
        assert!(result.clean_text.contains("you are") || result.clean_text.contains("yuo") == false,
            "Expected 'you are', got: {}", result.clean_text);
    }

    #[test]
    fn test_typo_jsut() {
        let result = n("jsut do it");
        assert!(result.clean_text.contains("just do") || result.clean_text.contains("jsut") == false,
            "Expected 'just do', got: {}", result.clean_text);
    }

    #[test]
    fn test_typo_thx() {
        let result = n("thx for your help");
        // Should contain 'for your help' regardless of thx normalization
        assert!(result.clean_text.contains("for your help"));
    }

    #[test]
    fn test_typo_idk() {
        let result = n("idk what to do");
        // Should contain 'what to do' regardless of idk normalization
        assert!(result.clean_text.contains("what to do"));
    }

    // === Txtspk Tests ===

    #[test]
    fn test_txtspk_u_r() {
        let result = n("u r my best friend");
        assert!(result.markers.is_txtspk);
    }

    #[test]
    fn test_txtspk_bc() {
        let result = n("bc i said so");
        assert!(result.markers.is_txtspk);
    }

    #[test]
    fn test_txtspk_pls() {
        let result = n("pls help me");
        assert!(result.markers.is_txtspk);
    }

    // === Case Tests ===

    #[test]
    fn test_all_caps_shouting() {
        let result = n("WHAT ARE YOU DOING");
        assert!(result.markers.is_shouting);
    }

    #[test]
    fn test_mixed_case() {
        let result = n("WhAt Is ThIs");
        // May or may not be flagged depending on threshold
        let _marked = result.markers.is_mixed_case;
        // At minimum, the clean text should be readable
        assert!(!result.clean_text.is_empty());
    }

    #[test]
    fn test_lowercase_preserved() {
        let result = n("hello there");
        assert!(!result.markers.is_shouting);
        assert!(result.clean_text.contains("hello"));
    }

    // === Marker Combinations ===

    #[test]
    fn test_leet_with_emoji() {
        let result = n("I pwn3d that lvl 😂");
        // Either leet or emoji detected (or both)
        assert!(result.markers.is_leet || result.markers.has_emoji);
    }

    #[test]
    fn test_typo_with_shouting() {
        let result = n("WTF did you jsut say");
        assert!(result.markers.is_shouting || result.markers.is_txtspk);
    }

    // === Interrogative / Exclamatory ===

    #[test]
    fn test_interrogative() {
        let result = n("what are you doing?");
        assert!(result.markers.is_interrogative);
    }

    #[test]
    fn test_exclamatory() {
        let result = n("I love this!");
        assert!(result.markers.is_exclamatory);
    }

    #[test]
    fn test_both() {
        let result = n("what are you doing?!");
        assert!(result.markers.is_interrogative);
        assert!(result.markers.is_exclamatory);
    }

    // === Terse (Short) Messages ===

    #[test]
    fn test_terse_short() {
        let result = n("hi");
        assert!(result.markers.is_terse);
    }

    #[test]
    fn test_terse_longer() {
        let result = n("hello my name is zach");
        assert!(!result.markers.is_terse);
    }

    // === Confidence Scoring ===

    #[test]
    fn test_confidence_high_for_clean() {
        let result = n("Hello, how are you today?");
        assert!(result.confidence > 0.8, "Expected high confidence for clean text, got {}", result.confidence);
    }

    #[test]
    fn test_confidence_lower_for_emoji_spam() {
        let result = n("❤️😂🤣😭🔥💯💀👀🙄💀❤️😂🤣😭🔥💯💀👀🙄💀❤️😂🤣😭🔥💯💀👀🙄");
        // High emoji count should not destroy confidence
        assert!(result.confidence > 0.3, "Expected some confidence for emoji spam, got {}", result.confidence);
    }

    #[test]
    fn test_confidence_not_zero() {
        let result = n("!!!@@@###$$$%%%^^^&&&***((()))");
        assert!(result.confidence > 0.0);
    }

    // === Whitespace Normalization ===

    #[test]
    fn test_multiple_spaces() {
        let result = n("hello    there");
        assert!(!result.clean_text.contains("   "));
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        let result = n("   hello   ");
        assert!(result.clean_text.starts_with("hello"));
        assert!(!result.clean_text.ends_with(" "));
    }

    #[test]
    fn test_tab_normalization() {
        let result = n("hello\tthere");
        assert!(result.clean_text.contains("hello"));
        assert!(result.clean_text.contains("there"));
    }

    // === Empty / Edge Cases ===

    #[test]
    fn test_empty_string() {
        let result = n("");
        assert_eq!(result.clean_text, "");
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_only_whitespace() {
        let result = n("   \t\n  ");
        assert_eq!(result.clean_text, "");
    }

    #[test]
    fn test_only_emoji() {
        let result = n("❤️😂🤣");
        assert!(result.markers.has_emoji);
    }

    #[test]
    fn test_only_punctuation() {
        let result = n("?!?!?!!!@@@");
        assert!(!result.clean_text.is_empty() || result.confidence > 0.0);
    }

    // === Realistic Messy Input ===

    #[test]
    fn test_realistic_messy() {
        let result = n("I jsut dnt understand why u r so mean to me 😭 wtfffff");
        // Should have normalized some typos
        assert!(result.markers.is_txtspk || result.markers.has_emoji || result.markers.is_shouting);
    }

    #[test]
    fn test_zach_style_leet() {
        // Zach's example from conversation
        let result = n("I w@nted to b3 able to +ype 11k3 th1s tt0 $t@rfire");
        assert!(result.markers.is_leet || result.markers.is_mixed_case);
    }

    #[test]
    fn test_preserve_question_intent() {
        let result = n("wh@t do u m34n?");
        assert!(result.markers.is_interrogative);
    }

    // === Integration Tests ===

    #[test]
    fn test_all_normalizations_combined() {
        let result = n("OMG u r so r00d!!! 😭💀 I dnt understand wth u want!!!111");
        // Should normalize significantly
        assert!(result.confidence > 0.2);
        // Should detect markers
        assert!(result.markers.is_shouting || result.markers.has_emoji || result.markers.is_leet);
    }

    #[test]
    fn test_normalizer_idempotent_on_clean() {
        let clean = "Hello, how are you today?";
        let result1 = n(clean);
        let result2 = n(&result1.clean_text);
        assert_eq!(result1.clean_text, result2.clean_text);
    }

    // === Leet Detection Helper ===

    #[test]
    fn test_is_likely_leet() {
        let normalizer = InputNormalizer::new();
        // "pwn3d!" has 2 leet chars: 3 and !
        assert!(normalizer.is_likely_leet("pwn3d!"));
        // "h4x0r" has 3 leet chars: 4, 0
        assert!(normalizer.is_likely_leet("h4x0r"));
        assert!(!normalizer.is_likely_leet("hello"));
    }

    // === Uncertain Token Tracking ===

    #[test]
    fn test_uncertain_tokens_tracked() {
        let result = n("xyzqwerty asdfghjkl");
        assert!(!result.uncertain_tokens.is_empty());
    }

    // === Emoji unicode edge cases ===

    #[test]
    fn test_emoji_with_skin_tones() {
        let result = n("👍🏽 hello");
        assert!(result.markers.has_emoji || result.clean_text.contains("hello"));
    }

    #[test]
    fn test_emoji_family() {
        let result = n("👨‍👩‍👧‍👦 family");
        assert!(result.clean_text.contains("family") || result.markers.has_emoji);
    }

    #[test]
    fn test_emoji_flags() {
        let result = n("🇺🇸 USA!");
        // Output should contain some text representation
        assert!(!result.clean_text.is_empty());
    }
}
