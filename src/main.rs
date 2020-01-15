use rand::Rng;
use std::cmp::Ordering;
use std::io;
use std::sync::Mutex;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

struct GuessGame {
    pub secret: u32,
    pub number_of_tries: Mutex<u32>,
    pub tries: Vec<Mutex<bool>>,
}

enum PlayResult {
    AlreadyTried,
    Value(Ordering),
}

impl GuessGame {
    fn new() -> GuessGame {
        let mut tries = Vec::with_capacity(100);
        for _ in 0..100 {
            tries.push(Mutex::new(false))
        }
        GuessGame {
            secret: rand::thread_rng().gen_range(1, 101),
            number_of_tries: Mutex::new(0),
            tries,
        }
    }

    fn play(&mut self, guess: u32) -> Option<PlayResult> {
        *self.number_of_tries.lock().unwrap() += 1;
        match self.tries.get(guess as usize) {
            Some(already_tried) => Some({
                let tried = *already_tried.lock().unwrap();
                if tried {
                    PlayResult::AlreadyTried
                } else {
                    *already_tried.lock().unwrap() = true;
                    PlayResult::Value(guess.cmp(&self.secret))
                }
            }),
            None => None,
        }
    }
}

async fn index(shared_state: web::Data<Mutex<GuessGame>>, path: web::Path<u32>) -> HttpResponse {
    let result = shared_state.lock().unwrap().play(path.into_inner());
    let number_of_tries: u32 = *shared_state.lock().unwrap().number_of_tries.lock().unwrap();
    let result = {
        match result {
            Some(result) => match result {
                PlayResult::AlreadyTried => "This value was already used!",
                PlayResult::Value(value) => match value {
                    Ordering::Less => "Too small!",
                    Ordering::Greater => "Too big!",
                    Ordering::Equal => {
                        *shared_state.lock().unwrap() = GuessGame::new();
                        "You win!"
                    }
                },
            },
            None => "The value should be between 1 and 100!",
        }
    };
    let body = format!(
        "{{\"number_of_tries\": {}, \"result\": \"{}\" }}\n",
        number_of_tries, result
    );
    HttpResponse::Ok().body(body)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    println!("Guess the Number!");
    let shared_state = web::Data::new(Mutex::new(GuessGame::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // register simple handler
            .service(web::resource("/{guess}").to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
