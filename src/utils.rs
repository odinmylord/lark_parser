pub fn output_cleaner(mut output: String) -> String {
    output = match output.strip_prefix(" ") {
        Some(stripped) => stripped.to_string(),
        None => output,
    };
    output = output.replace("e->z:", "e<->z:");
    output = "z->e:".to_string() + &output;
    output
}