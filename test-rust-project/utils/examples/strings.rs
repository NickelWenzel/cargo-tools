use utils::strings;

fn main() {
    println!("String Utils Example");

    let test_strings = vec!["hello world", "rust programming", "cargo tools extension"];

    for s in test_strings {
        println!("Original: {}", s);
        println!("Capitalized: {}", strings::capitalize(s));
        println!("Reversed: {}", strings::reverse(s));
        println!("Word count: {}", strings::word_count(s));
        println!("---");
    }

    // Email validation examples
    let emails = vec!["valid@example.com", "invalid-email", "another@test.org"];

    println!("Email validation:");
    for email in emails {
        let is_valid = strings::validate_email(email).unwrap_or(false);
        println!("{}: {}", email, if is_valid { "Valid" } else { "Invalid" });
    }
}
