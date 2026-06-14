
#[derive(Debug, Clone)]
pub struct HdrAnalysis {
    pub max_nits: f32,
    pub avg_nits: f32,
    pub dynamic_range: f32,
    pub peak_percentage: f32,
    pub content_type: HdrContentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrContentType {
    SdrUpscaled,
    LimitedHdr,
    EnhancedHdr,
    TrueHdr,
}

pub struct HdrAnalyzer;

impl HdrAnalyzer {
    pub fn classify_content_type(max_nits: f32, _avg_nits: f32) -> HdrContentType {
        if max_nits > 1000.0 {
            HdrContentType::TrueHdr
        } else if max_nits > 400.0 {
            HdrContentType::EnhancedHdr
        } else if max_nits > 200.0 {
            HdrContentType::LimitedHdr
        } else {
            HdrContentType::SdrUpscaled
        }
    }

    pub fn analyze_content(hdr_image: &super::types::HdrImage) -> HdrAnalysis {
        let mut max_nits: f32 = 0.0;
        let mut avg_nits: f32 = 0.0;
        let mut pixel_count = 0u64;

        for pixel in &hdr_image.data {
            let luminance = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
            max_nits = max_nits.max(luminance);
            avg_nits += luminance;
            pixel_count += 1;
        }

        avg_nits /= pixel_count as f32;

        HdrAnalysis {
            max_nits,
            avg_nits,
            dynamic_range: max_nits / avg_nits.max(0.1),
            peak_percentage: (max_nits / 10000.0 * 100.0).min(100.0),
            content_type: Self::classify_content_type(max_nits, avg_nits),
        }
    }

    pub fn calculate_quality_metrics(hdr_image: &super::types::HdrImage) -> HdrQualityMetrics {
        let mut saturation_avg = 0.0;
        let pixel_count = hdr_image.data.len() as f32;

        let mut min_luma = f32::MAX;
        let mut max_luma: f32 = 0.0;
        let mut highlight_pixels = 0u64;
        let mut shadow_pixels = 0u64;
        let mut contrast_ratio = 0.0;
        let mut highlight_preservation = 0.0;
        let mut shadow_preservation = 0.0;

        for pixel in &hdr_image.data {
            let luma = pixel.luminance();
            min_luma = min_luma.min(luma);
            max_luma = max_luma.max(luma);

            let r = pixel.r / (pixel.r + pixel.g + pixel.b).max(0.001);
            let g = pixel.g / (pixel.r + pixel.g + pixel.b).max(0.001);
            let b = pixel.b / (pixel.r + pixel.g + pixel.b).max(0.001);
            let saturation = 1.0 - (r.min(g).min(b).max(r.max(g).max(b)));
            saturation_avg += saturation;

            if luma > 0.8 * max_luma {
                highlight_pixels += 1;
            } else if luma < 0.2 * max_luma {
                shadow_pixels += 1;
            }
        }

        contrast_ratio = max_luma / min_luma.max(0.001);
        saturation_avg /= pixel_count;
        highlight_preservation = highlight_pixels as f32 / pixel_count;
        shadow_preservation = shadow_pixels as f32 / pixel_count;

        HdrQualityMetrics {
            contrast_ratio,
            saturation_avg,
            highlight_preservation,
            shadow_preservation,
            overall_quality: Self::calculate_overall_quality_static(contrast_ratio, saturation_avg),
        }
    }

    fn calculate_overall_quality_static(contrast_ratio: f32, saturation_avg: f32) -> f32 {
        let contrast_score = (contrast_ratio.log10() / 4.0).min(1.0).max(0.0);
        let saturation_score = saturation_avg;
        (contrast_score * 0.6 + saturation_score * 0.4) * 100.0
    }

    fn calculate_overall_quality(&self, contrast_ratio: f32, saturation_avg: f32) -> f32 {
        Self::calculate_overall_quality_static(contrast_ratio, saturation_avg)
    }

    pub fn generate_statistics(hdr_image: &super::types::HdrImage) -> HdrStatistics {
        let mut histogram = [0u32; 256];
        let mut total_pixels = 0u64;

        for pixel in &hdr_image.data {
            let luma = pixel.luminance();
            let bin = ((luma / hdr_image.max_nits * 255.0) as usize).min(255);
            histogram[bin] += 1;
            total_pixels += 1;
        }

        let mut cumulative = 0u64;
        let mut p5 = 0usize;
        let mut p50 = 0usize;
        let mut p95 = 0usize;

        for (bin, &count) in histogram.iter().enumerate() {
            cumulative += count as u64;
            let percentage = cumulative as f32 / total_pixels as f32;

            if percentage >= 0.05 && p5 == 0 {
                p5 = bin;
            }
            if percentage >= 0.5 && p50 == 0 {
                p50 = bin;
            }
            if percentage >= 0.95 && p95 == 0 {
                p95 = bin;
            }
        }

        HdrStatistics {
            histogram,
            total_pixels,
            percentile_5: p5 as f32 / 255.0 * hdr_image.max_nits,
            percentile_50: p50 as f32 / 255.0 * hdr_image.max_nits,
            percentile_95: p95 as f32 / 255.0 * hdr_image.max_nits,
        }
    }

    pub fn detect_issues(hdr_image: &super::types::HdrImage) -> Vec<HdrIssue> {
        let mut issues = Vec::new();

        let analysis = Self::analyze_content(hdr_image);

        let max_pixels = hdr_image.data.iter()
            .filter(|p| p.luminance() >= 0.99 * hdr_image.max_nits)
            .count();

        if max_pixels > hdr_image.data.len() / 100 {
            issues.push(HdrIssue::HighlightClipping(max_pixels as f32 / hdr_image.data.len() as f32));
        }

        if analysis.dynamic_range < 10.0 {
            issues.push(HdrIssue::LimitedDynamicRange(analysis.dynamic_range));
        }

        let unique_values = hdr_image.data.iter()
            .map(|p| (p.r * 100.0) as i32)
            .collect::<std::collections::HashSet<_>>()
            .len();

        if unique_values < 1000 {
            issues.push(HdrIssue::Banding);
        }

        let mut noise_sum = 0.0;
        let mut noise_count = 0;

        for i in 1..hdr_image.data.len().min(1000) {
            let current = hdr_image.data[i];
            let prev = hdr_image.data[i - 1];
            let diff = (current.luminance() - prev.luminance()).abs();
            noise_sum += diff;
            noise_count += 1;
        }

        let avg_noise = noise_sum / noise_count as f32;
        if avg_noise > hdr_image.max_nits * 0.01 {
            issues.push(HdrIssue::ExcessiveNoise(avg_noise));
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct HdrQualityMetrics {
    pub contrast_ratio: f32,
    pub saturation_avg: f32,
    pub highlight_preservation: f32,
    pub shadow_preservation: f32,
    pub overall_quality: f32,
}


#[derive(Debug, Clone)]
pub struct HdrStatistics {
    pub histogram: [u32; 256],
    pub total_pixels: u64,
    pub percentile_5: f32,
    pub percentile_50: f32,
    pub percentile_95: f32,
}


#[derive(Debug, Clone)]
pub enum HdrIssue {
    HighlightClipping(f32),
    LimitedDynamicRange(f32),
    Banding,
    ExcessiveNoise(f32),
}

impl HdrIssue {
    pub fn severity(&self) -> IssueSeverity {
        match self {
            HdrIssue::HighlightClipping(percentage) => {
                if *percentage > 0.1 { IssueSeverity::High } else { IssueSeverity::Medium }
            }
            HdrIssue::LimitedDynamicRange(ratio) => {
                if *ratio < 5.0 { IssueSeverity::High } else { IssueSeverity::Medium }
            }
            HdrIssue::Banding => IssueSeverity::Medium,
            HdrIssue::ExcessiveNoise(_) => IssueSeverity::Low,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HdrIssue::HighlightClipping(_) => "Highlight clipping detected - loss of detail in bright areas",
            HdrIssue::LimitedDynamicRange(_) => "Limited dynamic range - insufficient HDR characteristics",
            HdrIssue::Banding => "Color banding detected - insufficient bit depth or compression artifacts",
            HdrIssue::ExcessiveNoise(_) => "Excessive noise detected - may indicate sensor limitations or compression",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_content_classification() {

        assert_eq!(HdrAnalyzer::classify_content_type(100.0, 50.0), HdrContentType::SdrUpscaled);

        assert_eq!(HdrAnalyzer::classify_content_type(300.0, 100.0), HdrContentType::LimitedHdr);

        assert_eq!(HdrAnalyzer::classify_content_type(600.0, 200.0), HdrContentType::EnhancedHdr);

        assert_eq!(HdrAnalyzer::classify_content_type(1500.0, 500.0), HdrContentType::TrueHdr);
    }

    #[test]
    fn test_hdr_analysis() {
        let mut hdr_image = super::types::HdrImage::new(10, 10);

        for i in 0..100 {
            let x = i % 10;
            let y = i / 10;
            let pixel = super::types::HdrPixel {
                r: (i as f32 * 10.0),
                g: (i as f32 * 5.0),
                b: (i as f32 * 15.0),
            };
            hdr_image.set_pixel(x as u32, y as u32, pixel).unwrap();
        }

        let analysis = HdrAnalyzer::analyze_content(&hdr_image);

        assert!(analysis.max_nits > 0.0);
        assert!(analysis.avg_nits > 0.0);
        assert!(analysis.dynamic_range > 1.0);
        assert!(analysis.peak_percentage >= 0.0);
        assert!(analysis.peak_percentage <= 100.0);
    }

    #[test]
    fn test_quality_metrics() {
        let mut hdr_image = super::types::HdrImage::new(10, 10);

        hdr_image.set_pixel(0, 0, super::types::HdrPixel { r: 1000.0, g: 1000.0, b: 1000.0 }).unwrap();
        hdr_image.set_pixel(1, 1, super::types::HdrPixel { r: 0.1, g: 0.1, b: 0.1 }).unwrap();

        let metrics = HdrAnalyzer::calculate_quality_metrics(&hdr_image);

        assert!(metrics.contrast_ratio > 1.0);
        assert!(metrics.saturation_avg >= 0.0);
        assert!(metrics.saturation_avg <= 1.0);
        assert!(metrics.overall_quality >= 0.0);
        assert!(metrics.overall_quality <= 100.0);
    }

    #[test]
    fn test_hdr_statistics() {
        let mut hdr_image = super::types::HdrImage::new(10, 10);
        hdr_image.max_nits = 1000.0;

        for i in 0..50 {
            let x = i % 10;
            let y = i / 10;
            let pixel = super::types::HdrPixel {
                r: 500.0,
                g: 500.0,
                b: 500.0,
            };
            hdr_image.set_pixel(x as u32, y as u32, pixel).unwrap();
        }

        let stats = HdrAnalyzer::generate_statistics(&hdr_image);

        assert_eq!(stats.total_pixels, 100);
        assert!(stats.percentile_5 >= 0.0);
        assert!(stats.percentile_50 >= 0.0);
        assert!(stats.percentile_95 >= 0.0);
        assert!(stats.percentile_95 <= hdr_image.max_nits);
    }

    #[test]
    fn test_issue_detection() {
        let mut hdr_image = super::types::HdrImage::new(10, 10);
        hdr_image.max_nits = 1000.0;


        for i in 0..20 {
            let x = i % 10;
            let y = i / 10;
            let pixel = super::types::HdrPixel {
                r: 999.0,
                g: 999.0,
                b: 999.0,
            };
            hdr_image.set_pixel(x as u32, y as u32, pixel).unwrap();
        }

        let issues = HdrAnalyzer::detect_issues(&hdr_image);

        assert!(!issues.is_empty());

        for issue in &issues {
            match issue {
                HdrIssue::HighlightClipping(_) => {
                    assert!(matches!(issue.severity(), IssueSeverity::Medium));
                }
                HdrIssue::LimitedDynamicRange(_) => {
                    assert!(matches!(issue.severity(), IssueSeverity::Medium | IssueSeverity::High));
                }
                _ => {}
            }
        }
    }
}
