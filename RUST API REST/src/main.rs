// Importaciones necesarias
use postgres::{Client, Error, NoTls};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

// Importación del sistema de serialización y deserialización de JSON
#[macro_use]
extern crate serde_derive;

// Define la estructura de datos para una Preorden
#[derive(Serialize, Deserialize, Debug)]
struct Preorder {
    pub id: i32,         // Identificador único
    pub name: String,    // Nombre del juego
    pub price: f64,      // Precio del juego
    pub genre: String,   // Género del juego
    pub email: String,   // Correo electrónico asociado
    pub release: String, // Fecha de lanzamiento
}

// Variables de entorno definidas en el archivo de Docker Compose para conectar a la base de datos
const DB_URL: &'static str = env!("DATABASE_URL");

fn main() {
    // Configura la base de datos al inicio de la aplicación
    match set_database() {
        Ok(_) => (),    
        Err(_) => ()    
    }

    // Crea un oyente de conexiones en el puerto 8080
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    // Acepta conexiones entrantes y las maneja
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),  // Llama a la función para manejar la conexión
            Err(e) => println!("Error: {}", e)   // Muestra un mensaje de error si ocurre un problema con la conexión
        }
    }
}

// Configura la base de datos: crea la tabla 'preorders' si no existe
fn set_database() -> Result<(), Error> {
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    // Crea la tabla 'preorders' si no existe, definiendo las columnas y restricciones
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS preorders (
            id      SERIAL PRIMARY KEY,
            name    VARCHAR NOT NULL,
            price   NUMERIC NOT NULL,
            genre   VARCHAR NOT NULL,
            email   VARCHAR NOT NULL,
            release VARCHAR NOT NULL
        )",
    )?;

    Ok(())
}

// Maneja una solicitud entrante desde un cliente
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);

            // Verifica el tipo de solicitud y llama a la función correspondiente para manejarla
            let (status_line, content) = if request.starts_with("POST /preorders HTTP/1.1") {
                handle_post_request(&request)
            } else if request.starts_with("GET /preorders HTTP/1.1") {
                ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), handle_get_all_request())
            } else if request.starts_with("GET /hello HTTP/1.1") {
                ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), "Hello world".to_owned())
            } else if request.starts_with("DELETE /preorders") {
                handle_delete_request(&request)
            } else if request.starts_with("PUT /preorders") {
                handle_update_request(&request)
            } else {
                println!("Request: {}", request);  // Muestra un mensaje de solicitud desconocida
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned(), "404 Not Found".to_owned())
            };

            let response = format!("{}{}", status_line, content);
            stream.write(response.as_bytes()).unwrap();   // Escribe la respuesta en el flujo de la conexión
            stream.flush().unwrap();   // Asegura que la respuesta se envíe por completo
        }
        Err(e) => {
            println!("Error: {}", e);   // Muestra un mensaje de error si ocurre un problema con la lectura de la solicitud
        }
    }
}

// Maneja una solicitud de actualización (PUT) de una preorden
fn handle_update_request(request: &str) -> (String, String)  {    
    match update_one(request) { 
        Ok(_) => (),    
        Err(_) => ()    
    }
    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Update preorder"),)  // Retorna una respuesta exitosa
}

// Actualiza una preorden en la base de datos
fn update_one(request: &str) -> Result<(), Error>  {   
    let request_body = request.split("\r\n\r\n").last().unwrap_or("");
    let preorder: Preorder = serde_json::from_str(request_body).unwrap();
    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    let mut request_split = request.split(" ");
    let mut request_split2 = request_split.nth(1).unwrap().split("?");
    let mut request_split3 = request_split2.nth(1).unwrap().split("=");
    let mut request_split4 = request_split3.nth(1).unwrap().split(" ");
    let id = request_split4.nth(0).unwrap();
    let id = id.parse::<i32>().unwrap();

    // Ejecuta una actualización en la base de datos
    client.execute("UPDATE preorders SET name=$2, price=$3, genre=$4, email=$5, release=$6 WHERE id=$1", &[&id, &preorder.name, &preorder.price, &preorder.genre, &preorder.email, &preorder.release]).unwrap();
    Ok(())
}

// Maneja una solicitud de eliminación (DELETE) de una preorden
fn handle_delete_request(request: &str) -> (String, String)  {    
    match delete_one(request) { 
        Ok(_) => (),    
        Err(_) => ()    
    }

    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Deleted preorder"),)  // Retorna una respuesta exitosa
}

// Elimina una preorden de la base de datos
fn delete_one(request: &str) -> Result<(), Error>  {    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    let mut request_split = request.split(" ");
    let mut request_split2 = request_split.nth(1).unwrap().split("?");
    let mut request_split3 = request_split2.nth(1).unwrap().split("=");
    let mut request_split4 = request_split3.nth(1).unwrap().split(" ");
    let id = request_split4.nth(0).unwrap();
    let id = id.parse::<i32>().unwrap();

    // Ejecuta una eliminación en la base de datos
    client.execute("DELETE FROM preorders WHERE id=$1", &[&id]).unwrap();
    Ok(())
}

// Maneja una solicitud para obtener todas las preórdenes
fn handle_get_all_request() -> String {
    let mut client = Client::connect(DB_URL,NoTls,).unwrap();

    let mut preorders: Vec<Preorder> = Vec::new();

    // Recupera todas las preórdenes de la base de datos
    for row in client.query("SELECT id, name, price, genre, email, release FROM preorders", &[]).unwrap() {
        let id: i32 = row.get(0);
        let name: String = row.get(1);
        let price: f64 = row.get(2);
        let genre: String = row.get(3);
        let email: String = row.get(4);
        let release: String = row.get(5);

        let preorder = Preorder {
            id,
            name,
            price,
            genre,
            email,
            release,
        };

        preorders.push(preorder);
    }

    // Convierte la lista de preórdenes en formato JSON
    let preorders_json = serde_json::to_string(&preorders).unwrap();
    preorders_json
}

// Maneja una solicitud de creación (POST) de una preorden
fn handle_post_request(request: &str) -> (String, String)  {    
    let request_body = request.split("\r\n\r\n").last().unwrap_or("");

    // Crea una nueva preorden en la base de datos
    match create_one(request_body) { 
        Ok(_) => (),    
        Err(_) => ()    
    }

    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Received data: {}", request_body),)  // Retorna una respuesta exitosa
}

// Crea una nueva preorden en la base de datos
fn create_one(request_body: &str) -> Result<(), Error> {
    let preorder: Preorder = serde_json::from_str(request_body).unwrap();
    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    // Ejecuta una inserción en la base de datos
    client.execute(
        "INSERT INTO preorders (name, price, genre, email, release) VALUES ($1, $2, $3, $4, $5)",
        &[&preorder.name, &preorder.price, &preorder.genre, &preorder.email, &preorder.release],
    )?;

    Ok(())
}
