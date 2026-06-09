

#[cfg(test)]
mod tests {
    use std::collections::HashMap;


    #[derive(Debug, Clone, PartialEq)]
    pub enum ParamValue {
        Float(f64),
        Int(i64),
        Bool(bool),
        String(String),
        Color([f32; 4]),
        Vector2([f64; 2]),
        Vector3([f64; 3]),
        Enum(String, Vec<String>),
    }

    impl ParamValue {
        pub fn type_name(&self) -> &'static str {
            match self {
                ParamValue::Float(_) => "float",
                ParamValue::Int(_) => "int",
                ParamValue::Bool(_) => "bool",
                ParamValue::String(_) => "string",
                ParamValue::Color(_) => "color",
                ParamValue::Vector2(_) => "vector2",
                ParamValue::Vector3(_) => "vector3",
                ParamValue::Enum(_, _) => "enum",
            }
        }
    }


    #[derive(Debug, Clone)]
    pub struct ParamDef {
        pub name: String,
        pub default: ParamValue,
        pub min: Option<f64>,
        pub max: Option<f64>,
        pub step: Option<f64>,
        pub required: bool,
    }

    impl ParamDef {
        pub fn new(name: &str, default: ParamValue) -> Self {
            Self {
                name: name.to_string(),
                default,
                min: None,
                max: None,
                step: None,
                required: false,
            }
        }

        pub fn with_range(mut self, min: f64, max: f64) -> Self {
            self.min = Some(min);
            self.max = Some(max);
            self
        }

        pub fn with_step(mut self, step: f64) -> Self {
            self.step = Some(step);
            self
        }

        pub fn required(mut self) -> Self {
            self.required = true;
            self
        }

        pub fn validate(&self, value: &ParamValue) -> Result<(), String> {

            if self.default.type_name() != value.type_name() {
                return Err(format!(
                    "Type mismatch: expected {}, got {}",
                    self.default.type_name(),
                    value.type_name()
                ));
            }


            match value {
                ParamValue::Float(v) => {
                    if let Some(min) = self.min {
                        if *v < min {
                            return Err(format!("Value {} is below minimum {}", v, min));
                        }
                    }
                    if let Some(max) = self.max {
                        if *v > max {
                            return Err(format!("Value {} is above maximum {}", v, max));
                        }
                    }
                }
                ParamValue::Int(v) => {
                    if let Some(min) = self.min {
                        if (*v as f64) < min {
                            return Err(format!("Value {} is below minimum {}", v, min));
                        }
                    }
                    if let Some(max) = self.max {
                        if (*v as f64) > max {
                            return Err(format!("Value {} is above maximum {}", v, max));
                        }
                    }
                }
                ParamValue::Enum(selected, options) => {
                    if !options.contains(selected) {
                        return Err(format!(
                            "Invalid enum value '{}', expected one of: {:?}",
                            selected, options
                        ));
                    }
                }
                _ => {}
            }

            Ok(())
        }
    }


    pub struct ParameterizedNode {
        pub id: String,
        pub name: String,
        pub param_defs: HashMap<String, ParamDef>,
        pub param_values: HashMap<String, ParamValue>,
        pub bindings: HashMap<String, (String, String)>,
    }

    impl ParameterizedNode {
        pub fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                param_defs: HashMap::new(),
                param_values: HashMap::new(),
                bindings: HashMap::new(),
            }
        }

        pub fn add_param(&mut self, def: ParamDef) {
            let name = def.name.clone();
            let default = def.default.clone();
            self.param_defs.insert(name.clone(), def);
            self.param_values.insert(name, default);
        }

        pub fn set_param(&mut self, name: &str, value: ParamValue) -> Result<(), String> {
            let def = self.param_defs.get(name)
                .ok_or_else(|| format!("Parameter '{}' not found", name))?;

            def.validate(&value)?;
            self.param_values.insert(name.to_string(), value);
            Ok(())
        }

        pub fn get_param(&self, name: &str) -> Option<&ParamValue> {
            self.param_values.get(name)
        }

        pub fn bind_param(&mut self, param_name: &str, source_node: &str, source_param: &str) -> Result<(), String> {
            if !self.param_defs.contains_key(param_name) {
                return Err(format!("Parameter '{}' not found", param_name));
            }
            self.bindings.insert(
                param_name.to_string(),
                (source_node.to_string(), source_param.to_string()),
            );
            Ok(())
        }

        pub fn unbind_param(&mut self, param_name: &str) -> bool {
            self.bindings.remove(param_name).is_some()
        }

        pub fn is_bound(&self, param_name: &str) -> bool {
            self.bindings.contains_key(param_name)
        }

        pub fn validate_all_params(&self) -> Result<(), Vec<String>> {
            let mut errors = Vec::new();


            for (name, def) in &self.param_defs {
                if def.required && !self.param_values.contains_key(name) && !self.bindings.contains_key(name) {
                    errors.push(format!("Required parameter '{}' is not set", name));
                }
            }


            for (name, value) in &self.param_values {
                if let Some(def) = self.param_defs.get(name) {
                    if let Err(e) = def.validate(value) {
                        errors.push(format!("Parameter '{}': {}", name, e));
                    }
                }
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn test_float_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("opacity", ParamValue::Float(1.0))
            .with_range(0.0, 1.0);
        node.add_param(param);

        assert!(node.set_param("opacity", ParamValue::Float(0.5)).is_ok());
        assert_eq!(node.get_param("opacity"), Some(&ParamValue::Float(0.5)));
    }

    #[test]
    fn test_float_parameter_range_validation() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("opacity", ParamValue::Float(1.0))
            .with_range(0.0, 1.0);
        node.add_param(param);


        assert!(node.set_param("opacity", ParamValue::Float(0.0)).is_ok());
        assert!(node.set_param("opacity", ParamValue::Float(1.0)).is_ok());
        assert!(node.set_param("opacity", ParamValue::Float(0.5)).is_ok());


        assert!(node.set_param("opacity", ParamValue::Float(-0.1)).is_err());
        assert!(node.set_param("opacity", ParamValue::Float(1.1)).is_err());
    }

    #[test]
    fn test_int_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("iterations", ParamValue::Int(1))
            .with_range(1.0, 100.0);
        node.add_param(param);

        assert!(node.set_param("iterations", ParamValue::Int(50)).is_ok());
        assert_eq!(node.get_param("iterations"), Some(&ParamValue::Int(50)));
    }

    #[test]
    fn test_bool_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("enabled", ParamValue::Bool(true));
        node.add_param(param);

        assert!(node.set_param("enabled", ParamValue::Bool(false)).is_ok());
        assert_eq!(node.get_param("enabled"), Some(&ParamValue::Bool(false)));
    }

    #[test]
    fn test_color_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("tint", ParamValue::Color([1.0, 1.0, 1.0, 1.0]));
        node.add_param(param);

        let new_color = ParamValue::Color([1.0, 0.0, 0.0, 1.0]);
        assert!(node.set_param("tint", new_color.clone()).is_ok());
        assert_eq!(node.get_param("tint"), Some(&new_color));
    }

    #[test]
    fn test_vector_parameters() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param2 = ParamDef::new("position", ParamValue::Vector2([0.0, 0.0]));
        let param3 = ParamDef::new("scale", ParamValue::Vector3([1.0, 1.0, 1.0]));
        node.add_param(param2);
        node.add_param(param3);

        assert!(node.set_param("position", ParamValue::Vector2([100.0, 200.0])).is_ok());
        assert!(node.set_param("scale", ParamValue::Vector3([2.0, 2.0, 2.0])).is_ok());
    }

    #[test]
    fn test_enum_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let options = vec!["linear".to_string(), "ease-in".to_string(), "ease-out".to_string()];
        let param = ParamDef::new("interpolation", ParamValue::Enum("linear".to_string(), options.clone()));
        node.add_param(param);

        assert!(node.set_param("interpolation", ParamValue::Enum("ease-in".to_string(), options.clone())).is_ok());


        let result = node.set_param("interpolation", ParamValue::Enum("invalid".to_string(), options));
        assert!(result.is_err());
    }

    #[test]
    fn test_type_mismatch() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("opacity", ParamValue::Float(1.0));
        node.add_param(param);


        let result = node.set_param("opacity", ParamValue::Int(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_parameter_binding() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("opacity", ParamValue::Float(1.0));
        node.add_param(param);

        assert!(node.bind_param("opacity", "source_node", "output_value").is_ok());
        assert!(node.is_bound("opacity"));

        assert!(node.unbind_param("opacity"));
        assert!(!node.is_bound("opacity"));
    }

    #[test]
    fn test_binding_nonexistent_param() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let result = node.bind_param("nonexistent", "source", "output");
        assert!(result.is_err());
    }

    #[test]
    fn test_required_parameter_validation() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let param = ParamDef::new("required_input", ParamValue::Float(0.0)).required();
        node.add_param(param);


        node.param_values.remove("required_input");

        let result = node.validate_all_params();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_parameters() {
        let mut node = ParameterizedNode::new("color_correct", "Color Correction");

        node.add_param(ParamDef::new("brightness", ParamValue::Float(0.0)).with_range(-1.0, 1.0));
        node.add_param(ParamDef::new("contrast", ParamValue::Float(1.0)).with_range(0.0, 2.0));
        node.add_param(ParamDef::new("saturation", ParamValue::Float(1.0)).with_range(0.0, 2.0));
        node.add_param(ParamDef::new("enabled", ParamValue::Bool(true)));

        assert!(node.set_param("brightness", ParamValue::Float(0.5)).is_ok());
        assert!(node.set_param("contrast", ParamValue::Float(1.2)).is_ok());
        assert!(node.set_param("saturation", ParamValue::Float(0.8)).is_ok());
        assert!(node.set_param("enabled", ParamValue::Bool(false)).is_ok());

        assert!(node.validate_all_params().is_ok());
    }

    #[test]
    fn test_default_values() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        node.add_param(ParamDef::new("opacity", ParamValue::Float(1.0)));
        node.add_param(ParamDef::new("enabled", ParamValue::Bool(true)));


        assert_eq!(node.get_param("opacity"), Some(&ParamValue::Float(1.0)));
        assert_eq!(node.get_param("enabled"), Some(&ParamValue::Bool(true)));
    }

    #[test]
    fn test_nonexistent_parameter() {
        let mut node = ParameterizedNode::new("node1", "Test Node");

        let result = node.set_param("nonexistent", ParamValue::Float(1.0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
