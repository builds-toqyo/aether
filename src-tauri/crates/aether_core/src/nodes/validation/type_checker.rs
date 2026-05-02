use crate::nodes::NodeError;
use aether_types::{PinDataType, ParameterValue};
use log::debug;

/// Type compatibility checker for pins and parameters
pub struct TypeChecker;

impl TypeChecker {
    /// Check if two pin data types are compatible
    pub fn check_type_compatibility(output_type: &PinDataType, input_type: &PinDataType) -> Result<(), NodeError> {
        debug!("Checking type compatibility: {:?} -> {:?}", output_type, input_type);
        
        if output_type == input_type {
            return Ok(());
        }
        
        match (output_type, input_type) {
            // Float can be converted to integer and vectors
            (PinDataType::Float, PinDataType::Integer) => Ok(()),
            (PinDataType::Float, PinDataType::Vector2) => Ok(()),
            (PinDataType::Float, PinDataType::Vector3) => Ok(()),
            (PinDataType::Float, PinDataType::Vector4) => Ok(()),
            
            // Vector upcasting
            (PinDataType::Vector2, PinDataType::Vector3) => Ok(()),
            (PinDataType::Vector2, PinDataType::Vector4) => Ok(()),
            (PinDataType::Vector3, PinDataType::Vector2) => Ok(()),
            (PinDataType::Vector3, PinDataType::Vector4) => Ok(()),
            (PinDataType::Vector4, PinDataType::Vector2) => Ok(()),
            (PinDataType::Vector4, PinDataType::Vector3) => Ok(()),
            
            // Color and Vector4 compatibility
            (PinDataType::Vector4, PinDataType::Color) => Ok(()),
            (PinDataType::Color, PinDataType::Vector4) => Ok(()),
            
            // Array type compatibility (recursive)
            (PinDataType::Array(ref output_inner), PinDataType::Array(ref input_inner)) => {
                Self::check_type_compatibility(output_inner, input_inner)
            }
            
            _ => Err(NodeError::TypeMismatch {
                expected: format!("{:?}", input_type),
                actual: format!("{:?}", output_type),
            }),
        }
    }
    
    /// Check if two types are compatible (non-erroring version)
    pub fn are_types_compatible(output_type: &PinDataType, input_type: &PinDataType) -> bool {
        Self::check_type_compatibility(output_type, input_type).is_ok()
    }
    
    /// Check if two parameter values are compatible
    pub fn are_values_compatible(value1: &ParameterValue, value2: &ParameterValue) -> bool {
        match (value1, value2) {
            (ParameterValue::Float(_), ParameterValue::Float(_)) => true,
            (ParameterValue::Integer(_), ParameterValue::Integer(_)) => true,
            (ParameterValue::Boolean(_), ParameterValue::Boolean(_)) => true,
            (ParameterValue::String(_), ParameterValue::String(_)) => true,
            (ParameterValue::Vector2(_, _), ParameterValue::Vector2(_, _)) => true,
            (ParameterValue::Vector3(_, _, _), ParameterValue::Vector3(_, _, _)) => true,
            (ParameterValue::Vector4(_, _, _, _), ParameterValue::Vector4(_, _, _, _)) => true,
            (ParameterValue::Color(_, _, _, _), ParameterValue::Color(_, _, _, _)) => true,
            (ParameterValue::Array(_), ParameterValue::Array(_)) => true,
            (ParameterValue::Image(_), ParameterValue::Image(_)) => true,
            (ParameterValue::None, ParameterValue::None) => true,
            _ => false,
        }
    }
    
    /// Check if a value is within bounds
    pub fn is_value_in_bounds(value: &ParameterValue, min: &ParameterValue, max: &ParameterValue) -> Result<bool, NodeError> {
        match (value, min, max) {
            (ParameterValue::Float(val), ParameterValue::Float(min_val), ParameterValue::Float(max_val)) => {
                Ok(val >= min_val && val <= max_val)
            }
            (ParameterValue::Integer(val), ParameterValue::Integer(min_val), ParameterValue::Integer(max_val)) => {
                Ok(val >= min_val && val <= max_val)
            }
            (ParameterValue::String(val), ParameterValue::String(min_val), ParameterValue::String(max_val)) => {
                Ok(val >= min_val && val <= max_val)
            }
            _ => Err(NodeError::InvalidParameterValue(
                "Incompatible value types for bounds checking".to_string()
            )),
        }
    }
    
    /// Check if value is greater than or equal to another
    pub fn is_value_ge(value: &ParameterValue, min: &ParameterValue) -> Result<bool, NodeError> {
        match (value, min) {
            (ParameterValue::Float(val), ParameterValue::Float(min_val)) => Ok(val >= min_val),
            (ParameterValue::Integer(val), ParameterValue::Integer(min_val)) => Ok(val >= min_val),
            (ParameterValue::String(val), ParameterValue::String(min_val)) => Ok(val >= min_val),
            _ => Err(NodeError::InvalidParameterValue(
                "Incompatible value types for comparison".to_string()
            )),
        }
    }
    
    /// Check if value is less than or equal to another
    pub fn is_value_le(value: &ParameterValue, max: &ParameterValue) -> Result<bool, NodeError> {
        match (value, max) {
            (ParameterValue::Float(val), ParameterValue::Float(max_val)) => Ok(val <= max_val),
            (ParameterValue::Integer(val), ParameterValue::Integer(max_val)) => Ok(val <= max_val),
            (ParameterValue::String(val), ParameterValue::String(max_val)) => Ok(val <= max_val),
            _ => Err(NodeError::InvalidParameterValue(
                "Incompatible value types for comparison".to_string()
            )),
        }
    }
    
    /// Convert a value to a compatible type if possible
    pub fn convert_value(value: &ParameterValue, target_type: &PinDataType) -> Result<ParameterValue, NodeError> {
        match (value, target_type) {
            // Float to Integer conversion
            (ParameterValue::Float(val), PinDataType::Integer) => {
                Ok(ParameterValue::Integer(val as i64))
            }
            
            // Float to Vector2 conversion (broadcast)
            (ParameterValue::Float(val), PinDataType::Vector2) => {
                Ok(ParameterValue::Vector2(*val, *val))
            }
            
            // Float to Vector3 conversion (broadcast)
            (ParameterValue::Float(val), PinDataType::Vector3) => {
                Ok(ParameterValue::Vector3(*val, *val, *val))
            }
            
            // Float to Vector4 conversion (broadcast)
            (ParameterValue::Float(val), PinDataType::Vector4) => {
                Ok(ParameterValue::Vector4(*val, *val, *val, *val))
            }
            
            // Vector2 to Vector3 conversion (pad with 0)
            (ParameterValue::Vector2(x, y), PinDataType::Vector3) => {
                Ok(ParameterValue::Vector3(*x, *y, 0.0))
            }
            
            // Vector2 to Vector4 conversion (pad with 0, 1)
            (ParameterValue::Vector2(x, y), PinDataType::Vector4) => {
                Ok(ParameterValue::Vector4(*x, *y, 0.0, 1.0))
            }
            
            // Vector3 to Vector2 conversion (drop z)
            (ParameterValue::Vector3(x, y, _), PinDataType::Vector2) => {
                Ok(ParameterValue::Vector2(*x, *y))
            }
            
            // Vector3 to Vector4 conversion (pad with 1)
            (ParameterValue::Vector3(x, y, z), PinDataType::Vector4) => {
                Ok(ParameterValue::Vector4(*x, *y, *z, 1.0))
            }
            
            // Vector4 to Vector2 conversion (drop z, w)
            (ParameterValue::Vector4(x, y, _, _), PinDataType::Vector2) => {
                Ok(ParameterValue::Vector2(*x, *y))
            }
            
            // Vector4 to Vector3 conversion (drop w)
            (ParameterValue::Vector4(x, y, z, _), PinDataType::Vector3) => {
                Ok(ParameterValue::Vector3(*x, *y, *z))
            }
            
            // Vector4 to Color conversion
            (ParameterValue::Vector4(r, g, b, a), PinDataType::Color) => {
                Ok(ParameterValue::Color(*r, *g, *b, *a))
            }
            
            // Color to Vector4 conversion
            (ParameterValue::Color(r, g, b, a), PinDataType::Vector4) => {
                Ok(ParameterValue::Vector4(*r, *g, *b, *a))
            }
            
            // Same type, no conversion needed
            _ if Self::get_value_type(value) == Some(target_type) => Ok(value.clone()),
            
            _ => Err(NodeError::TypeMismatch {
                expected: format!("{:?}", target_type),
                actual: format!("{:?}", value),
            }),
        }
    }
    
    /// Get the PinDataType for a ParameterValue
    pub fn get_value_type(value: &ParameterValue) -> Option<&PinDataType> {
        match value {
            ParameterValue::Float(_) => Some(&PinDataType::Float),
            ParameterValue::Integer(_) => Some(&PinDataType::Integer),
            ParameterValue::Boolean(_) => Some(&PinDataType::Boolean),
            ParameterValue::String(_) => Some(&PinDataType::String),
            ParameterValue::Vector2(_, _) => Some(&PinDataType::Vector2),
            ParameterValue::Vector3(_, _, _) => Some(&PinDataType::Vector3),
            ParameterValue::Vector4(_, _, _, _) => Some(&PinDataType::Vector4),
            ParameterValue::Color(_, _, _, _) => Some(&PinDataType::Color),
            ParameterValue::Array(_) => Some(&PinDataType::Array(Box::new(PinDataType::Float))), // Default
            ParameterValue::Image(_) => Some(&PinDataType::Image),
            ParameterValue::None => None,
        }
    }
    
    /// Check if a type is numeric
    pub fn is_numeric_type(data_type: &PinDataType) -> bool {
        matches!(data_type, PinDataType::Float | PinDataType::Integer)
    }
    
    /// Check if a type is a vector type
    pub fn is_vector_type(data_type: &PinDataType) -> bool {
        matches!(data_type, PinDataType::Vector2 | PinDataType::Vector3 | PinDataType::Vector4)
    }
    
    /// Check if a type is a color type
    pub fn is_color_type(data_type: &PinDataType) -> bool {
        matches!(data_type, PinDataType::Color)
    }
    
    /// Get the component count for a type
    pub fn get_component_count(data_type: &PinDataType) -> usize {
        match data_type {
            PinDataType::Float => 1,
            PinDataType::Integer => 1,
            PinDataType::Boolean => 1,
            PinDataType::String => 1,
            PinDataType::Vector2 => 2,
            PinDataType::Vector3 => 3,
            PinDataType::Vector4 => 4,
            PinDataType::Color => 4,
            PinDataType::Array(_) => 0, // Variable
            PinDataType::Image => 1,
        }
    }
    
    /// Get the size in bytes for a type (approximate)
    pub fn get_type_size(data_type: &PinDataType) -> usize {
        match data_type {
            PinDataType::Float => 4,
            PinDataType::Integer => 8,
            PinDataType::Boolean => 1,
            PinDataType::String => 8, // Pointer
            PinDataType::Vector2 => 8,
            PinDataType::Vector3 => 12,
            PinDataType::Vector4 => 16,
            PinDataType::Color => 16,
            PinDataType::Array(_) => 8, // Pointer
            PinDataType::Image => 8, // Pointer/UUID
        }
    }
    
    /// Check if conversion from source to target type is lossy
    pub fn is_conversion_lossy(source_type: &PinDataType, target_type: &PinDataType) -> bool {
        match (source_type, target_type) {
            // Vector downcasting is lossy
            (PinDataType::Vector3, PinDataType::Vector2) => true,
            (PinDataType::Vector4, PinDataType::Vector2) => true,
            (PinDataType::Vector4, PinDataType::Vector3) => true,
            
            // Float to Integer is lossy
            (PinDataType::Float, PinDataType::Integer) => true,
            
            // Same type is not lossy
            _ if source_type == target_type => false,
            
            // Vector upcasting is not lossy
            (PinDataType::Vector2, PinDataType::Vector3) => false,
            (PinDataType::Vector2, PinDataType::Vector4) => false,
            (PinDataType::Vector3, PinDataType::Vector4) => false,
            
            // Color/Vector4 conversion is not lossy
            (PinDataType::Vector4, PinDataType::Color) => false,
            (PinDataType::Color, PinDataType::Vector4) => false,
            
            // Other conversions
            _ => false,
        }
    }
    
    /// Get conversion cost (lower is better)
    pub fn get_conversion_cost(source_type: &PinDataType, target_type: &PinDataType) -> u32 {
        if source_type == target_type {
            return 0;
        }
        
        match (source_type, target_type) {
            // Simple numeric conversions
            (PinDataType::Float, PinDataType::Integer) => 1,
            (PinDataType::Integer, PinDataType::Float) => 1,
            
            // Vector broadcasting
            (PinDataType::Float, PinDataType::Vector2) => 2,
            (PinDataType::Float, PinDataType::Vector3) => 3,
            (PinDataType::Float, PinDataType::Vector4) => 4,
            
            // Vector reshaping
            (PinDataType::Vector2, PinDataType::Vector3) => 2,
            (PinDataType::Vector2, PinDataType::Vector4) => 3,
            (PinDataType::Vector3, PinDataType::Vector2) => 1, // Lossy
            (PinDataType::Vector3, PinDataType::Vector4) => 2,
            (PinDataType::Vector4, PinDataType::Vector2) => 2, // Lossy
            (PinDataType::Vector4, PinDataType::Vector3) => 1, // Lossy
            
            // Color/Vector4
            (PinDataType::Vector4, PinDataType::Color) => 1,
            (PinDataType::Color, PinDataType::Vector4) => 1,
            
            // Array conversions (expensive)
            (PinDataType::Array(_), PinDataType::Array(_)) => 10,
            
            // Incompatible
            _ => u32::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_compatibility() {
        // Same types should be compatible
        assert!(TypeChecker::are_types_compatible(&PinDataType::Float, &PinDataType::Float));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector3, &PinDataType::Vector3));
        
        // Float to vector should be compatible
        assert!(TypeChecker::are_types_compatible(&PinDataType::Float, &PinDataType::Vector2));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Float, &PinDataType::Vector3));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Float, &PinDataType::Vector4));
        
        // Vector upcasting should be compatible
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector2, &PinDataType::Vector3));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector2, &PinDataType::Vector4));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector3, &PinDataType::Vector4));
        
        // Vector downcasting should be compatible
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector3, &PinDataType::Vector2));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector4, &PinDataType::Vector2));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector4, &PinDataType::Vector3));
        
        // Color/Vector4 compatibility
        assert!(TypeChecker::are_types_compatible(&PinDataType::Vector4, &PinDataType::Color));
        assert!(TypeChecker::are_types_compatible(&PinDataType::Color, &PinDataType::Vector4));
        
        // Incompatible types
        assert!(!TypeChecker::are_types_compatible(&PinDataType::Float, &PinDataType::Boolean));
        assert!(!TypeChecker::are_types_compatible(&PinDataType::String, &PinDataType::Vector3));
    }
    
    #[test]
    fn test_value_conversion() {
        // Float to Integer
        let result = TypeChecker::convert_value(&ParameterValue::Float(3.7), &PinDataType::Integer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ParameterValue::Integer(3));
        
        // Float to Vector2
        let result = TypeChecker::convert_value(&ParameterValue::Float(1.5), &PinDataType::Vector2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ParameterValue::Vector2(1.5, 1.5));
        
        // Vector2 to Vector3
        let result = TypeChecker::convert_value(&ParameterValue::Vector2(1.0, 2.0), &PinDataType::Vector3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ParameterValue::Vector3(1.0, 2.0, 0.0));
        
        // Vector3 to Vector2
        let result = TypeChecker::convert_value(&ParameterValue::Vector3(1.0, 2.0, 3.0), &PinDataType::Vector2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ParameterValue::Vector2(1.0, 2.0));
        
        // Color to Vector4
        let result = TypeChecker::convert_value(&ParameterValue::Color(1.0, 0.5, 0.25, 1.0), &PinDataType::Vector4);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ParameterValue::Vector4(1.0, 0.5, 0.25, 1.0));
        
        // Same type conversion
        let original = ParameterValue::Float(2.5);
        let result = TypeChecker::convert_value(&original, &PinDataType::Float);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original);
    }
    
    #[test]
    fn test_value_bounds() {
        // Float bounds
        let value = ParameterValue::Float(5.0);
        let min = ParameterValue::Float(1.0);
        let max = ParameterValue::Float(10.0);
        
        assert!(TypeChecker::is_value_in_bounds(&value, &min, &max).unwrap());
        
        let max_out = ParameterValue::Float(3.0);
        assert!(!TypeChecker::is_value_in_bounds(&value, &min, &max_out).unwrap());
        
        // Integer bounds
        let int_value = ParameterValue::Integer(5);
        let int_min = ParameterValue::Integer(1);
        let int_max = ParameterValue::Integer(10);
        
        assert!(TypeChecker::is_value_in_bounds(&int_value, &int_min, &int_max).unwrap());
    }
    
    #[test]
    fn test_type_properties() {
        assert!(TypeChecker::is_numeric_type(&PinDataType::Float));
        assert!(TypeChecker::is_numeric_type(&PinDataType::Integer));
        assert!(!TypeChecker::is_numeric_type(&PinDataType::String));
        
        assert!(TypeChecker::is_vector_type(&PinDataType::Vector2));
        assert!(TypeChecker::is_vector_type(&PinDataType::Vector3));
        assert!(TypeChecker::is_vector_type(&PinDataType::Vector4));
        assert!(!TypeChecker::is_vector_type(&PinDataType::Float));
        
        assert!(TypeChecker::is_color_type(&PinDataType::Color));
        assert!(!TypeChecker::is_color_type(&PinDataType::Vector4));
        
        assert_eq!(TypeChecker::get_component_count(&PinDataType::Float), 1);
        assert_eq!(TypeChecker::get_component_count(&PinDataType::Vector2), 2);
        assert_eq!(TypeChecker::get_component_count(&PinDataType::Vector3), 3);
        assert_eq!(TypeChecker::get_component_count(&PinDataType::Vector4), 4);
        assert_eq!(TypeChecker::get_component_count(&PinDataType::Color), 4);
    }
    
    #[test]
    fn test_conversion_lossy() {
        // Same type is not lossy
        assert!(!TypeChecker::is_conversion_lossy(&PinDataType::Float, &PinDataType::Float));
        
        // Vector downcasting is lossy
        assert!(TypeChecker::is_conversion_lossy(&PinDataType::Vector3, &PinDataType::Vector2));
        assert!(TypeChecker::is_conversion_lossy(&PinDataType::Vector4, &PinDataType::Vector3));
        
        // Vector upcasting is not lossy
        assert!(!TypeChecker::is_conversion_lossy(&PinDataType::Vector2, &PinDataType::Vector3));
        assert!(!TypeChecker::is_conversion_lossy(&PinDataType::Vector3, &PinDataType::Vector4));
        
        // Float to Integer is lossy
        assert!(TypeChecker::is_conversion_lossy(&PinDataType::Float, &PinDataType::Integer));
    }
    
    #[test]
    fn test_conversion_cost() {
        // Same type has zero cost
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Float, &PinDataType::Float), 0);
        
        // Numeric conversions have low cost
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Float, &PinDataType::Integer), 1);
        
        // Vector broadcasting has moderate cost
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Float, &PinDataType::Vector2), 2);
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Float, &PinDataType::Vector3), 3);
        
        // Vector reshaping has various costs
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Vector2, &PinDataType::Vector3), 2);
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Vector3, &PinDataType::Vector2), 1); // Lossy
        
        // Incompatible types have maximum cost
        assert_eq!(TypeChecker::get_conversion_cost(&PinDataType::Float, &PinDataType::Boolean), u32::MAX);
    }
}
