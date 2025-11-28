// Note: async/await code - formatting skipped for edition compatibility
use super::super::utils::http_client::HttpClient;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct User { pub name : String }
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct UserProcessor { pub client : HttpClient < User > }
impl UserProcessor { pub fn new () -> Self { Self { client : HttpClient :: new (String :: from ("https://api.users.com")) } } pub fn new_di () -> Self { Self { client : Default :: default () } } pub async fn process (& self , id : String) -> Result < String , Box < dyn std :: error :: Error + Send + Sync >> { let user = self . client . clone () . get (String :: from ("/") + & id) . await ? ; return Ok (user . name . trim () . to_uppercase ()) ; } }
