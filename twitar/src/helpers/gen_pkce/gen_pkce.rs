use pkce;

#[cfg(test)]
#[path = "./gen_pkce.test.rs"]
mod pkce_test;


#[derive(Debug, Clone, derive_more::From, derive_more::Display)]
#[display(fmt = self.0)]
pub struct Pkce(String);

impl Pkce {
    fn generate_pkce() -> String {
        let code_verify = pkce::code_verifier(128);
        let code_challenge = pkce::code_challenge(&code_verify);
        
        code_challenge
    }

    pub fn new() -> Self {
        let mut value = Self::generate_pkce();
        value.remove(value.len()-1);
        let base_64_url = value.replace("+", "-").replace("/", "_");

        Self(base_64_url)
    }

}

