use std::io;

fn main() {
    println!("Guess the number!");

    let secret = rand::random_range(1..=100);

    println!("The secret number is: {}", secret);

    println!("Please input your guess.");

    let mut guess = String::new();

    loop {
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        let parsed = guess.trim().parse::<u32>().expect("not a number");
        match parsed.cmp(&secret) {
            std::cmp::Ordering::Equal => println!("you winn"),
            std::cmp::Ordering::Greater => println!("too big"),
            std::cmp::Ordering::Less => println!("too small"),
        }
        guess = "".to_string();
    }
}
