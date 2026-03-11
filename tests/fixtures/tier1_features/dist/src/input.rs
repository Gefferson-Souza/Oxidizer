// Note: async/await code - formatting skipped for edition compatibility
# [derive (Debug , Clone , Copy , PartialEq , Eq , serde :: Serialize , serde :: Deserialize)] enum Status { # [serde (rename = "active")] Active , # [serde (rename = "inactive")] Inactive }
async fn run_test (arr : Vec < f64 > , status : Status) -> Result < String , crate :: AppError > { let mut sum = 0f64 ; for item in arr { let mut val = if item > 5f64 { 5f64 } else { item } ; } if status == Status :: Active { return Ok (String :: from ("Active")) ; } return Ok (String :: from ("Done")) ; }
