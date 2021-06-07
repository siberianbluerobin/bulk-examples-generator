use config::ConfigError;
use serde_derive::{Deserialize, Serialize};

/// Struct for define the config of the generator
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneratorConfig {
    /// Max terminals generated, when this limit is reached the generation is stopped
    pub terminals_limit: Option<usize>,

    // Limite usado para determinar la cantidad de reglas que son expandidas "abiertas" presenta un error +/- 10% debido a las reglas que contienen negación
    // None para indicar que no existe limite (ese es el valor por defecto)
    /// Max rules opened in generation, when this limit is reached the generation of subsequent rules
    /// return the parameter text_expand_limit.
    ///
    /// _default value:_ None (No limit)
    pub rule_expand_limit: Option<usize>,

    //  Limite suave para detener el procesamiento de elementos y acelerar la convergencia
    //  Se encarga de reemplazar los rangos
    //  [0, n] o [0, infinito) por [0, 1]
    //  [1, n] o [1, infinito) por [1, 2]
    /// To process a rule the elements are placed in a stack. If the grammar is very deep or recursive the number of
    ///  elements in the stack will be large, if the length of the stack exceeds the value of this parameter then
    ///  the delimiters "\*", "+" will be converted into ranges [0,1] and [1,2] respectively in order to reduce
    ///  the number of items to process.
    ///
    /// _default value:_ 10.000
    pub soft_limit: usize,

    // Limite agresivo para detener la expansión interna de elementos y obligar la convergencia
    // La cantidad es el número máximo de nodos que se procesaran desde el inicio del algoritmo
    /// In the process of generating an example, each processed expression increases the expression counter, if the
    /// parameter value is reached, all the unprocessed expressions from now on will not produce any results, the
    /// identifiers will only return the parameter text_expand_limit.
    ///
    /// _default value:_ 25.000
    pub hard_limit: usize,

    /// Limite máximo de recursividad, usado para evitar la excepción StackOverflow
    /// All of the generation process of an example happens in a stack (There isn't recursion involved) except for
    /// a little expression `!b ~ a`.
    /// If you have a recursive grammar with a lot of negations, the parameter limit_depth_level return
    /// the parameter text_expand_limit.
    ///
    /// _default value:_ 200
    pub limit_depth_level: usize,

    /// factor de convergencia alternativo (no usado actualmente)
    // pub c_factor: u8,

    // Texto a mostrar si a la hora de expandir un identificador se ha alcanzado el limite de expansión
    /// It's the text returned by rules when the hard_limit or limit_depth_level is reached
    ///
    /// _default value:_ ""
    pub text_expand_limit: String,

    /// Dummy config, para futuros valores
    _dummy: bool,

    /// Limit the elements generated in a ("example")* case
    /// e.g 1, will be a probability of 0.5 of not generating nothing (0/1)
    /// e.g 2, will be a probability of 0.33 of not generating nothing (0/1/2)
    /// e.g 3, will be a probability of 0.25 of not generating nothing (0/1/2/3)
    /// _default value:_ 5
    pub upper_bound_zero_or_more_repetition: u32,

    /// Limit the elements generated in a ("example")+ case
    /// e.g 2, will generate maximum 2 "example" strings
    /// e.g 3, will generate maximum 3 "example" strings
    /// _default value:_ 5
    pub upper_bound_one_or_more_repetition: u32,

    /// Upper limit present in "at least" expression e{n,}
    /// e.g 15, will generate between {n, n+15} "example" strings
    /// e.g 20, will generate between {n, n+20} "example" strings
    /// _default value:_ 10
    pub upper_bound_at_least_repetition: u32,

    /// When generator finds an expression !A ~ B
    /// It has to generate B and then probe that is not A
    /// here you can limit the times that B is generated and compared with A
    /// for more details please refer to README
    /// _default value:_ 100
    pub max_attempts_negation: u32,
}

impl GeneratorConfig {
    /// Create a config with the provided TOML file
    ///
    /// `GeneratorConfig::new("config.toml")`
    ///
    /// If you want to get default config
    ///
    /// `let default: GeneratorConfig = Default::default();`
    ///
    pub fn new(config_file: &str) -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name("src/config/default.toml"))
            .unwrap()
            .merge(config::File::with_name(config_file))
            .unwrap()
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .merge(config::Environment::with_prefix("APP"))
            .unwrap();

        settings.try_into()
    }
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            terminals_limit: None,
            rule_expand_limit: None,
            soft_limit: 10000,
            hard_limit: 25000,
            // Valor calculado a mano teniendo en cuenta que la profundidad a la que explota es 400
            limit_depth_level: 200,
            // c_factor: 1,
            text_expand_limit: "".to_string(),
            _dummy: false,
            upper_bound_zero_or_more_repetition: 5,
            upper_bound_one_or_more_repetition: 5,
            upper_bound_at_least_repetition: 10,
            max_attempts_negation: 100,
        }
    }
}
