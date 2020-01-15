use std::io;
use std::cmp::Ordering;
use rand::Rng;
use std::sync::Mutex;

struct GuessGame {
    pub secret: u32,
    pub number_of_tries: Mutex<u32>,
    pub tries: Vec<Mutex<bool>>
}

enum PlayResult {
    AlreadyTried,
    Value(Ordering)
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
            tries 
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
            None => None
        } 
    }
}

fn main() {
   println!("Guess the Number!");

   let  mut game = GuessGame::new();

   println!("The secret number is: {}", game.secret);

   loop {
    println!("Please input your guess.");

    let mut guess = String::new();
 
    io::stdin().read_line(&mut guess).expect("Failed to read line");
 
    let guess: u32 = match guess.trim().parse() {
        Ok(num) => num,
        Err(_) => continue
    };
 
    println!("You guessed: {}", guess);
    
    match game.play(guess) {
        Some(result) => match result {
            PlayResult::AlreadyTried => println!("This value was already used!"),
            PlayResult::Value(value) => match value {
                Ordering::Less => println!("Too small!"),
                Ordering::Greater => println!("Too big!"),
                Ordering::Equal =>  {
                    println!("You win!");
                    break;
                }
            }
        },
        None => println!("The value should be between 1 and 100!")
    }
   }
}
