use actix_web::{client, get, post, web, App, HttpServer, HttpResponse, Error, error};
use serde_json::{json, Value};
use serde :: {Deserialize, Serialize};

const SERVICE_NAME : &str = "skeleton.herp.app";

const SERVICE_TITLE : &str = "Rust Skeleton Service";

const SERVICE_DESCRIPTION : &str = "A skeleton service with all necessary functionality to talk with herp but without any logic. Feel free to integrate your ideas!";

const SERVICE_VERSION : &str = "1.0.0";

const SERVICE_HOST : &str = "127.0.0.1:6100";

const HERP_HOST : &str = "127.0.0.1:5050";

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    // Setup client to interact with hERP Server
    let client = client::Client::default();

    // Setup and run Server to provide Service for hERP.
    let server = HttpServer::new(|| App::new()
				 .service(run)
				 .service(install_get)
				 .service(install_post))
	.bind(&SERVICE_HOST)?
        .run();

    println!("Service started on {}", &SERVICE_HOST);

    // Register running Server to hERP.
    match register(&HERP_HOST, &client).await{
	Ok(_) => println!("Registered service to herp on {}", SERVICE_HOST),
	Err(_) => panic!("Registration failed. Check if hERP Server is running on port {} !", HERP_HOST), 
    };

    // Wait for further calls by hERP.
    server.await
}


#[derive(Deserialize, Serialize, Debug)]
struct Credentials{
    name: String,
    password: String,
}

impl Credentials{
    // TODO: Implement encrypted way to store Informations.
    fn store(&self){
	confy::store("credentials", &self).unwrap();
    }
    fn load() -> Credentials{
	match confy::load("credentials"){
	    Ok(credentials) => credentials,
	    Err(_) => Credentials::default(),
	}
    }
}

impl ::std::default::Default for Credentials{
    fn default() -> Self{ Self{name: String::from("your-email@example.com"), password: String::from("admin")}}
}

async fn register(host: &str, client: &client::Client) -> Result<(), Box<dyn std::error::Error>>{
    let url = ["http://",host,"/services/register"].join("");
    let data = json!(
	{
	    "name": &SERVICE_NAME,
	    "host": host,
	    "title": &SERVICE_TITLE,
	    "version": &SERVICE_VERSION,
	    "description": &SERVICE_DESCRIPTION
	});
    let response = client
	.post(&url)
	.send_json(&data).await?;

    println!("response: {:?}", response);
    
    Ok(())
}

/**
 * This endpoint is called by herp if a user requests installation info.
 */
#[get("/install")]
async fn install_get() -> Result<HttpResponse,Error> {
    let data = json!({
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
    Ok(HttpResponse::Ok().json(data))
}

/**
 * The install post method will call by herb after a successfull installation.
 * This can be used to populate database with data
*/
#[post("/install")]
async fn install_post(payload: web::Json<Credentials>) -> Result<HttpResponse,Error>{
    println!("service is installed for User {}", payload.name);

    payload.store();

    let data = json!({"test": "ok"});
    Ok(HttpResponse::Ok().json(data))
}

/**
 * This endpoint is default and called whenever a node instance of this service gets triggerd inside a workflow
 */
#[post("/do")]
// JSON could also be deserialized into Struct. In this example, the simple, generic implementiation is used. Due to this, used payload field have to be type-checked (otherwise unwrap might cause Server to panic).
async fn run(payload: web::Json<Value>) -> Result<HttpResponse,Error>{
    println!("{:?}",payload);

    // Test type of used payload Field (necessary for each used Value)
    match &payload["inputField1"]{
	Value::String(string) => println!("Processing input {}",string),
	_ => return Err(error::ErrorBadRequest("test error"))
    }
    // Content of inputField1 is converted to String to assure the right type.    
    let result = process(payload["inputField1"].as_str().unwrap().to_string());

    let data = json!({
	"outputField" : result
     });

    Ok(HttpResponse::Ok().json(data))
}

/**
 * This function contains all the logic you need for your service.
 * Feel free to extend this function by others to structure more complex services.
 */
fn process(input_field : String) -> String{
    // At the moment input is simply reflected.
    // Feel free to customize behaviour of this method.
    input_field
}


#[cfg(test)]
mod test {
    use actix_rt;
    use super::*;
    use actix_web::{test, body::Body};
    

    #[actix_rt::test] // Requirement: hERP is running.
    async fn register_test(){
	let client = client::Client::default();
	let result = register(&HERP_HOST, &client).await;
	assert_eq!(result.unwrap(), ());
    }

    #[actix_rt::test]  // Requirement: hERP is running.
    #[should_panic(expected = "Connection refused")]
    async fn register_test_wrong_port(){
	let client = client::Client::default();
	let result = register("127.0.0.1:6000", &client).await;
	result.unwrap();
    }
    
    #[actix_rt::test]
    async fn test_server_endpoint_install_post() {
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
    async fn test_server_endpoint_install_get() {
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
    async fn test_server_endpoint_do_post() {
        let mut app = test::init_service(App::new().service(run)).await;
	let data = json!({"inputField1": "value1"});
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
	let est_resp = json!({"outputField" : "value1"});
	assert!(resp.status().is_success());
	assert_eq!(resp.take_body().as_ref().unwrap(), &Body::from(est_resp));	
	// Here you can add more Tests depending on what the do endpoint should do or return.
    }

}



mod herp_interaction {
    use serde_json::{json, Value};
    use actix_web::{client, error, Error};
    use super::{HERP_HOST, SERVICE_NAME, SERVICE_TITLE, Credentials};

    async fn get_token() -> Result<String,Error>{
	let client = client::Client::default();
	let url = ["http://",&HERP_HOST,"/users/login"].join("");

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

    async fn get_service_id(token: &str) -> Result<String,Error>{
	let client = client::Client::default();
	let url = ["http://",&HERP_HOST,"/content/system/service"].join("");

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
			x["name"].as_str()==Some(SERVICE_NAME)
			&& x["title"].as_str()==Some(SERVICE_TITLE));
	    return Ok(id.next().unwrap()["_id"].as_str().unwrap().to_string())
	}
	Err(error::ErrorBadRequest("Service not found."))
    }

    async fn klick_install() -> Result<String,Error>{
	if let Ok(token) = get_token().await{
	    let id = get_service_id(token.as_str()).await?;
	    println!("{}",id.as_str());
	    println!("{}",token);
	    let client = client::Client::default();
	    let url = ["http://",&HERP_HOST,"/services/install/",id.as_str()].join("");
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
    
    mod test {
	use super::*;

	#[actix_rt::test] // Requirement: hERP is running.
	async fn login_test(){
	    let credentials = Credentials{
		name : "your-email@example.com".to_string(),
		password: "admin".to_string()};
	    credentials.store();
	    let token = get_token().await.unwrap();
	    assert_eq!(token.len(),171);
	}

	#[actix_rt::test] // Requirement: hERP is running.
	async fn get_service_id_test(){
	    let credentials = Credentials{
		name : "your-email@example.com".to_string(),
		password: "admin".to_string()};
	    credentials.store();
	    let id = get_service_id(get_token().await.unwrap().as_str()).await;
	    println!("{:?}", id);
	    assert_eq!(id.unwrap().len(),24);
	}

	
	#[actix_rt::test] // Requirement: hERP is running.
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Installaton rejected.\"")]
	async fn klick_install_test(){
	    let credentials = Credentials{
		name : "your-email@example.com".to_string(),
		password: "admin".to_string()};
	    credentials.store();
	    klick_install().await.unwrap();
	}
    }
}
