<img src="https://herp.app/herp-logo.svg" width="200">

# hERP Service Rust Skeleton

This is an empty service you can start from to create your own service for [hERP](https://herp.app). 

If you are not familiar with Services read the [documentation here](https://herp.app/docs/services/hello-world-service/) to get started.

## Features

This skeleton does already:

* Starts a webserver with necessary endpoints **/do** and **/install**
* Store user credentials of the service user

## How to use

1. Clone
2. cargo run

## Where and how to adjust

### Where to - abstract
Basic information about your Service can be adjusted in the static variables **SERVICE_NAME** **SERVICE_TITLE** **SERVICE_DESCRIPTION** **SERVICE_VERSION** and **SERVICE_HOST**. 

Be shure to define **HERP_HOST** in the right way.

Use the function **process** in the main module to customize behaviour of the Service. 
This function is called by the function **run** which provides the necessary data. 
The payload of the **run** function is pushed by hERP and defined in the variable **data** in the **install_get** function.

### How to - Step by step
1. Provide basic information of your Service via capitalized variables.
2. Define inputs and outputs you need with correspondig data types via json in **install_get**.
3. Typecheck all json values, you want to use in **run** or implement and use deserializer there. 
4. Implement all business logic in **process**.
