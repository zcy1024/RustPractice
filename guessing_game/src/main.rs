use std::io;
use std::cmp::Ordering;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let target = rand::thread_rng().gen_range(1..=100);
    // println!("{target}");

    loop {
        let mut guess = String::new();
        println!("Please input your guess:");
        io::stdin().read_line(&mut guess).expect("Filed to read line.");

        let guess = match guess.trim().parse::<i32>() {
            Ok(num) => num,
            Err(_) => {
                println!("Please type a number!");
                continue;
            }
        };

        if guess < 0 || guess > 100 {
            println!("Error! Target number is between 1 and 100!");
            continue;
        }

        match guess.cmp(&target) {
            Ordering::Less => println!("{guess} is too small!"),
            Ordering::Greater => println!("{guess} is too big!"),
            Ordering::Equal => {
                println!("You Win!");
                break;
            }
        }
    }
}