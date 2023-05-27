
use std::cell::RefCell;

thread_local!(static ENTER_EXIT_LOGGER_LEVEL : RefCell<u16> = RefCell::new(1));

struct EnterExitLogger{
    msg: String
}

impl Drop for EnterExitLogger {
    fn drop(&mut self) {
        self.exit()
    }
}
impl EnterExitLogger{

    fn new(msg: String) -> EnterExitLogger{
        let ret = EnterExitLogger{
            msg
        };
        ret.enter();

        ret
    }

    fn current_level() -> u16 {
        ENTER_EXIT_LOGGER_LEVEL.with(|level| {
            level.clone().into_inner()
        })
    }

    pub fn current_indent()  -> String {
        let level = Self::current_level();
        let mut ret: String = "".to_string();
        for _i in 0..level {
            ret = ret + "    ";
        }
        ret.to_string()
    }

    fn enter(&self){
        println!("{} --> {}", Self::current_indent(), self.msg );
        ENTER_EXIT_LOGGER_LEVEL.with(|level| {
            *level.borrow_mut() = Self::current_level() + 1;
        });
    }

    fn exit(&self){
        ENTER_EXIT_LOGGER_LEVEL.with(|level| {
            *level.borrow_mut() = Self::current_level() - 1;
        });
        println!("{} <-- {}", Self::current_indent(), self.msg );
    }
}

#[macro_export]
macro_rules! EnterExitLogger {
    ($msg:expr, $($args:expr),+ ) => {
        let _un_nombre_de_variable_que_nunca_nadie_va_a_usar_jamas_nunca_no = EnterExitLogger::new(format!( $msg, $($args),+ ));
    };
    ($msg:expr  ) => {
        let _un_nombre_de_variable_que_nunca_nadie_va_a_usar_jamas_nunca_no = EnterExitLogger::new($msg.to_string());
    };

}

#[cfg(test)]
mod tests {
    use crate::utils::EnterExitLogger;

    fn hanoi(discs: u16, pole_from: u16, pole_to: u16 ){
        EnterExitLogger!( "{} discos del polo {} al polo {}", discs, pole_from, pole_to );
        fn compute_aux_pole( pole_from: u16, pole_to: u16 ) -> Result<u16,String> {
            EnterExitLogger!( "calculo palo que me queda si uso {} y {}", pole_from, pole_to );
            match (pole_from, pole_to) {
                (1, 3) => { Ok(2) }
                (3, 1) => { Ok(2) }
                (1, 2) => { Ok(3) }
                (2, 1) => { Ok(3) }
                (2, 3) => { Ok(1) }
                (3, 2) => { Ok(1) }
                (_,_) => Err(format!("aux_pole error: {} {}", pole_from, pole_to) )
            }
        }
        if discs == 1 {
            println!( "{} Se mueve el disco del polo {} al polo {}", EnterExitLogger::current_indent(), pole_from, pole_to );
        }
        else {
            let pole_aux = compute_aux_pole(pole_from, pole_to).expect("Malos polos");
            hanoi(discs - 1, pole_from, pole_aux);
            hanoi(1, pole_from, pole_to);
            hanoi(discs - 1, pole_aux, pole_to);
        }
    }

    #[test]
    fn test_hanoi(){
        hanoi(3, 1, 3);
    }

    #[test]
    fn test_nested_for(){
        EnterExitLogger!("Tablas del 1 al 10");
        for i in 1..10 {
            EnterExitLogger!("Tabla del {}", i );
            for j in 1..10{
                EnterExitLogger!("{} x {} = {}", i, j, i*j );
            }
        }
    }
}


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
