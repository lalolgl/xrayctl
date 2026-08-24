use owo_colors::OwoColorize;

pub fn title(text: &str) {
    println!("{}", text.bold());
    println!("{}", "────────────────────────────────".dimmed());
}

pub fn separator() {
    println!("{}", "────────────────────────────────".dimmed());
}

pub fn info(text: &str) {
    println!("{} {}", "●".blue(), text);
}

pub fn success(text: &str) {
    println!("{} {}", "✓".green(), text);
}

pub fn error(text: &str) {
    println!("{} {}", "✗".red(), text);
}

pub fn warning(text: &str) {
    println!("{} {}", "!".yellow(), text);
}

pub fn field(name: &str, value: impl std::fmt::Display) {
    println!("  {:<10} {}", name.dimmed(), value);
}
