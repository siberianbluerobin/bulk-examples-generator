use crate::compile_grammar;
use crate::config::GeneratorConfig;
use crate::parse_input;

use pest_meta::ast::{Expr, Rule as AstRule};
use rand::prelude::*;
use regex::Regex;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct Grammar {
    pub rules: HashMap<String, AstRule>,
}

/// Estructura interna usada para almacenar los datos que se van a procesar por el algoritmo
/// gramática original, y gramática preprocesada para evitar conflictos con pest
#[derive(Clone, Debug)]
pub struct InputData {
    /// Gramática original ingresada por el usuario
    grammar: Grammar,
    /// Gramática limpia (sin blacklist), para otras operaciones con pest
    clean_grammar: Grammar,
}

impl InputData {
    pub fn new(grammar: String) -> Self {
        let (grammar, clean_grammar) = init_grammar(grammar);
        InputData {
            grammar: grammar.unwrap(),
            clean_grammar: clean_grammar.unwrap(),
        }
    }
}

/// Estructura de contexto, para guardar datos del estado actual de cada elemento a procesar
///  Some((weights, choices_count, previous_rule, actual_rule, actual_expr))
///  Vec<(Vec<f32>, HashMap<String, u32>, Option<AstRule>, AstRule, Expr)> = Vec::new();
#[derive(Clone, Debug, Default)]
struct Context {
    total_count: usize,
    /// Contador de profundidad actual como si fuera Breadth-first search
    depth_count: usize,
    /// Contador de hermanos como si fuera Breadth-first search
    breadth_count: usize,
    /// Pesos acumulados
    /// no esta siendo usado actualmente
    weights: Vec<f32>,
    /// Parametro diseñado para mantener un conteo de las reglas que han aparecido hasta el momento
    /// no esta siendo usado actualmente
    choices_count: HashMap<String, u32>,
}

fn init_grammar(
    grammar_string: String,
) -> (
    Result<Grammar, Vec<HashMap<String, String>>>,
    Result<Grammar, Vec<HashMap<String, String>>>,
) {
    // Compilar gramática normal
    let grammar = compile_grammar(grammar_string.clone());

    // Compilar gramática limpia (Sin blacklist)
    let re = Regex::new(r#""\|BLACKLIST\|[I\|R]\|.+\|""#).unwrap();
    let clean_grammar_string = re.replace_all(&grammar_string, "\"\"");
    let clean_grammar = compile_grammar(clean_grammar_string.to_string());

    (grammar, clean_grammar)
}

pub fn generate_example(
    input_data: InputData,
    start_rule: String,
    config: &GeneratorConfig,
) -> Result<String, String> {
    let mut rng = thread_rng();

    // let config = GeneratorConfig {
    //     expand_limit: expand_limit,
    //     soft_limit: 10000,
    //     hard_limit: 25000,
    //     ..Default::default()
    // };

    let result = traverse(input_data, &start_rule, &mut rng, config);
    result
}

pub fn traverse(
    input_data: InputData,
    start_rule: &String,
    rng: &mut ThreadRng,
    config: &GeneratorConfig,
) -> Result<String, String> {
    // Factor de convergencia
    // let c_factor: f32 = 0.01;
    // let c_factor: f32 = 0.5;

    // Regla Inicial
    let rule;
    // Extrae la regla inicial
    let find_lhs = input_data.grammar.rules.get(start_rule);
    match find_lhs {
        Some(r) => rule = r.clone(),
        None => return Ok(start_rule.to_string()),
    }

    // Lista negra dinámica, usada para evitar la expansión de ciertos elementos de forma dinámica
    // Por ejemplo para evitar <a><p><a>TEXT</a></p></a>
    let mut dynamic_blacklist: Vec<String> = Vec::new();

    // Stack usado para almacenar todos los términos sintácticos
    // weights, definitions_count, actual_definition, actual_term
    // Contexto, definicion anterior, definición actual, termino actual a procesar
    let mut processing_stack: Vec<(Context, Option<Rc<AstRule>>, Rc<AstRule>, Rc<Expr>)> =
        Vec::new();

    // Add first term
    let context: Context = Default::default();
    let rc_rule = Rc::new(rule.clone());
    let rc_expr = Rc::new(rule.expr);
    processing_stack.push((context, None, rc_rule, rc_expr));

    // Variable que contiene la cadena generada
    let result = processing_terms(
        &input_data,
        rng,
        config,
        0,
        processing_stack,
        &mut dynamic_blacklist,
    );

    result
}

// depth level es una variable auxiliar para apoyar y detener la profundidad de la recursión
fn processing_terms(
    input_data: &InputData,
    rng: &mut ThreadRng,
    config: &GeneratorConfig,
    depth_level: usize,
    processing_stack: Vec<(Context, Option<Rc<AstRule>>, Rc<AstRule>, Rc<Expr>)>,
    dynamic_blacklist: &mut Vec<String>,
) -> Result<String, String> {
    // Call to processing_stack
    let result = processing_stack_fn(
        input_data,
        rng,
        config,
        depth_level,
        processing_stack,
        dynamic_blacklist,
    );

    Ok(result.unwrap().0)
}

/// Retorna (result, count_output, count_nodes_processed, count_expand_idents)
/// el resultado en String, los nodos procesados por la función, y los identificadores expandidos (reglas)
fn processing_stack_fn(
    input_data: &InputData,
    mut rng: &mut ThreadRng,
    config: &GeneratorConfig,
    depth_level: usize,
    mut processing_stack: Vec<(Context, Option<Rc<AstRule>>, Rc<AstRule>, Rc<Expr>)>,
    dynamic_blacklist: &mut Vec<String>,
) -> Result<(String, usize, usize, usize), String> {
    // Variable que contiene la cadena generada
    let mut result = String::new();

    // println!("Profundidad: {}", depth_level);

    // 32KB
    let stack_red_zone: usize = 32 * 1024;
    // println!("Stack restante: {}", stacker::remaining_stack().unwrap());
    // println!("Hard Limit: {}", hard_limit);
    if depth_level > config.limit_depth_level
        || config.hard_limit < 1
        || stacker::remaining_stack().unwrap() < stack_red_zone
    {
        // Si el stack esta lleno, forzar un string vacio
        // En realidad debería retorna un error de generación
        // return Err(Error::RecursionLimit(format!(
        //     "Limit for recursion reached processing <{:?}>!",
        //     processing_stack
        // )));

        // return Ok((result, 0, 0));
        return Ok((config.text_expand_limit.to_owned(), 0, 0, 0));
    }

    // Counter of the number of 'strings' or output generated
    let mut count_output = 0;

    // Contador de la cantidad de expresiones que han sido procesadas
    let mut count_nodes_processed = 0;

    // Contador de la cantidad de identificadores expandidos (rules)
    let mut count_expand_idents = 0;

    // Secuencia superior de repetición
    // 0 - No repeticiones
    // 1 - 0/1 50/50
    // 2 - 0/1/2 33/33/33
    // etc
    // let mut upper_bound_repeated_sequence = 45;
    let mut upper_bound_repeated_sequence = config.upper_bound_zero_or_more_repetition;
    let mut upper_bound_repeated_one_sequence = config.upper_bound_one_or_more_repetition;

    // Limite agresivo para detener la expansión interna de elementos y forzar la convergencia
    let mut bool_hard_limit = false;

    // Variable de control usada para decidir si se debe seguir procesando una expr choice recursivamente
    let mut continue_processing_choice = false;
    // Variable usada para decidir si el procesamiento de opciones aleatorias es detenido por no existir más opciones disponibles
    // Valores posibles
    // -1 -> No se están procesando valores
    // 0 -> Aún se continúan procesando valores
    // 1 -> Ya se ha procesado uno de los dos últimos valores
    // 2 -> Ya se está procesando el último valor
    let mut last_processing_choice = -1;
    let mut selected_choice = Rc::new(Expr::Str("RESERVED".to_string()));
    let mut choice_count = 0;

    while let Some((context, previous_rule, actual_rule, actual_expr)) = processing_stack.pop() {
        // println!("TERM: {:?}", actual_expr);
        // result.push_str(" ' ");

        match &*actual_expr {
            // match actual_expr {
            // Matches an exact string, e.g. `"a"`
            Expr::Str(string) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        Rc::clone(&actual_expr),
                        Rc::clone(&actual_rule),
                        previous_rule.clone(),
                        &mut processing_stack,
                    )
                } else {
                    if string.starts_with("|BLACKLIST|I|") {
                        add_blacklist_items(dynamic_blacklist, string, &input_data.clean_grammar);
                    } else if string.starts_with("|BLACKLIST|R|") {
                        remove_blacklist_items(
                            dynamic_blacklist,
                            string,
                            &input_data.clean_grammar,
                        );
                    } else {
                        count_output += 1;
                        result.push_str(&string);
                    }
                }
            }
            // Matches an exact string, case insensitively (ASCII only), e.g. `^"a"`
            Expr::Insens(string) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else {
                    // FIXME: esto tal vez debería generar algo como hOla

                    // Random bool for transform to lowercase or uppercase
                    let tmp = rng.gen_bool(0.5);
                    count_output += 1;
                    if tmp {
                        result.push_str(&string.to_lowercase());
                    } else {
                        result.push_str(&string.to_uppercase());
                    }
                }
            }
            // Matches one character in the range, e.g. `'a'..'z'`
            Expr::Range(initial_char, end_char) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else {
                    // let mut rng = rand::thread_rng();
                    let from = initial_char.chars().next().unwrap();
                    let to = end_char.chars().next().unwrap();
                    let random_char = rng.gen_range(from..=to);

                    count_output += 1;
                    result.push(random_char);
                }
            }
            // Matches the rule with the given name, e.g. `a`
            Expr::Ident(name) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else {
                    if config.rule_expand_limit.is_none()
                        || config.rule_expand_limit.unwrap() > count_expand_idents
                    {
                        if !dynamic_blacklist.contains(name) {
                            match input_data.grammar.rules.get(name) {
                                Some(new_rule) => {
                                    let mut new_context = context.clone();
                                    new_context.depth_count += 1;
                                    new_context.breadth_count = 0;
                                    processing_stack.push((
                                        new_context,
                                        Some(actual_rule.clone()),
                                        Rc::new(new_rule.clone()),
                                        Rc::new(new_rule.expr.clone()),
                                    ));
                                    count_expand_idents += 1;
                                }
                                None => {
                                    unimplemented!("The rule hasn't been found: {}", &name);
                                }
                            }
                        } else {
                            // Si `name` está en la blacklist, se coloca la regla actual nuevamente, para que otra opción sea elegida
                            // FIXME: Advertencia: si una regla contiene como única opción un identificador presente en la blacklist
                            // podría ingresar en un bucle infinito, por ejemplo:  IdentExample = OtherIdent; si OtherIdent esta en blacklist se producirá un bucle infinito
                            // println!("Blacklist actual: {:?}", dynamic_blacklist);
                            // println!("Blacklist - {:?} - Se reingresara la regla: {:?}", &name, actual_rule.clone());

                            // Verifica si hay un posible loop comparando la blacklist con los elementos de la regla a abrir
                            if verify_infinite_loop_blacklist(
                                &input_data.clean_grammar,
                                &actual_rule.clone().name,
                                dynamic_blacklist,
                            ) {
                                let mut new_context = context.clone();
                                new_context.depth_count += 1;
                                processing_stack.push((
                                    new_context,
                                    previous_rule.clone(),
                                    actual_rule.clone(),
                                    Rc::new(actual_rule.expr.clone()),
                                ));
                            } else {
                                match previous_rule {
                                    Some(ref previous) => {
                                        if verify_infinite_loop_blacklist(
                                            &input_data.clean_grammar,
                                            &previous.name.clone(),
                                            dynamic_blacklist,
                                        ) {
                                            let mut new_context = context.clone();
                                            new_context.depth_count += 1;
                                            processing_stack.push((
                                                new_context,
                                                None,
                                                previous_rule.unwrap().clone(),
                                                Rc::new(actual_rule.expr.clone()),
                                            ));
                                        }
                                    }
                                    None => {
                                        // println!("Loop detected in grammar");
                                        // return Err("Existe un ciclo en la gramática, se ha detenido la ejecución".to_string());
                                        return Err("Loop detected in grammar, the execution has been stopped".to_string());
                                    }
                                }
                            }
                        }
                    // }
                    } else {
                        // FIXME No se expande el identificador pero se adiciona un texto temporalmente, para que no salgan tantos tags vacios
                        result.push_str(&config.text_expand_limit);
                    }
                }
            }
            //     /// Matches a custom part of the stack, e.g. `PEEK[..]`
            //     Expr::PeekSlice(i32, Option<i32>),
            //     /// Positive lookahead; matches expression without making progress, e.g. `&e`
            //     Expr::PosPred(Box<Expr>),
            //     /// Negative lookahead; matches if expression doesn't match, without making progress, e.g. `!e`
            //     Expr::NegPred(Box<Expr>),
            // Matches a sequence of two expressions, e.g. `e1 ~ e2`
            Expr::Seq(lhs, rhs) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else {
                    match &**lhs {
                        // Si es una negación seguida de algo más el procesamiento implica un parseo
                        Expr::NegPred(neg_expr) => {
                            let mut new_processing_stack: Vec<(
                                Context,
                                Option<Rc<AstRule>>,
                                Rc<AstRule>,
                                Rc<Expr>,
                            )> = Vec::new();

                            // Add first term
                            let mut new_context = context.clone();
                            new_context.breadth_count += 1;
                            new_processing_stack.push((
                                new_context,
                                previous_rule.clone(),
                                actual_rule.clone(),
                                Rc::new(*rhs.clone()),
                            ));

                            let mut invalid_neg_generation = false;
                            let mut count_remaining_attempts = config.max_attempts_negation;
                            loop {
                                // Se usa un valor más pequeño de soft limit y hard limit para reducir posibilidad de OVERFLOW STACK
                                let mut new_config = config.clone();
                                if let Some(exp_lim) = config.rule_expand_limit {
                                    new_config.rule_expand_limit =
                                        Some(exp_lim.saturating_sub(count_expand_idents));
                                }
                                new_config.soft_limit = 20;
                                new_config.hard_limit =
                                    config.hard_limit.saturating_sub(count_nodes_processed);

                                match processing_stack_fn(
                                    &input_data,
                                    rng,
                                    &new_config,
                                    depth_level + 1,
                                    new_processing_stack.clone(),
                                    dynamic_blacklist,
                                ) {
                                    Ok((
                                        result_neg,
                                        neg_count_output,
                                        neg_count_nodes_processed,
                                        neg_count_expand_idents,
                                    )) => {
                                        // println!("Resultado negación: {}", &result_neg);
                                        // Se hace un recorrido en toda la expresión de negación (B) unicamente revisando los Ident
                                        let _item = neg_expr.clone().map_bottom_up(|inner_expr| {
                                            match inner_expr.clone() {
                                                Expr::Ident(name) => {
                                                    let parsing = parse_input(
                                                        input_data.clean_grammar.clone(),
                                                        name.clone(),
                                                        result_neg.clone(),
                                                    );
                                                    match parsing {
                                                        // Si el parseo fue exitoso quiere decir que la secuencia generada es invalidad
                                                        Ok(_) => {
                                                            invalid_neg_generation = true;
                                                        }
                                                        Err(_a) => {
                                                            //     println!(
                                                            //     "Error de parseo en negación: {:?}, regla {}",
                                                            //     _a,
                                                            //     name
                                                            // ),
                                                            // Si el parseo no es exitoso se cumple la premisa A - B
                                                            invalid_neg_generation = false
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            };
                                            inner_expr
                                        });

                                        // Si la secuencia generada es valida, es decir no hace parte de la negación se debe adicionar al string
                                        if !invalid_neg_generation {
                                            // Sumar los nodos que han sido procesados en la negación al conteo general
                                            count_output += neg_count_output;
                                            count_nodes_processed += neg_count_nodes_processed;
                                            count_expand_idents += neg_count_expand_idents;
                                            result.push_str(&result_neg);
                                        }
                                    }
                                    error => {
                                        return error;
                                    }
                                }

                                if !invalid_neg_generation {
                                    break;
                                }

                                if count_remaining_attempts <= 0 {
                                    println!(
                                        "Exceso de intentos para A - B en: !{:?} ~ {:?}",
                                        neg_expr, *rhs
                                    );
                                    break;
                                }

                                count_remaining_attempts -= 1;
                                // println!("{}", count_remaining_attempts);
                            }
                        }
                        // Si no es Una negación, la secuencia se procesa normalmente
                        _ => {
                            let mut new_context = context.clone();
                            new_context.breadth_count += 1;
                            // El orden importa, puesto que es un stack
                            processing_stack.push((
                                new_context.clone(),
                                previous_rule.clone(),
                                actual_rule.clone(),
                                Rc::new(*rhs.clone()),
                            ));

                            new_context.breadth_count += 1;
                            processing_stack.push((
                                new_context,
                                previous_rule,
                                actual_rule,
                                Rc::new(*lhs.clone()),
                            ));
                        }
                    }
                }
            }
            // Matches either of two expressions, e.g. `e1 | e2`
            Expr::Choice(lhs, rhs) => {
                // TODO debería hacer match también en rhs
                // println!("Izquierda: {:?}", lhs.clone());
                // println!("Derecha: {:?}", rhs.clone());
                match &**lhs {
                    Expr::Choice(_lhs_inner, _rhs_inner) => {
                        continue_processing_choice = true;
                        // selected_choice = *rhs.clone();
                        processing_stack.push((
                            context.clone(),
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*lhs.clone()),
                        ));
                        processing_stack.push((
                            context.clone(),
                            previous_rule,
                            actual_rule,
                            Rc::new(*rhs.clone()),
                        ));
                    }
                    _ => {
                        if continue_processing_choice {
                            // Hora de procesar las últimas dos opciones
                            last_processing_choice = 0;
                            processing_stack.push((
                                context.clone(),
                                previous_rule.clone(),
                                actual_rule.clone(),
                                Rc::new(*rhs.clone()),
                            ));
                            processing_stack.push((
                                context.clone(),
                                previous_rule,
                                actual_rule,
                                Rc::new(*lhs.clone()),
                            ));
                        } else {
                            // Reinicio de las variables de apoyo para selección
                            selected_choice = Rc::new(Expr::Str("RESERVED".to_string()));
                            continue_processing_choice = false;
                            choice_count = 0;

                            let selected = random_definition(
                                &vec![(**lhs).clone(), (**rhs).clone()],
                                &mut rng,
                            )
                            .unwrap();
                            // println!("SELECCTED: {:?}", &selected);
                            // processing_stack((Vec::new(), HashMap::new(), rule, rule.expr))
                            let mut new_context = context.clone();
                            new_context.breadth_count += 1;
                            processing_stack.push((
                                new_context,
                                previous_rule,
                                actual_rule,
                                Rc::new(selected),
                            ));
                        }
                    }
                }

                // processing_stack.push((Vec::new(), HashMap::new(), actual_rule, selected));
            }
            // Optionally matches an expression, e.g. `e?`
            Expr::Opt(expr) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else if !bool_hard_limit {
                    let option = rng.gen_bool(0.5);
                    if option {
                        let mut new_context = context.clone();
                        new_context.breadth_count += 1;
                        processing_stack.push((
                            new_context,
                            previous_rule,
                            actual_rule,
                            Rc::new(*expr.clone()),
                        ));
                    }
                }
            }
            // Matches an expression zero or more times, e.g. `e*`
            Expr::Rep(expr) => {
                // HARD LIMIT
                if processing_stack.len() > config.soft_limit {
                    upper_bound_repeated_sequence = 1;
                    upper_bound_repeated_one_sequence = 2;
                } else if !bool_hard_limit {
                    let num_reps = rng.gen_range(0..upper_bound_repeated_sequence);
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            }
            // Matches an expression one or more times, e.g. `e+`
            Expr::RepOnce(expr) => {
                // HARD LIMIT
                if processing_stack.len() > config.soft_limit {
                    upper_bound_repeated_sequence = 1;
                    upper_bound_repeated_one_sequence = 2;
                } else if !bool_hard_limit {
                    let num_reps = rng.gen_range(1..upper_bound_repeated_one_sequence);
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            }
            // Matches an expression an exact number of times, e.g. `e{n}`
            Expr::RepExact(expr, num_reps) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else if !bool_hard_limit {
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            }
            // Matches an expression at least a number of times, e.g. `e{n,}`
            Expr::RepMin(expr, min_reps) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else if !bool_hard_limit {
                    let max_reps = min_reps + config.upper_bound_at_least_repetition;
                    let num_reps = rng.gen_range(*min_reps..=max_reps);
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            }
            // Matches an expression at most a number of times, e.g. `e{,n}`
            Expr::RepMax(expr, max_reps) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else if !bool_hard_limit {
                    let num_reps = rng.gen_range(0..=*max_reps);
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            }
            // Matches an expression a number of times within a range, e.g. `e{m, n}`
            Expr::RepMinMax(expr, min_reps, max_reps) => {
                if continue_processing_choice {
                    auxiliar_function(
                        rng,
                        &mut choice_count,
                        &mut selected_choice,
                        &mut continue_processing_choice,
                        &mut last_processing_choice,
                        context,
                        actual_expr,
                        actual_rule,
                        previous_rule,
                        &mut processing_stack,
                    )
                } else if !bool_hard_limit {
                    let num_reps = rng.gen_range(*min_reps..=*max_reps);
                    (1..num_reps + 1).for_each(|rep| {
                        let mut new_context = context.clone();
                        new_context.breadth_count += rep as usize;
                        processing_stack.push((
                            new_context,
                            previous_rule.clone(),
                            actual_rule.clone(),
                            Rc::new(*expr.clone()),
                        ))
                    });
                }
            } //     /// Continues to match expressions until one of the strings in the `Vec` is found
            //     Expr::Skip(Vec<String>),
            //     /// Matches an expression and pushes it to the stack, e.g. `push(e)`
            // Expr::Push(Box<Expr>),
            Expr::PeekSlice(_, _) => {}
            Expr::PosPred(_) => {}
            Expr::NegPred(_) => {}
            Expr::Skip(_) => {}
            Expr::Push(_) => {}
        }
        // println!("{:?}", result);
        // println!("Len: {}", processing_stack.len());
        // println!("Nodes processed: {}", count_nodes_processed);
        count_nodes_processed += 1;

        if count_nodes_processed > config.hard_limit {
            // Activar HARD LIMIT
            bool_hard_limit = true;
            // println!("HARD LIMIT REACHED: {}", config.hard_limit);
            break;
        }

        if let Some(config_count) = config.terminals_limit {
            if count_output >= config_count {
                // println!("TERMINALS LIMIT REACHED: {}", config_count);
                break;
            }
        }
    }

    // println!("Nodes processed: {}", count_nodes_processed);
    Ok((
        result,
        count_output,
        count_nodes_processed,
        count_expand_idents,
    ))
}

/// Random entre Simbolos |
pub fn random_definition(definitions: &Vec<Expr>, rng: &mut ThreadRng) -> Result<Expr, String> {
    // println!("Selección aleatoria: {:?}", &definitions);
    match definitions.choose(rng) {
        Some(selected) => {
            // println!("Gano: {:?}", &selected);
            Ok(selected.to_owned())
        }
        None => Err("crate::error::GenerationError::RandomChoose".to_string()),
    }
}

fn auxiliar_function(
    rng: &mut ThreadRng,
    choice_count: &mut i32,
    selected_choice: &mut Rc<Expr>,
    continue_processing_choice: &mut bool,
    last_processing_choice: &mut i8,
    context: Context,
    actual_expr: Rc<Expr>,
    actual_rule: Rc<AstRule>,
    previous_rule: Option<Rc<AstRule>>,
    processing_stack: &mut Vec<(Context, Option<Rc<AstRule>>, Rc<AstRule>, Rc<Expr>)>,
) {
    if *choice_count == 0 {
        *selected_choice = Rc::clone(&actual_expr);
        *choice_count += 1;
    } else {
        // println!(
        //     "Actual: {:?} vs Selected: {:?}",
        //     actual_expr, selected_choice
        // );
        // println!("Range selection: [{}, {})", 0, *choice_count + 1);
        let num = rng.gen_range(0..=*choice_count);
        if num == *choice_count {
            *selected_choice = Rc::clone(&actual_expr);
        }
        // println!("Choice count: {} - Num: {}", choice_count, num);
        // println!("WIN: {:?}", selected_choice);
        // println!("Last processing choice: {:?}", last_processing_choice);

        if *last_processing_choice == 1 {
            processing_stack.push((
                context.clone(),
                previous_rule.clone(),
                actual_rule.clone(),
                Rc::clone(&selected_choice),
            ));

            // Reinicio de variables
            *last_processing_choice = -1;
            *continue_processing_choice = false;
            *selected_choice = Rc::new(Expr::Str("RESERVED".to_string()));
            // -1 Para evitar que RESERVED entre en comparación
            // choice_count al final de esta sección terminaria valiendo cero
            *choice_count = -1;
        }

        // Si se estan procesando las últimas dos opciones posibles
        if *last_processing_choice == 0 {
            *last_processing_choice += 1;
        }
        *choice_count += 1;
    }
}

/// Dada una regla, verifica si unicamente se componen de choice que sean idents y retorna los idents
/// si no retorna None
/// TODO: elaborar test para esta función
fn rule_is_only_ident_choices(grammar: &Grammar, blacklisted_ident: &str) -> Option<Vec<String>> {
    if let Some(rule) = grammar.rules.get(blacklisted_ident) {
        let mut idents = Vec::<String>::new();
        let mut aux_expr = Some(rule.expr.clone());
        while let Some(expr) = aux_expr.clone() {
            match expr {
                // Matches either of two expressions, e.g. `e1 | e2`
                Expr::Choice(lhs, rhs) => {
                    // Matching del rhs para valdiar que sea un ident
                    match *rhs {
                        Expr::Ident(rhs_name) => {
                            idents.push(rhs_name);
                        }
                        _ => {
                            // Si es otra cosa diferente de un ident, cancelar
                            return None;
                        }
                    }

                    match *lhs {
                        Expr::Ident(rhs_name) => {
                            idents.push(rhs_name);
                            // aux_expr = None;
                            break;
                        }
                        Expr::Choice(_, _) => aux_expr = Some(*lhs),
                        _ => {
                            // Si es otra cosa diferente de un ident, cancelar
                            return None;
                        }
                    }
                }
                Expr::Ident(name) => {
                    idents.push(name);
                    // aux_expr = None;
                    break;
                }
                _ => {
                    return None;
                }
            }
        }
        return Some(idents);
    } else {
        return None;
    }
}

/**
 * Retorna true si encuentra un ciclo infinito
 */
fn verify_infinite_loop_blacklist(grammar: &Grammar, ident: &str, blacklist: &Vec<String>) -> bool {
    match rule_is_only_ident_choices(grammar, ident) {
        Some(idents) => {
            for ident_item in idents {
                if !blacklist.contains(&ident_item) {
                    return true;
                }
            }
            false
        }
        None => false,
    }
}

fn add_blacklist_items(blacklist: &mut Vec<String>, string: &String, grammar: &Grammar) {
    // Adicionar un ident a la blacklist
    let blacklisted_idents = string
        .trim_start_matches("|BLACKLIST|I|")
        .trim_end_matches("|")
        .split(",");

    for ident in blacklisted_idents {
        if let Some(mut rules) = rule_is_only_ident_choices(grammar, &ident) {
            // Si la regla se compone unicamente de choices donde cada choice es un ident, adicionar todas las choices
            // println!("{:?}", rules);
            blacklist.append(&mut rules);
        } else {
            // sino solo adicionar la regla
            blacklist.push(ident.to_string());
        }
    }
}

fn remove_blacklist_items(blacklist: &mut Vec<String>, string: &String, grammar: &Grammar) {
    // Remover un ident de la blacklist
    let blacklisted_idents = string
        .trim_start_matches("|BLACKLIST|R|")
        .trim_end_matches("|")
        .split(",");

    for ident in blacklisted_idents {
        if let Some(rules) = rule_is_only_ident_choices(grammar, &ident) {
            // Si la regla se compone unicamente de choices donde cada choice es un ident, remover todas las choices
            // dynamic_blacklist.retain(|r| !rules.contains(r));

            // Solo se remueve la ultima ocurrencia encontrada para evitar problemas de remover un item de la blacklist en
            // profundidad por ejemplo |BLACKLIST|I|ThingWithA|  |BLACKLIST|I|A| / |BLACKLIST|R|A| |BLACKLIST|R|ThingWithA|
            for rule in rules {
                // println!("{:?}", rule);
                // println!("Antes: {:?}", dynamic_blacklist);
                let index = blacklist.iter().rev().position(|x| *x == rule).unwrap();
                // println!("Index: {:?}", index);
                // println!("Nuevo Index: {:?}", blacklist.len() - 1 - index);

                // Puesto que la position obtenida es la invertida, debido a que el iterador recorre al reves
                // es necesario hacer el siguiente calculo para poder obtener la posición correcta
                // index + len_lista % len_lista => posición
                blacklist.remove(blacklist.len() - 1 - index);
                // println!("Despues: {:?}", blacklist);
            }
        } else {
            // sino solo remover la regla
            let index = blacklist.iter().rev().position(|x| *x == ident).unwrap();
            blacklist.remove(blacklist.len() - 1 - index);
        }
    }
}

#[test]
fn test_only_ident_in_rules() {
    let g = compile_grammar(
        r#"
        Uno = {"a"}
        Dos = {"b"}
        Tres = {"c"}
        Cuatro = {"d"}
        Example = { Uno | Dos | Tres | Cuatro }
    "#
        .to_string(),
    )
    .unwrap();

    let mut results = rule_is_only_ident_choices(&g, &"Example").unwrap();
    assert_eq!(vec!["Uno", "Dos", "Tres", "Cuatro"].sort(), results.sort());
}

#[test]
fn test_only_ident_in_rules_1() {
    let g = compile_grammar(
        r#"
        Uno = {"a"}
        Dos = {"b"}
        Tres = {"c"}
        Example = { Uno | Dos | Tres | "Cuatro" }
    "#
        .to_string(),
    )
    .unwrap();

    let results = rule_is_only_ident_choices(&g, &"Example");
    assert_eq!(None, results);
}

#[test]
fn test_only_ident_in_rules_2() {
    let g = compile_grammar(
        r#"
        Uno = {"a"}
        Dos = {"b"}
        Tres = {"c"}
        Example = { Uno | Dos | Tres ~ Uno }
    "#
        .to_string(),
    )
    .unwrap();

    let results = rule_is_only_ident_choices(&g, &"Example");
    assert_eq!(None, results);
}
