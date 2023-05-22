pub fn print_hilighted(
    base_string: &String,
    start: usize,
    end: usize,
    indentation: String,
) -> String {
    let context_margin = 30;

    let precontext =
        &base_string[(start as isize - context_margin as isize).max(0) as usize..start];
    let error = &base_string[start..end];
    let postcontext = &base_string[end..(end + context_margin).min(base_string.len())];

    let lined_precontext = &precontext[precontext.find("\n").unwrap_or(0)..precontext.len()];
    let lined_postcontext = &postcontext[0..postcontext.rfind("\n").unwrap_or(postcontext.len())];

    // let padded_precontext = lined_precontext.replace("\n", &format!("\n{indentation} 00 : "));
    // let padded_error = error.replace("\n", &format!("\n{indentation} 00 : "));
    // let padded_postcontext = lined_postcontext.replace("\n", &format!("\n{indentation} 00 : "));

    let preprecontext = &base_string[0..(start as isize - context_margin as isize).max(0) as usize];

    let mut running_line_numer: usize = preprecontext.chars().filter(|e| e == &'\n').count();

    let padded_precontext: String = lined_precontext
        .chars()
        .map(|e| {
            if e == '\n' {
                running_line_numer += 1;
                format!("\n{indentation} {running_line_numer} : ")
            } else {
                e.to_string()
            }
        })
        .collect();
    let padded_error: String = error
        .chars()
        .map(|e| {
            if e == '\n' {
                running_line_numer += 1;

                format!("\n{indentation} {running_line_numer} : ")
            } else {
                e.to_string()
            }
        })
        .collect();
    let padded_postcontext: String = lined_postcontext
        .chars()
        .map(|e| {
            if e == '\n' {
                running_line_numer += 1;
                format!("\n{indentation} {running_line_numer} : ")
            } else {
                e.to_string()
            }
        })
        .collect();

    format!(
        "{}\x1b[1m\x1b[37;41m{}\x1b[0m{}",
        padded_precontext, padded_error, padded_postcontext
    )
}

// fn vector_find_replace<T: 'static>(v: &Vec<T>, find: &T, replace: &T) -> Vec<T>
// where
//     T: PartialEq<T>,
//     T: Clone,
// {
//     v.iter()
//         .map(|original_value| {
//             if original_value.clone() == find.clone() {
//                 replace.clone()
//             } else {
//                 original_value.clone()
//             }
//         })
//         .collect::<Vec<T>>()
// }
