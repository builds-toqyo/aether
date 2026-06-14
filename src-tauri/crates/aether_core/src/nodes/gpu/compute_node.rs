use crate::nodes::{NodeExecutor, ExecutionContext, NodeResult, NodeError};
use aether_types::{Node, NodeType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::{Result, anyhow};

pub struct GpuComputeNode {
    pub node: Node,
    pub shader_source: String,
    pub binding_config: Option<crate::gpu::shaders::BindingConfig>,
    pub pipeline_handle: Option<crate::gpu::shaders::PipelineHandle>,
}

impl GpuComputeNode {
    pub fn new(node: Node, shader_source: String) -> Self {
        Self {
            node,
            shader_source,
            binding_config: None,
            pipeline_handle: None,
        }
    }

    pub fn with_binding_config(mut self, config: crate::gpu::shaders::BindingConfig) -> Self {
        self.binding_config = Some(config);
        self
    }

    fn ensure_pipeline_initialized(&mut self, context: &ExecutionContext) -> Result<()> {
        if self.pipeline_handle.is_some() {
            return Ok(());
        }

        let gpu_context = context.gpu_context
            .as_ref()
            .ok_or_else(|| anyhow!("GPU context not available"))?;

        let shader_system = gpu_context.shader_system
            .as_ref()
            .ok_or_else(|| anyhow!("Shader system not available"))?;

        let binding_config = self.binding_config
            .as_ref()
            .cloned()
            .unwrap_or_default();

        let pipeline = shader_system.create_compute_pipeline(
            &format!("compute_{}", self.node.id),
            &self.shader_source,
            &binding_config,
        )?;

        self.pipeline_handle = Some(pipeline);
        Ok(())
    }

    fn create_bindings(&self, context: &ExecutionContext) -> Result<HashMap<String, wgpu::BindingResource>> {
        let mut bindings = HashMap::new();

        for (param_name, param_value) in &context.global_parameters {
            match param_value {
                ParameterValue::Float(val) => {
                    // Create a uniform buffer for float parameters
                    // This would need actual buffer allocation from memory manager
                }
                ParameterValue::Int(val) => {
                    // Create a uniform buffer for int parameters
                }
                _ => {}
            }
        }

        Ok(bindings)
    }
}

impl NodeExecutor for GpuComputeNode {
    fn execute(&mut self, context: &mut ExecutionContext) -> NodeResult<()> {
        log::debug!("Executing GPU compute node: {}", self.node.name);

        if let Err(e) = self.ensure_pipeline_initialized(context) {
            return Err(NodeError::ExecutionFailed(format!("Failed to initialize pipeline: {}", e)));
        }

        let gpu_context = context.gpu_context
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU context not available".to_string()))?;

        let shader_system = gpu_context.shader_system
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("Shader system not available".to_string()))?;

        let pipeline = self.pipeline_handle
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("Pipeline not initialized".to_string()))?;

        let bindings = self.create_bindings(context)
            .map_err(|e| NodeError::ExecutionFailed(format!("Failed to create bindings: {}", e)))?;

        let bind_group = shader_system.create_bind_group(pipeline, &bindings)
            .map_err(|e| NodeError::ExecutionFailed(format!("Failed to create bind group: {}", e)))?;

        let device = gpu_context.device
            .as_ref()
            .ok_or_else(|| NodeError::ExecutionFailed("GPU device not available".to_string()))?;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("compute_encoder_{}", self.node.id)),
        });

        let workgroup_count = (
            (context.resolution.0 + 15) / 16,
            (context.resolution.1 + 15) / 16,
            1,
        );

        shader_system.execute_pipeline(pipeline, &bind_group, workgroup_count, &mut encoder)
            .map_err(|e| NodeError::ExecutionFailed(format!("Failed to execute pipeline: {}", e)))?;

        if let Some(queue) = &gpu_context.queue {
            queue.submit(Some(encoder.finish()));
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Custom("gpu_compute".to_string())
    }

    fn get_inputs(&self) -> Vec<Uuid> {
        self.node.inputs.iter().map(|pin| pin.id).collect()
    }

    fn get_outputs(&self) -> Vec<Uuid> {
        self.node.outputs.iter().map(|pin| pin.id).collect()
    }
}
