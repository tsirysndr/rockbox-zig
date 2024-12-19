pub fn format_milliseconds(ms: u64) -> String {
    // Convert milliseconds to seconds and remaining milliseconds
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;

    // Format as mm:ss
    format!("{:02}:{:02}", minutes, remaining_seconds)
}
