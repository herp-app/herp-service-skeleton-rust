use actix_web::{client, get, post, web, App, HttpServer, HttpResponse, Error};
use serde_json::{json, Value};

const SERVICE_HOST : &str = "127.0.0.1:6100";
const HERP_HOST : &str = "127.0.0.1:5050";

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let client = client::Client::default();
    let server = HttpServer::new(|| App::new().service(install).service(run))
        .bind(&SERVICE_HOST)?
    .run();
    println!("Service started on {}", &SERVICE_HOST);
    match register(&HERP_HOST, &client).await{
	Ok(_) => println!("Registered service to herp on {}", SERVICE_HOST),
	Err(_) => panic!("Registration failed. Check if hERP Server is running on port {} !", HERP_HOST), 
    };
    server.await
}

async fn register(host: &str, client: &client::Client) -> Result<(), Box<dyn std::error::Error>>{
    let url = ["http://",host,"/services/register"].join("");
    let data = json!(
	{
	    "name": "myService",
	    "host": host,
	    "title": "My Service",
	    "version": "1.0.0",
	    "description":"Example Workflow"
	});
    let response = client
	.post(&url)
	.send_json(&data).await?;

    println!("response: {:?}", response);
    Ok(())
}

#[get("/install")]
async fn install() -> Result<HttpResponse,Error> {
    let data = json!({
	"nodeDefinitions": [{
	    "name": "myService",
	    "label": "My Service Node",
	}],
    });
    Ok(HttpResponse::Ok().json(data))
}


#[post("/do")]
// JSON can be deserialized into Struct, but in this example, this is not the case.
async fn run(payload: web::Json<Value>) -> Result<HttpResponse,Error>{
    println!("{:?}",payload);

    let data = json!({
	"nodeDefinitions": [{
	    "name": "myService",
	    "label": "My Service Node",
	}],
     });
    Ok(HttpResponse::Ok().json(data))
}

use actix_rt;

#[cfg(test)]
mod test {
    use super::*;
    #[actix_rt::test]
    // Requirement: hERP is running.
    async fn register_test(){
	let client = client::Client::default();
	let result = register(&HERP_HOST, &client).await;
	assert_eq!(result.unwrap(), ());
    }

    #[actix_rt::test]
    #[should_panic(expected = "Connection refused")]
    async fn register_test_wrong_port(){
	let client = client::Client::default();
	let result = register("127.0.0.1:6000", &client).await;
	result.unwrap();
    }
}
