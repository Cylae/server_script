// Mock test for GDHD
#[test]
fn test_gdhd_logic() {
   // This logic is duplicated from core/hardware but useful for regression
   let mem_gb = 2.0;
   let cpu_count = 1;
   let swap_gb = 0.5;

   let is_low_spec = mem_gb < 4.0 || cpu_count <= 2 || swap_gb < 1.0;
   assert!(is_low_spec);
}
