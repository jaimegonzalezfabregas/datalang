pub fn print_hilighted(base_string: &String, start: usize, end: usize, indentation: String) {
    let context_margin = 20;

    let precontext =
        &base_string[(start as isize - context_margin as isize).max(0) as usize..start];
    let error = &base_string[start..end];
    let postcontext = &base_string[end..(end + context_margin).min(base_string.len())];

    precontext.replace("\n", &format!("\n{indentation}|"));
    error.replace("\n", &format!("\n{indentation}|"));
    postcontext.replace("\n", &format!("\n{indentation}|"));

    print!(
        "\n\n{indentation}{}\x1b[1m\x1b[37;41m{}\x1b[0m\x1b[21m{}\n\n",
        precontext, error, postcontext
    );
}
