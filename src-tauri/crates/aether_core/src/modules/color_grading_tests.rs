#[cfg(test)]
mod tests {
    use super::super::color_grading::*;
    use std::path::PathBuf;
    use anyhow::Result;

    // Helper function to create a test engine
    fn create_test_engine() -> Result<ColorGradingEngine> {
        let mut engine = ColorGradingEngine::new()?;
        Ok(engine)
    }

    #[test]
    fn test_create_engine() -> Result<()> {
        let engine = create_test_engine()?;
        assert!(!engine.is_initialized());
        Ok(())
    }

    #[test]
    fn test_initialize_engine() -> Result<()> {
        let mut engine = create_test_engine()?;
        engine.initialize()?;
        assert!(engine.is_initialized());
        engine.shutdown()?;
        assert!(!engine.is_initialized());
        Ok(())
    }

    #[test]
    fn test_color_adjustments() -> Result<()> {
        let mut engine = create_test_engine()?;
        
        // Test setting and getting brightness
        engine.set_brightness(0.5)?;
        assert_eq!(engine.get_brightness(), 0.5);
        
        // Test clamping of values
        engine.set_brightness(2.0)?;
        assert_eq!(engine.get_brightness(), 1.0);
        
        engine.set_brightness(-2.0)?;
        assert_eq!(engine.get_brightness(), -1.0);
        
        // Test other adjustments
        engine.set_contrast(1.5)?;
        assert_eq!(engine.get_contrast(), 1.5);
        
        engine.set_saturation(0.8)?;
        assert_eq!(engine.get_saturation(), 0.8);
        
        engine.set_gamma(1.2)?;
        assert_eq!(engine.get_gamma(), 1.2);
        
        engine.set_hue(0.3)?;
        assert_eq!(engine.get_hue(), 0.3);
        
        Ok(())
    }

    #[test]
    fn test_color_curves() -> Result<()> {
        let mut engine = create_test_engine()?;
        
        // Test setting RGB curve
        let rgb_curve = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.25, y: 0.3 },
            CurvePoint { x: 0.5, y: 0.6 },
            CurvePoint { x: 0.75, y: 0.8 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        
        engine.set_curve("rgb", &rgb_curve)?;
        
        // Test getting curve
        let retrieved_curve = engine.get_curve("rgb")?;
        assert_eq!(retrieved_curve.len(), rgb_curve.len());
        
        for (i, point) in retrieved_curve.iter().enumerate() {
            assert_eq!(point.x, rgb_curve[i].x);
            assert_eq!(point.y, rgb_curve[i].y);
        }
        
        // Test resetting curve
        engine.reset_curve("rgb")?;
        let reset_curve = engine.get_curve("rgb")?;
        assert_eq!(reset_curve.len(), 2);
        assert_eq!(reset_curve[0].x, 0.0);
        assert_eq!(reset_curve[0].y, 0.0);
        assert_eq!(reset_curve[1].x, 1.0);
        assert_eq!(reset_curve[1].y, 1.0);
        
        // Test invalid curve type
        let result = engine.set_curve("invalid", &rgb_curve);
        assert!(result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_lut_operations() -> Result<()> {
        let mut engine = create_test_engine()?;
        
        // Test LUT strength
        engine.set_lut_strength(0.75)?;
        assert_eq!(engine.get_lut_strength(), 0.75);
        
        // Test clearing LUT
        engine.clear_lut()?;
        assert!(engine.get_lut_file().is_none());
        
        Ok(())
    }

    #[test]
    fn test_scopes() -> Result<()> {
        let mut engine = create_test_engine()?;
        
        // Test enabling scope
        engine.enable_scope(ScopeType::Histogram, 256, 100, false)?;
        
        // Test getting configured scopes
        let scopes = engine.get_configured_scopes();
        assert!(scopes.contains(&ScopeType::Histogram));
        
        // Test disabling scope
        engine.disable_scope(ScopeType::Histogram)?;
        let scopes_after = engine.get_configured_scopes();
        assert!(!scopes_after.contains(&ScopeType::Histogram));
        
        Ok(())
    }

    #[test]
    fn test_presets() -> Result<()> {
        let mut engine = create_test_engine()?;
        
        // Set up some adjustments
        engine.set_brightness(0.2)?;
        engine.set_contrast(1.3)?;
        engine.set_saturation(0.8)?;
        
        // Create a preset
        engine.create_preset("test_preset")?;
        
        // Change adjustments
        engine.set_brightness(0.0)?;
        engine.set_contrast(1.0)?;
        engine.set_saturation(1.0)?;
        
        // Apply preset
        engine.apply_preset("test_preset")?;
        
        // Check values were restored
        assert_eq!(engine.get_brightness(), 0.2);
        assert_eq!(engine.get_contrast(), 1.3);
        assert_eq!(engine.get_saturation(), 0.8);
        
        // Delete preset
        engine.delete_preset("test_preset")?;
        
        // Try to apply deleted preset
        let result = engine.apply_preset("test_preset");
        assert!(result.is_err());
        
        Ok(())
    }
}
