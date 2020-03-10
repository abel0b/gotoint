pub fn pass(url: &String) -> bool {
    // TODO: regex
    // TODO: filter file extensions
    // TODO: protocol check
    !url.contains("#")
}
