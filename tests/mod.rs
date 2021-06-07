/// Función de apoyo para crear tests
/// Recibe grammar $grammar_string:expr, $initial_rule:expr, $examples_generated:expr
#[macro_export]
macro_rules! boilerplate_test_grammar {
    ($grammar_string:expr, $initial_rule:expr, $examples_generated:expr) => {
        let gen_config: GeneratorConfig = Default::default();
        let mut exe_config: ExecutorConfig = Default::default();
        exe_config.print_stdout = false;

        let grammar_string = $grammar_string.to_string();

        // Se compila la gramatica para comprobar la validez de la gramatica
        // y porque es el mecanismo usado para parsear de vuelta los ejemplos
        let g = compile_grammar(grammar_string.clone()).unwrap();
        //println!("{:?}", g);

        let results = generate_examples(
            grammar_string,
            $examples_generated,
            $initial_rule.to_string(),
            &gen_config,
            &exe_config,
        );

        // println!("{:?}", results);

        for result in results {
            let parsing = parse_input(
                g.clone(),
                $initial_rule.to_string(),
                result.clone().unwrap(),
            );
            // println!("{}", result.unwrap());
            assert_eq!(Ok(()), parsing);
        }
    };
}

#[cfg(test)]
mod test {

    // use crate::compile_grammar;
    use bulk_examples_generator::config::*;
    use bulk_examples_generator::parse_input;
    use bulk_examples_generator::*;

    /// La mecánica de los tests es la siguiente
    /// Si genero n elementos a partir de una misma gramatica
    /// estos mismos n elementos se deben poder parsear usando la misma gramatica
    #[test]
    fn char_range() {
        boilerplate_test_grammar!(
            r#"
            alpha = { 'a'..'z' | 'A'..'Z' }
            digit = { '0'..'9' }
            ident = { (alpha | digit)+ }
            "#,
            "ident",
            50
        );
    }

    #[test]
    fn char_range_builtin_rules() {
        boilerplate_test_grammar!(
            r#"
            alpha = { ASCII_ALPHA }
            digit = { ASCII_DIGIT }
            ident = { (alpha | digit)+ }"#,
            "ident",
            50
        );
    }

    // TODO: Hacer test para probar la función init_grammar que tiene grammar y grammar_clean
}

#[cfg(test)]
mod pest_rules {
    use bulk_examples_generator::config::*;
    use bulk_examples_generator::parse_input;
    use bulk_examples_generator::{compile_grammar, generate_examples};

    /// expr{n} exactly n repetitions
    #[test]
    fn exactly_n_repetitions() {
        boilerplate_test_grammar!(
            r#"
                list_content = { li{1} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{5} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{7} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{400} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );
    }

    /// expr{m, n} between m and n repetitions, inclusive
    #[test]
    fn between_m_and_n_repetitions() {
        boilerplate_test_grammar!(
            r#"
                list_content = { li{0, 50} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{123, 125} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{9, 10} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );
    }

    /// expr{, n}  at most n repetitions
    #[test]
    fn at_most_n_repetitions() {
        boilerplate_test_grammar!(
            r#"
                list_content = { li{, 1} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{, 15} }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );
    }

    #[test]
    fn at_least_n_repetitions() {
        boilerplate_test_grammar!(
            r#"
                list_content = { li{60, } }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );

        boilerplate_test_grammar!(
            r#"
                list_content = { li{2, } }
                li = { "<li>Hola</li>\n" }
            "#,
            "list_content",
            10
        );
    }

    #[test]
    fn negation_char() {
        //TODO este test debería ser
        // Line = { " "{5} ~ (!(" " | "0") ~ alphabet_numbers) ~ ASCII_ALPHA{,15} }
        // sin embargo actualmente solo se procesan idents en la negación "!"
        boilerplate_test_grammar!(
            r#"
                alphabet_numbers = { "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" 
                    | "i" | "j" | "k" | "l" | "m" | "n" | "ñ" | "o" | "p" | "q" 
                    | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
                    | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8"
                    | "9" | " "}
                forbidden_chars = {" " | "0"}
                Line = { " "{5} ~ (!forbidden_chars ~ alphabet_numbers) ~ ASCII_ALPHA{,15} }
            "#,
            "Line",
            500
        );
    }

    #[test]
    fn negation_ident() {
        boilerplate_test_grammar!(
            r#"
                Etiqueta = { Parrafo | Enlace | Texto }
                Parrafo = { "<p>" ~ Texto ~ "</p>" ~ (Parrafo | "") }
                Enlace = { "<a>" ~ (Texto | Parrafo) ~ "</a>" }
                Texto = { ASCII_ALPHA{,15} }
                EtiquetaSinEnlace = { !Enlace ~ Etiqueta }
                "#,
            "EtiquetaSinEnlace",
            500
        );

        boilerplate_test_grammar!(
            r#"
                alphabet = { "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" |
                "m" | "n" | "ñ" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" }
                vocal = { "a" | "e" | "i" | "o" | "u" }
                consonant = { !vocal ~ alphabet }
            "#,
            "consonant",
            500
        );
    }
}

// #[cfg(test)]
// mod tricky_tests {
//     use bulk_examples_generator::config::*;
//     use bulk_examples_generator::parse_input;
//     use bulk_examples_generator::{compile_grammar, parallel_generate_examples};

//     #[test]
//     fn ordered_choice() {
//         boilerplate_test_grammar!(
//             r#"
//                 tricky = {("a" | "ab") ~ "c"}
//             "#,
//             "tricky",
//             100
//         );
//     }
// }
