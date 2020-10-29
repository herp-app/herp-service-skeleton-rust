use serde::{Deserialize};
use serde_json::{json, Value};
use actix_web::{client, error, Error};
use super::{Credentials, Service};

#[derive(Deserialize, PartialEq, Debug)]
pub struct Herp {
    pub (super) host : String,
    pub (super) endpoints : Value,
}

impl Herp{
    pub fn new(host : String, endpoints: Value) -> Herp{
	Herp{host: host, endpoints: endpoints}
    }
    
    /// This function needs to be called to to register the service to herp, so herp does know about the service. 
    /// This funciton passes all the basic information about your service to herp. 
    pub async fn register(&self, service: &Service) -> Result<(), Error>{
	let url = self.get_url("register");
	let client = client::Client::default();
	let _response = client
	    .post(&url)
	    .send_json(&service).await?;
	Ok(())
    }

    fn get_url(&self, endpoint: &str) -> String{
	["http://",self.host.as_str(),self.endpoints[endpoint].as_str().unwrap()].join("")
    }

    async fn get_token(&self) -> Result<String,Error>{
	let client = client::Client::default();
	let url = self.get_url("login");
	let credentials = Credentials::load();

	let data = json!(
	    {
		"email": credentials.name,
		"password": credentials.password
	    });
	
	let response = client
	    .post(&url)
	    .send_json(&data).await?.json::<Value>().await?;

	let token = response["token"].as_str().unwrap().to_string();

	Ok(token)
    }

    async fn get_service_id(&self, token: &str, service : &Service) -> Result<String,Error>{
	let client = client::Client::default();
	let url = self.get_url("services");
	let response = client
	    .get(&url)
	    .header("content-type", "application/json")
	    .header("Authorization", token)
	    .send()
	    .await?.json::<Value>().await?;

	if let Value::Array(array) = &response["data"]{
	    let mut id = array
		.iter()
		.filter(|x|
			x["name"].as_str()==Some(service.name.as_str())
			&& x["title"].as_str()==Some(service.title.as_str()));
	    return Ok(id.next().unwrap()["_id"].as_str().unwrap().to_string())
	}
	Err(error::ErrorBadRequest("Service not found."))
    }

    async fn klick_install(&self, service : &Service) -> Result<String,Error>{
	if let Ok(token) = self.get_token().await{
	    let id = &self.get_service_id(token.as_str(), service).await?;
	    println!("{}",id.as_str());
	    println!("{}",token);
	    let client = client::Client::default();
	    let url = ["http://",&self.host,&self.endpoints["install"].as_str().unwrap(),"/" , id.as_str()].join("");
	    let response = client
		.get(&url)
		.header("content-type", "application/json")
		.header("Authorization", token)
		.send()
		.await?.json::<Value>().await?;
	    println!("response {:?}",response);
	    match response.as_str(){
		Some(value) => return Ok(value.to_string()),
		_ => return Err(error::ErrorBadRequest("Installaton rejected.")) 
	    }
	}
	Err(error::ErrorBadRequest("Authorization failed."))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::test::default_service;

    fn default_herp() -> Herp {
	let endpoints = json!({
	    "register" : "/services/register",
	    "login" : "/users/login",
	    "install" : "/services/install",
	    "services" : "/content/system/service"
	});
	Herp{
	    host: String::from("127.0.0.1:5050"),
	    endpoints : endpoints}
    }

    #[actix_rt::test] // Requirement: hERP is running.
    async fn register_integrationtest(){
	let herp = default_herp();
	let service = default_service();

	assert_eq!(herp.register(&service).await.unwrap(), ());
    }

    #[actix_rt::test]  // Requirement: hERP is running.
    #[should_panic(expected = "Connection refused")]
    async fn register_test_wrong_port(){
	let _result = Herp{
	    host: String::from("127.0.0.1:6000"),
	    endpoints: default_herp().endpoints}
	.register(&default_service()).await.unwrap();
    }

    
    
    #[actix_rt::test] // Requirement: hERP is running.
    async fn login_integrationtest(){
	let credentials = Credentials::new("your-email@example.com".to_string(),"admin".to_string());
	credentials.store();
	let token = default_herp().get_token().await.unwrap();
	assert_eq!(token.len(),171);
    }

    #[actix_rt::test] // Requirement: hERP is running.
    async fn get_service_id_integrationtest(){
	let credentials = Credentials{
	    name : "your-email@example.com".to_string(),
	    password: "admin".to_string()};
	credentials.store();

	let herp = default_herp();

	let service = default_service();
	
	let id = herp.get_service_id(
	    herp.get_token().await.unwrap().as_str(),
	    &service
	).await;
	println!("{:?}", id);
	assert_eq!(id.unwrap().len(),24);
    }

    
    #[actix_rt::test] // Requirement: hERP is running.
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Installaton rejected.\"")]
    async fn klick_install_integrationtest(){
	let credentials = Credentials{
	    name : "your-email@example.com".to_string(),
	    password: "admin".to_string()};
	credentials.store();
	
	let herp = default_herp();
	
	let service = default_service();
	
	herp.klick_install(&service).await.unwrap();
    }
}

