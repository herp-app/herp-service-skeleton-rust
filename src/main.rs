use actix_web::{get, post, web, App, HttpServer, HttpResponse, Error, error};
use serde_json::{json, Value};
use serde::{Deserialize};

use std::fs::{File};
use std::io::BufReader;

use herp_proxy::Herp;
use service::{Service, Credentials};

mod herp_proxy;

#[derive(Deserialize, PartialEq, Debug)]
struct Config{
    service: Service,
    herp : Herp,
    service_interface : Value
}

impl Config{
    fn new(service: Service, herp:Herp, service_interface: Value) -> Config{
	Config{service: service, herp: herp, service_interface: service_interface}
    }
    fn load_config() -> Config {
	Config {
	    service : Service{
		name: String::from("skeleton.herp.app"),
		title: String::from("Rust Skeleton Service"),
		description: String::from("A skeleton service with all necessary functionality to talk with herp but without any logic. Feel free to integrate your ideas!"),
		version: String::from("1.0.0"),
		host: String::from("127.0.0.1:6100")},
	    
	    herp: Herp{
		host: String::from("127.0.0.1:5050"),
		// Endpoints are parsed from JSON to show you, how parsing from JSON to serde_json::Value works.
		endpoints : json!({
		    "register" : "/services/register"
		})
	    },
	    // the service interface is parsed from JSON to show you, how parsing from JSON to serde_json::Value works.
	    service_interface : json!({
		"nodeDefinitions": [{
		    "name": "myService",
		    "label": "My Service Node",
		    "inputs" : [
			{
			    "fieldType" : "string",
			    "name" : "inputField1",
			    "label" : "String input field."
			}
		    ],
		    "outputs" : [
			{
			    "fieldType": "string",
			    "name": "outputField",
			    "label": "Output strings"
			}
		    ]
		}]
	    })
	}
    }

    fn load_config_from_file(filename:&str) -> Config{
	let file = File::open(filename).unwrap();
	let reader = BufReader::new(file);
	serde_json::from_reader(reader).unwrap()
    }
}

fn node_definition() -> Value {
    Config::load_config().service_interface
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    // Load configurations and extract relevant information
    let config = Config::load_config();
    let (service, herp) = (config.service, config.herp);

    // Setup and run Server to provide Service for hERP.
    let server = HttpServer::new(|| App::new()
				 .service(run)
				 .service(install_get)
				 .service(install_post))
	.bind(&service.host)?
        .run();

    println!("Service started on {}", herp.host);

    // Register running Server to hERP.
    //match register(&config.herp, &config.service, &client).await{
    match herp.register(&service).await{
	Ok(_) => println!("Registered service to herp on {}", service.host),
	Err(_) => panic!("Registration failed. Check if hERP Server is running on port {} !", herp.host), 
    }

    // Wait for further calls by hERP.
    server.await
}

/// This endpoint is called by herp if a user requests installation info.
///
/// The installation process includes vital information about inputs and outputs of the interface, that the *process* function plugs into.
/// Due to this it is neccesarry to adjust the head of *process* function and the *inputs* and *outputs* of the *nodeDefinition*.
#[get("/install")]
async fn install_get() -> Result<HttpResponse,Error> {

    let data = node_definition();
    Ok(HttpResponse::Ok().json(data))
}

/// The install post method will be called by herb after a successfull installation.
/// This can, for example, be used to populate database with data.
#[post("/install")]
async fn install_post(payload: web::Json<Credentials>) -> Result<HttpResponse,Error>{

    println!("service is installed for User {}", payload.name);
    payload.store();
    Ok(HttpResponse::Ok().json(()))
}

/// This endpoint is default and called whenever a node instance of this service gets triggerd inside a workflow.
/// HINT: JSON could also be deserialized into a struct. In this example, a more native, generic, implementiation is used. Due to this, processed payload fields have to be type-checked (otherwise unwrap might cause server to panic!!).
#[post("/do")]
async fn run(payload: web::Json<Value>) -> Result<HttpResponse,Error>{
    
    // Testing type of used payload Field (necessary for each used Value)
    if let (Value::String(input_field1), Value::String(input_field2)) = (&payload["inputField1"], &payload["inputField2"]){
	let result = process(input_field1, input_field2);

	// Be shure to stick to the defined output, so hERP is able to read you result(s).
	let data = json!({
	    "outputField" : result
	});
	
	println!("Request on endpoint /do with params {} and {} was responded with {}.",input_field1,input_field2,data);
	Ok(HttpResponse::Ok().json(data))
    }
    else{
	return Err(error::ErrorBadRequest("Input field required by process was not found or decal."));
    }
}


/// This function contains all the business logic you need for your service.
/// Feel free to extend this function by others to structure more complex services.
/// If you change the function's head be shure to
/// 1. Validate the data type in *run* before passing it in.
/// 2. Adjust the nodeDefinition used by *install* if necessary.
fn process(input_field_1 : &String, input_field_2: &String) -> String{
    
    // A.t.m. given inputs are concattenated. 
    // Feel abyolutely free to customize behaviour of this method.
    [input_field_1.as_str(),input_field_2.as_str()].join("")

}


mod service{
    use serde::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    pub struct Service{
	pub name: String,
	pub title: String,
	pub description: String,
	pub version: String,
	pub host: String
    }


    /// Credentials provided by hERP have to be processed and stored.
    /// By deriving Deserialize and Serialize, this struct is able to process Credentials in JSON.
    #[derive(Deserialize, Serialize, Debug)]
    pub struct Credentials{
	pub (super) name: String,
	pub (super) password: String,
    }

    impl Credentials{
	pub fn new(name: String, password: String) -> Credentials{
	    Credentials{name: name, password: password}
	}
	// TODO: Implement encrypted way to store Informations.

	/// Persist the given Credentials on filesystem.
	pub (super) fn store(&self){
	    confy::store("credentials", &self).unwrap();
	}

	/// Loads the give Credentials from filesystem.
	pub (super) fn load() -> Credentials{
	    match confy::load("credentials"){
		Ok(credentials) => credentials,
		Err(_) => Credentials::default(),
	    }
	}
    }
    /// Default credentials for test cases. Required by confy module (if used).
    impl ::std::default::Default for Credentials{
	fn default() -> Self{ Self{name: String::from("your-email@example.com"), password: String::from("admin")}}
    }
}

#[cfg(test)]
mod test {
    use actix_rt;
    use super::*;
    use actix_web::{test, body::Body};
    
    pub fn default_service() -> Service{
	Service {
	    name: String::from("rust.skeleton.test"),
	    title: String::from("Rust Skeleton Service Test"),
	    description: String::from("A service for testing purposes."),
	    version: String::from("0.0.1"),
	    host: String::from("127.0.0.1:6100")}
    }
    pub fn default_herp() -> Herp{
	Herp{
	    host: String::from("127.0.0.1:5050"),
	    endpoints : json!({ "register" : "/services/register" })
	}
    }

    #[test]
    fn load_config_test(){
	assert_eq!(Config::load_config_from_file("config.json"), Config::load_config());
    }
    #[actix_rt::test] // Requirement: hERP is running.
    async fn register_integrationtest(){
	assert_eq!(default_herp().register(&default_service()).await.unwrap(), ());
    }

    #[test]
    fn load_service_test(){
	assert_eq!(Config::load_config().service.name ,String::from("skeleton.herp.app"));
    }
    
    #[actix_rt::test]
    async fn server_endpoint_install_post_test() {
        let mut app = test::init_service(App::new().service(install_post)).await;

	let data = json!({"name": "testname", "password": "testpassword"});
	let data_wrong_key = json!({"namedd2": "testname", "password": "testpassword"});

	let req_with_data = test::TestRequest::post()
	    .header("content-type", "application/json")
	    .uri("/install")
	    .set_json(&data)
	    .to_request();
        let req_with_wrong_key = test::TestRequest::post()
	    .header("content-type", "application/json")
	    .uri("/install")
	    .set_json(&data_wrong_key)
	    .to_request();
	
        let resp = test::call_service(&mut app, req_with_data).await;
        assert!(resp.status().is_success());

	let resp = test::call_service(&mut app, req_with_wrong_key).await;
        assert!(resp.status().is_client_error());
    }
    
    #[actix_rt::test]
    async fn server_endpoint_install_get_test() {
	let mut app = test::init_service(App::new().service(install_get)).await;
	let est_resp = json!({
	    "nodeDefinitions": [{
		"name": "myService",
		"label": "My Service Node",
		"inputs" : [
		    {
			"fieldType" : "string",
			"name" : "inputField1",
			"label" : "String input field."
		    }
		],
		"outputs" : [
		    {
			"fieldType": "string",
			"name": "outputField",
			"label": "Output strings"
                    }
		]
	    }],
	});
	let req = test::TestRequest::get()
	    .header("content-type", "application/json")
	    .uri("/install")
	    .to_request();
	let mut resp = test::call_service(&mut app, req).await;
	assert!(resp.status().is_success());
	assert_eq!(resp.take_body().as_ref().unwrap(), &Body::from(est_resp));
    }
    
    #[actix_rt::test]
    async fn server_endpoint_do_post_test() {
        let mut app = test::init_service(App::new().service(run)).await;
	let data = json!({"inputField1": "value1", "inputField2": "value2"});
	let data_wrong_key = json!({"input": "value1"});
	
	let req_with_data = test::TestRequest::post()
	    .header("content-type", "application/json")
	    .uri("/do")
	    .set_json(&data)
	    .to_request();
	let req_with_wrong_data = test::TestRequest::post()
	    .header("content-type", "application/json")
	    .uri("/do")
	    .set_json(&data_wrong_key)
	    .to_request();
	let req_without_data = test::TestRequest::post()
	    .header("content-type", "application/json")
	    .uri("/do")
	    .to_request();
	
	let resp = test::call_service(&mut app, req_without_data).await;
	assert!(resp.status().is_client_error());

        let resp = test::call_service(&mut app, req_with_wrong_data).await;
	assert!(resp.status().is_client_error());
	
        let mut resp = test::call_service(&mut app, req_with_data).await;
	let est_resp = json!({"outputField" : "value1value2"});
	assert!(resp.status().is_success());
	assert_eq!(resp.take_body().as_ref().unwrap(), &Body::from(est_resp));	
	// Here you can add more Tests depending on what the do endpoint should do or return.
    }
}



