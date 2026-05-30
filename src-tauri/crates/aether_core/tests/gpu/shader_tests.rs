

#[cfg(test)]
mod tests {
    use std::collections::HashMap;


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ShaderType {
        Vertex,
        Fragment,
        Compute,
    }


    #[derive(Debug)]
    pub enum ShaderCompileResult {
        Success { bytecode: Vec<u8> },
        Error { message: String, line: Option<usize> },
    }


    pub struct MockShaderCompiler {
        pub supported_versions: Vec<String>,
        pub max_uniforms: usize,
        pub max_samplers: usize,
    }

    impl MockShaderCompiler {
        pub fn new() -> Self {
            Self {
                supported_versions: vec!["450".to_string(), "460".to_string()],
                max_uniforms: 16,
                max_samplers: 16,
            }
        }

        pub fn compile(&self, source: &str, shader_type: ShaderType) -> ShaderCompileResult {

            if source.is_empty() {
                return ShaderCompileResult::Error {
                    message: "Empty shader source".to_string(),
                    line: None,
                };
            }


            if !source.contains("#version") {
                return ShaderCompileResult::Error {
                    message: "Missing #version directive".to_string(),
                    line: Some(1),
                };
            }


            if !source.contains("void main()") && !source.contains("void main(void)") {
                return ShaderCompileResult::Error {
                    message: "Missing main function".to_string(),
                    line: None,
                };
            }


            match shader_type {
                ShaderType::Vertex => {
                    if !source.contains("gl_Position") && !source.contains("out vec") {
                        return ShaderCompileResult::Error {
                            message: "Vertex shader must set gl_Position or have outputs".to_string(),
                            line: None,
                        };
                    }
                }
                ShaderType::Fragment => {
                    if !source.contains("out vec4") && !source.contains("gl_FragColor") {
                        return ShaderCompileResult::Error {
                            message: "Fragment shader must have color output".to_string(),
                            line: None,
                        };
                    }
                }
                ShaderType::Compute => {
                    if !source.contains("layout(local_size") {
                        return ShaderCompileResult::Error {
                            message: "Compute shader must specify local_size".to_string(),
                            line: None,
                        };
                    }
                }
            }


            let uniform_count = source.matches("uniform").count();
            if uniform_count > self.max_uniforms {
                return ShaderCompileResult::Error {
                    message: format!("Too many uniforms: {} (max: {})", uniform_count, self.max_uniforms),
                    line: None,
                };
            }


            ShaderCompileResult::Success {
                bytecode: vec![0u8; 256],
            }
        }

        pub fn validate_syntax(&self, source: &str) -> Result<(), Vec<String>> {
            let mut errors = Vec::new();


            let open_braces = source.matches('{').count();
            let close_braces = source.matches('}').count();
            if open_braces != close_braces {
                errors.push(format!("Unbalanced braces: {} open, {} close", open_braces, close_braces));
            }


            let open_parens = source.matches('(').count();
            let close_parens = source.matches(')').count();
            if open_parens != close_parens {
                errors.push(format!("Unbalanced parentheses: {} open, {} close", open_parens, close_parens));
            }


            for (i, line) in source.lines().enumerate() {
                let trimmed = line.trim();
                if !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("#")
                    && !trimmed.ends_with('{')
                    && !trimmed.ends_with('}')
                    && !trimmed.ends_with(';')
                    && !trimmed.contains("void main")
                    && !trimmed.starts_with("if")
                    && !trimmed.starts_with("for")
                    && !trimmed.starts_with("while")
                {

                }
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }


    #[derive(Debug, Clone)]
    pub struct UniformBinding {
        pub name: String,
        pub binding: u32,
        pub uniform_type: UniformType,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum UniformType {
        Float,
        Vec2,
        Vec3,
        Vec4,
        Mat3,
        Mat4,
        Sampler2D,
        Int,
        Bool,
    }


    pub struct ShaderProgram {
        pub vertex_source: String,
        pub fragment_source: String,
        pub uniforms: HashMap<String, UniformBinding>,
        pub compiled: bool,
    }

    impl ShaderProgram {
        pub fn new(vertex: &str, fragment: &str) -> Self {
            Self {
                vertex_source: vertex.to_string(),
                fragment_source: fragment.to_string(),
                uniforms: HashMap::new(),
                compiled: false,
            }
        }

        pub fn add_uniform(&mut self, name: &str, binding: u32, uniform_type: UniformType) {
            self.uniforms.insert(name.to_string(), UniformBinding {
                name: name.to_string(),
                binding,
                uniform_type,
            });
        }

        pub fn compile(&mut self, compiler: &MockShaderCompiler) -> Result<(), String> {
            match compiler.compile(&self.vertex_source, ShaderType::Vertex) {
                ShaderCompileResult::Error { message, .. } => {
                    return Err(format!("Vertex shader error: {}", message));
                }
                _ => {}
            }

            match compiler.compile(&self.fragment_source, ShaderType::Fragment) {
                ShaderCompileResult::Error { message, .. } => {
                    return Err(format!("Fragment shader error: {}", message));
                }
                _ => {}
            }

            self.compiled = true;
            Ok(())
        }

        pub fn get_uniform(&self, name: &str) -> Option<&UniformBinding> {
            self.uniforms.get(name)
        }
    }

    const VALID_VERTEX_SHADER: &str = r#"
        #version 450
        layout(location = 0) in vec3 position;
        layout(location = 1) in vec2 texcoord;
        out vec2 v_texcoord;
        void main() {
            gl_Position = vec4(position, 1.0);
            v_texcoord = texcoord;
        }
    "#;

    const VALID_FRAGMENT_SHADER: &str = r#"
        #version 450
        in vec2 v_texcoord;
        out vec4 fragColor;
        uniform sampler2D u_texture;
        void main() {
            fragColor = texture(u_texture, v_texcoord);
        }
    "#;

    const VALID_COMPUTE_SHADER: &str = r#"
        #version 450
        layout(local_size_x = 16, local_size_y = 16) in;
        layout(rgba8, binding = 0) uniform image2D outputImage;
        void main() {
            ivec2 pos = ivec2(gl_GlobalInvocationID.xy);
            imageStore(outputImage, pos, vec4(1.0, 0.0, 0.0, 1.0));
        }
    "#;

    #[test]
    fn test_valid_vertex_shader_compilation() {
        let compiler = MockShaderCompiler::new();
        let result = compiler.compile(VALID_VERTEX_SHADER, ShaderType::Vertex);

        match result {
            ShaderCompileResult::Success { bytecode } => {
                assert!(!bytecode.is_empty());
            }
            ShaderCompileResult::Error { message, .. } => {
                panic!("Compilation failed: {}", message);
            }
        }
    }

    #[test]
    fn test_valid_fragment_shader_compilation() {
        let compiler = MockShaderCompiler::new();
        let result = compiler.compile(VALID_FRAGMENT_SHADER, ShaderType::Fragment);

        match result {
            ShaderCompileResult::Success { bytecode } => {
                assert!(!bytecode.is_empty());
            }
            ShaderCompileResult::Error { message, .. } => {
                panic!("Compilation failed: {}", message);
            }
        }
    }

    #[test]
    fn test_valid_compute_shader_compilation() {
        let compiler = MockShaderCompiler::new();
        let result = compiler.compile(VALID_COMPUTE_SHADER, ShaderType::Compute);

        match result {
            ShaderCompileResult::Success { bytecode } => {
                assert!(!bytecode.is_empty());
            }
            ShaderCompileResult::Error { message, .. } => {
                panic!("Compilation failed: {}", message);
            }
        }
    }

    #[test]
    fn test_empty_shader_fails() {
        let compiler = MockShaderCompiler::new();
        let result = compiler.compile("", ShaderType::Vertex);

        match result {
            ShaderCompileResult::Error { message, .. } => {
                assert!(message.contains("Empty"));
            }
            _ => panic!("Expected compilation to fail"),
        }
    }

    #[test]
    fn test_missing_version_directive() {
        let compiler = MockShaderCompiler::new();
        let shader = r#"
            void main() {
                gl_Position = vec4(0.0);
            }
        "#;
        let result = compiler.compile(shader, ShaderType::Vertex);

        match result {
            ShaderCompileResult::Error { message, .. } => {
                assert!(message.contains("version"));
            }
            _ => panic!("Expected compilation to fail"),
        }
    }

    #[test]
    fn test_missing_main_function() {
        let compiler = MockShaderCompiler::new();
        let shader = r#"
            #version 450
            out vec4 color;
        "#;
        let result = compiler.compile(shader, ShaderType::Fragment);

        match result {
            ShaderCompileResult::Error { message, .. } => {
                assert!(message.contains("main"));
            }
            _ => panic!("Expected compilation to fail"),
        }
    }

    #[test]
    fn test_compute_shader_missing_local_size() {
        let compiler = MockShaderCompiler::new();
        let shader = r#"
            #version 450
            void main() {
            }
        "#;
        let result = compiler.compile(shader, ShaderType::Compute);

        match result {
            ShaderCompileResult::Error { message, .. } => {
                assert!(message.contains("local_size"));
            }
            _ => panic!("Expected compilation to fail"),
        }
    }

    #[test]
    fn test_shader_program_compilation() {
        let compiler = MockShaderCompiler::new();
        let mut program = ShaderProgram::new(VALID_VERTEX_SHADER, VALID_FRAGMENT_SHADER);

        assert!(!program.compiled);
        assert!(program.compile(&compiler).is_ok());
        assert!(program.compiled);
    }

    #[test]
    fn test_shader_uniform_binding() {
        let mut program = ShaderProgram::new(VALID_VERTEX_SHADER, VALID_FRAGMENT_SHADER);

        program.add_uniform("u_texture", 0, UniformType::Sampler2D);
        program.add_uniform("u_time", 1, UniformType::Float);
        program.add_uniform("u_resolution", 2, UniformType::Vec2);

        assert!(program.get_uniform("u_texture").is_some());
        assert_eq!(program.get_uniform("u_texture").unwrap().uniform_type, UniformType::Sampler2D);
        assert!(program.get_uniform("nonexistent").is_none());
    }

    #[test]
    fn test_syntax_validation_balanced_braces() {
        let compiler = MockShaderCompiler::new();

        let valid = "void main() { if (true) { } }";
        assert!(compiler.validate_syntax(valid).is_ok());

        let invalid = "void main() { if (true) { }";
        assert!(compiler.validate_syntax(invalid).is_err());
    }

    #[test]
    fn test_too_many_uniforms() {
        let mut compiler = MockShaderCompiler::new();
        compiler.max_uniforms = 2;

        let shader = r#"
            #version 450
            uniform float u1;
            uniform float u2;
            uniform float u3;
            out vec4 color;
            void main() {
                color = vec4(u1, u2, u3, 1.0);
            }
        "#;

        let result = compiler.compile(shader, ShaderType::Fragment);
        match result {
            ShaderCompileResult::Error { message, .. } => {
                assert!(message.contains("Too many uniforms"));
            }
            _ => panic!("Expected compilation to fail"),
        }
    }
}
